use ferrodock_core::{AppError, AppResult, Id};
use ferrodock_db::backups::{
    DATABASE_BACKUP_JOB, DATABASE_RESTORE_JOB, DuePolicy, NewRun, RunOutcome, RunRecord,
};
use ferrodock_db::jobs::{LOCAL_QUEUE, NewJob};
use serde_json::json;

use super::model::BackupRun;
use super::service::DatabaseBackupService;
use crate::audit::Actor;
use crate::auth::{Principal, Role};

const LISTED_RUNS: i64 = 50;

pub(super) struct RunRequest {
    organization_id: Id,
    run: NewRun,
}

impl RunRequest {
    pub(super) fn scheduled(due: &DuePolicy) -> Self {
        Self {
            organization_id: due.organization_id,
            run: NewRun {
                id: Id::new(),
                database_id: due.policy.database_id,
                destination_id: due.policy.destination_id,
                operation: "backup",
                trigger: "schedule",
                source_run_id: None,
                created_by: None,
            },
        }
    }
}

impl DatabaseBackupService {
    pub async fn runs(&self, principal: &Principal, database_id: Id) -> AppResult<Vec<BackupRun>> {
        self.database(principal, database_id).await?;
        let records = self.parts.runs.runs(database_id, LISTED_RUNS).await?;
        Ok(records.into_iter().map(Into::into).collect())
    }

    pub async fn back_up_now(
        &self,
        actor: Actor<'_>,
        database_id: Id,
        destination_id: Id,
    ) -> AppResult<BackupRun> {
        actor.principal.require(Role::Developer)?;
        self.database(actor.principal, database_id).await?;
        self.destination(actor.principal, destination_id).await?;
        let request = RunRequest {
            organization_id: actor.principal.organization_id,
            run: NewRun {
                id: Id::new(),
                database_id,
                destination_id,
                operation: "backup",
                trigger: "manual",
                source_run_id: None,
                created_by: Some(actor.principal.user_id),
            },
        };
        let run = self.enqueue(request).await?;
        self.parts
            .audit
            .change(
                actor,
                "database.backup_requested",
                database_id,
                json!({ "run_id": run.id }),
            )
            .await;
        Ok(run)
    }

    pub async fn restore(&self, actor: Actor<'_>, run_id: Id) -> AppResult<BackupRun> {
        actor.principal.require(Role::Admin)?;
        let source = self.stored(actor.principal, run_id).await?;
        let destination_id = source
            .destination_id
            .ok_or_else(|| AppError::validation("BACKUP_DESTINATION_NOT_FOUND"))?;
        let request = RunRequest {
            organization_id: actor.principal.organization_id,
            run: NewRun {
                id: Id::new(),
                database_id: source.database_id,
                destination_id,
                operation: "restore",
                trigger: "manual",
                source_run_id: Some(source.id),
                created_by: Some(actor.principal.user_id),
            },
        };
        let run = self.enqueue(request).await?;
        let metadata = json!({ "run_id": run.id, "source_run_id": source.id });
        self.parts
            .audit
            .change(
                actor,
                "database.restore_requested",
                source.database_id,
                metadata,
            )
            .await;
        Ok(run)
    }

    pub async fn delete_backup(&self, actor: Actor<'_>, run_id: Id) -> AppResult<()> {
        actor.principal.require(Role::Admin)?;
        let source = self.stored(actor.principal, run_id).await?;
        if let (Some(destination_id), Some(key)) = (source.destination_id, &source.object_key) {
            let destination = self.destination(actor.principal, destination_id).await?;
            let storage = self.open(&destination, Some(source.database_id)).await?;
            storage.delete(key).await?;
        }
        self.parts.runs.delete_run(run_id).await?;
        self.parts
            .audit
            .change(
                actor,
                "database.backup_deleted",
                source.database_id,
                json!({ "run_id": run_id }),
            )
            .await;
        Ok(())
    }

    async fn stored(&self, principal: &Principal, run_id: Id) -> AppResult<RunRecord> {
        let not_found = || AppError::not_found("BACKUP_NOT_FOUND");
        let run = self.parts.runs.run(run_id).await?.ok_or_else(not_found)?;
        self.database(principal, run.database_id)
            .await
            .map_err(|_| not_found())?;
        let usable =
            run.operation == "backup" && run.status == "succeeded" && run.object_key.is_some();
        if usable { Ok(run) } else { Err(not_found()) }
    }

    pub(super) async fn enqueue(&self, request: RunRequest) -> AppResult<BackupRun> {
        let run = request.run;
        let kind = if run.operation == "restore" {
            DATABASE_RESTORE_JOB
        } else {
            DATABASE_BACKUP_JOB
        };
        let (id, database_id) = (run.id, run.database_id);
        self.parts.runs.create_run(run).await?;
        let job = NewJob {
            organization_id: Some(request.organization_id),
            kind: kind.to_owned(),
            queue: LOCAL_QUEUE.to_owned(),
            group_key: Some(format!("database:{database_id}")),
            payload: json!({ "run_id": id }),
        };
        if let Err(error) = self.parts.jobs.enqueue(job).await {
            let outcome = RunOutcome {
                status: "failed",
                object_key: None,
                size_bytes: None,
                error_code: Some(error.code().to_owned()),
            };
            self.parts.runs.finish_run(id, outcome).await?;
            return Err(error);
        }
        let record = self.parts.runs.run(id).await?;
        record
            .map(Into::into)
            .ok_or_else(|| AppError::not_found("BACKUP_NOT_FOUND"))
    }
}
