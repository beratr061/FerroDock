use ferrodock_core::{AppError, AppResult, Id};
use ferrodock_db::backups::{BackupTarget, DestinationRecord, RunOutcome, RunRecord};
use ferrodock_db::databases::DatabaseRecord;
use ferrodock_db::jobs::JobRecord;
use ferrodock_domain::databases::driver;
use ferrodock_domain::scheduled_backups::{open_destination, storage_scope};
use ferrodock_infra::docker::application_volume;
use ferrodock_infra::object_storage::ObjectStorage;
use time::OffsetDateTime;
use time::macros::format_description;

use super::BackupJobContext;

const SEALED_SUFFIX: &str = "fdenc";
pub(super) const VOLUME_EXTENSION: &str = "tar.gz";

pub(super) enum Subject {
    Database(Box<DatabaseRecord>),
    Volume {
        application_id: Id,
        docker_volume: String,
    },
}

pub(super) struct Work<'a> {
    pub ctx: &'a BackupJobContext,
    pub run: RunRecord,
    pub subject: Subject,
    pub destination: DestinationRecord,
    pub storage: ObjectStorage,
}

fn object_key(run: &RunRecord, extension: &str) -> AppResult<String> {
    let stamp = OffsetDateTime::now_utc()
        .format(format_description!(
            "[year][month][day]-[hour][minute][second]"
        ))
        .map_err(AppError::internal)?;
    let short = run.id.to_string();
    let short = &short[short.len() - 8..];
    Ok(format!("{stamp}-{short}.{extension}.{SEALED_SUFFIX}"))
}

async fn subject(
    ctx: &BackupJobContext,
    organization_id: Id,
    run: &RunRecord,
) -> AppResult<Subject> {
    match &run.target {
        BackupTarget::Database(id) => ctx
            .databases
            .find(organization_id, *id)
            .await?
            .map(|record| Subject::Database(Box::new(record)))
            .ok_or_else(|| AppError::not_found("DATABASE_NOT_FOUND")),
        BackupTarget::Volume {
            application_id,
            volume,
        } => {
            ctx.applications
                .find(organization_id, *application_id)
                .await?
                .ok_or_else(|| AppError::not_found("APPLICATION_NOT_FOUND"))?;
            Ok(Subject::Volume {
                application_id: *application_id,
                docker_volume: application_volume(*application_id, volume),
            })
        }
    }
}

impl<'a> Work<'a> {
    pub async fn load(ctx: &'a BackupJobContext, job: &JobRecord, id: Id) -> AppResult<Self> {
        let not_found = || AppError::not_found("BACKUP_NOT_FOUND");
        let organization_id = job.organization_id.ok_or_else(not_found)?;
        let run = ctx.runs.run(id).await?.ok_or_else(not_found)?;
        let subject = subject(ctx, organization_id, &run).await?;
        let destination_id = run
            .destination_id
            .ok_or_else(|| AppError::not_found("BACKUP_DESTINATION_NOT_FOUND"))?;
        let destination = ctx
            .destinations
            .destination(organization_id, destination_id)
            .await?
            .ok_or_else(|| AppError::not_found("BACKUP_DESTINATION_NOT_FOUND"))?;
        let scope = storage_scope(&run.target);
        let storage =
            open_destination(&destination, &ctx.secrets, &ctx.local_root, Some(&scope)).await?;
        Ok(Self {
            ctx,
            run,
            subject,
            destination,
            storage,
        })
    }

    fn extension(&self) -> AppResult<&'static str> {
        match &self.subject {
            Subject::Database(database) => Ok(driver(&database.engine)?.dump_extension()),
            Subject::Volume { .. } => Ok(VOLUME_EXTENSION),
        }
    }

    pub async fn back_up(&self) -> AppResult<RunOutcome> {
        let key = object_key(&self.run, self.extension()?)?;
        let size = match &self.subject {
            Subject::Database(database) => self.dump_database(database, &key).await?,
            Subject::Volume { docker_volume, .. } => {
                self.archive_volume(docker_volume, &key).await?
            }
        };
        self.prune().await;
        Ok(RunOutcome {
            status: "succeeded",
            object_key: Some(key),
            size_bytes: Some(size),
            error_code: None,
        })
    }

    pub async fn restore(&self) -> AppResult<RunOutcome> {
        let not_found = || AppError::not_found("BACKUP_NOT_FOUND");
        let source_id = self.run.source_run_id.ok_or_else(not_found)?;
        let source = self.ctx.runs.run(source_id).await?.ok_or_else(not_found)?;
        let key = source.object_key.ok_or_else(not_found)?;
        self.verify(&key).await?;
        match &self.subject {
            Subject::Database(database) => self.restore_database(database, &key).await?,
            Subject::Volume {
                application_id,
                docker_volume,
            } => {
                self.restore_volume(*application_id, docker_volume, &key)
                    .await?;
            }
        }
        Ok(RunOutcome {
            status: "succeeded",
            object_key: None,
            size_bytes: None,
            error_code: None,
        })
    }

    async fn prune(&self) {
        if let Err(error) = self.try_prune().await {
            tracing::warn!(code = error.code(), "old backups not pruned");
        }
    }

    async fn try_prune(&self) -> AppResult<()> {
        let target = &self.run.target;
        let Some(policy) = self.ctx.policies.policy(target).await? else {
            return Ok(());
        };
        if policy.destination_id != self.destination.id {
            return Ok(());
        }
        let stored = self
            .ctx
            .runs
            .stored_backups(target, self.destination.id)
            .await?;
        let kept = usize::from(policy.retention).saturating_sub(1);
        for stale in stored
            .into_iter()
            .filter(|run| run.id != self.run.id)
            .skip(kept)
        {
            if let Some(key) = &stale.object_key {
                self.storage.delete(key).await?;
            }
            self.ctx.runs.delete_run(stale.id).await?;
        }
        Ok(())
    }
}
