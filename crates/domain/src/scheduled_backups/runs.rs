use ferrodock_core::{AppError, AppResult, Id};
use ferrodock_db::backups::{
    BACKUP_RESTORE_JOB, BACKUP_RUN_JOB, BackupTarget, DuePolicy, NewRun, RunOutcome, RunRecord,
};
use ferrodock_db::jobs::{LOCAL_QUEUE, NewJob};
use serde_json::json;

use super::model::BackupRun;
use super::service::ScheduledBackupService;
use super::targets::{describe, resource_id, storage_scope};
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
                target: due.policy.target.clone(),
                destination_id: due.policy.destination_id,
                operation: "backup",
                trigger: "schedule",
                source_run_id: None,
                created_by: None,
            },
        }
    }

    fn manual(actor: Actor<'_>, target: BackupTarget, destination_id: Id) -> Self {
        Self {
            organization_id: actor.principal.organization_id,
            run: NewRun {
                id: Id::new(),
                target,
                destination_id,
                operation: "backup",
                trigger: "manual",
                source_run_id: None,
                created_by: Some(actor.principal.user_id),
            },
        }
    }
}

impl ScheduledBackupService {
    pub async fn runs(
        &self,
        principal: &Principal,
        target: &BackupTarget,
    ) -> AppResult<Vec<BackupRun>> {
        self.authorize(principal, target).await?;
        let records = self.parts.runs.runs(target, LISTED_RUNS).await?;
        Ok(records.into_iter().map(Into::into).collect())
    }

    pub async fn back_up_now(
        &self,
        actor: Actor<'_>,
        target: BackupTarget,
        destination_id: Id,
    ) -> AppResult<BackupRun> {
        actor.principal.require(Role::Developer)?;
        self.authorize(actor.principal, &target).await?;
        self.destination(actor.principal, destination_id).await?;
        let (resource, mut metadata) = (resource_id(&target), describe(&target));
        let run = self
            .enqueue(RunRequest::manual(actor, target, destination_id))
            .await?;
        metadata["run_id"] = json!(run.id);
        self.parts
            .audit
            .change(actor, "backup.requested", resource, metadata)
            .await;
        Ok(run)
    }

    pub async fn restore(&self, actor: Actor<'_>, run_id: Id) -> AppResult<BackupRun> {
        actor.principal.require(Role::Admin)?;
        let source = self.stored(actor.principal, run_id).await?;
        let destination_id = source
            .destination_id
            .ok_or_else(|| AppError::validation("BACKUP_DESTINATION_NOT_FOUND"))?;
        let mut request = RunRequest::manual(actor, source.target.clone(), destination_id);
        request.run.operation = "restore";
        request.run.source_run_id = Some(source.id);
        let run = self.enqueue(request).await?;
        let mut metadata = describe(&source.target);
        metadata["run_id"] = json!(run.id);
        metadata["source_run_id"] = json!(source.id);
        self.parts
            .audit
            .change(
                actor,
                "backup.restore_requested",
                resource_id(&source.target),
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
            let scope = storage_scope(&source.target);
            let storage = self.open(&destination, Some(&scope)).await?;
            storage.delete(key).await?;
        }
        self.parts.runs.delete_run(run_id).await?;
        let mut metadata = describe(&source.target);
        metadata["run_id"] = json!(run_id);
        self.parts
            .audit
            .change(
                actor,
                "backup.deleted",
                resource_id(&source.target),
                metadata,
            )
            .await;
        Ok(())
    }

    async fn stored(&self, principal: &Principal, run_id: Id) -> AppResult<RunRecord> {
        let not_found = || AppError::not_found("BACKUP_NOT_FOUND");
        let run = self.parts.runs.run(run_id).await?.ok_or_else(not_found)?;
        self.authorize(principal, &run.target)
            .await
            .map_err(|_| not_found())?;
        let usable =
            run.operation == "backup" && run.status == "succeeded" && run.object_key.is_some();
        if usable { Ok(run) } else { Err(not_found()) }
    }

    pub(super) async fn enqueue(&self, request: RunRequest) -> AppResult<BackupRun> {
        let run = request.run;
        let kind = if run.operation == "restore" {
            BACKUP_RESTORE_JOB
        } else {
            BACKUP_RUN_JOB
        };
        let (id, group_key) = (run.id, run.target.group_key());
        self.parts.runs.create_run(run).await?;
        let job = NewJob {
            organization_id: Some(request.organization_id),
            kind: kind.to_owned(),
            queue: LOCAL_QUEUE.to_owned(),
            group_key: Some(group_key),
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
