use ferrodock_core::{AppError, AppResult, Id};
use ferrodock_db::backups::{DestinationRecord, RunOutcome, RunRecord};
use ferrodock_db::databases::DatabaseRecord;
use ferrodock_db::jobs::JobRecord;
use ferrodock_domain::database_backups::open_destination;
use ferrodock_infra::object_storage::ObjectStorage;
use time::OffsetDateTime;
use time::macros::format_description;

use super::BackupJobContext;

const SEALED_SUFFIX: &str = "fdenc";

pub(super) struct Work<'a> {
    pub ctx: &'a BackupJobContext,
    pub run: RunRecord,
    pub database: DatabaseRecord,
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

impl<'a> Work<'a> {
    pub async fn load(ctx: &'a BackupJobContext, job: &JobRecord, id: Id) -> AppResult<Self> {
        let not_found = || AppError::not_found("BACKUP_NOT_FOUND");
        let organization_id = job.organization_id.ok_or_else(not_found)?;
        let run = ctx.runs.run(id).await?.ok_or_else(not_found)?;
        let database = ctx
            .databases
            .find(organization_id, run.database_id)
            .await?
            .ok_or_else(|| AppError::not_found("DATABASE_NOT_FOUND"))?;
        let destination_id = run
            .destination_id
            .ok_or_else(|| AppError::not_found("BACKUP_DESTINATION_NOT_FOUND"))?;
        let destination = ctx
            .destinations
            .destination(organization_id, destination_id)
            .await?
            .ok_or_else(|| AppError::not_found("BACKUP_DESTINATION_NOT_FOUND"))?;
        let storage = open_destination(
            &destination,
            &ctx.secrets,
            &ctx.local_root,
            Some(database.id),
        )
        .await?;
        Ok(Self {
            ctx,
            run,
            database,
            destination,
            storage,
        })
    }

    pub async fn container(&self) -> AppResult<String> {
        self.ctx
            .docker
            .database_containers(Some(&self.database.id.to_string()))
            .await?
            .into_iter()
            .find(|container| container.running)
            .map(|container| container.id)
            .ok_or_else(|| AppError::conflict("DATABASE_NOT_RUNNING"))
    }

    pub async fn back_up(&self) -> AppResult<RunOutcome> {
        let engine = ferrodock_domain::databases::driver(&self.database.engine)?;
        let container = self.container().await?;
        let dump = self
            .ctx
            .docker
            .dump_with(&container, &engine.dump_script())
            .await?;
        let sealed = self.ctx.secrets.seal_bytes(&dump)?;
        let size = i64::try_from(sealed.len()).unwrap_or(i64::MAX);
        let key = object_key(&self.run, engine.dump_extension())?;
        self.storage.put(&key, sealed).await?;
        self.prune().await;
        Ok(RunOutcome {
            status: "succeeded",
            object_key: Some(key),
            size_bytes: Some(size),
            error_code: None,
        })
    }

    async fn prune(&self) {
        if let Err(error) = self.try_prune().await {
            tracing::warn!(code = error.code(), "old backups not pruned");
        }
    }

    async fn try_prune(&self) -> AppResult<()> {
        let Some(policy) = self.ctx.policies.policy(self.database.id).await? else {
            return Ok(());
        };
        if policy.destination_id != self.destination.id {
            return Ok(());
        }
        let stored = self
            .ctx
            .runs
            .stored_backups(self.database.id, self.destination.id)
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
