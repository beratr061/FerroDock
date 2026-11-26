mod database;
mod pipeline;
mod volume;
mod work;

use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use ferrodock_core::crypto::SecretBox;
use ferrodock_core::{AppError, AppResult, Id};
use ferrodock_db::backups::{BACKUP_RESTORE_JOB, BACKUP_RUN_JOB, RunOutcome};
use ferrodock_db::jobs::JobRecord;
use ferrodock_db::{
    ApplicationRepo, BackupDestinationRepo, BackupPolicyRepo, BackupRunRepo, DatabaseRepo,
};
use ferrodock_infra::docker::DockerEngine;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use self::work::Work;
use crate::jobs::JobHandler;

#[derive(Clone)]
pub struct BackupJobContext {
    pub docker: DockerEngine,
    pub databases: Arc<dyn DatabaseRepo>,
    pub applications: Arc<dyn ApplicationRepo>,
    pub destinations: Arc<dyn BackupDestinationRepo>,
    pub policies: Arc<dyn BackupPolicyRepo>,
    pub runs: Arc<dyn BackupRunRepo>,
    pub secrets: SecretBox,
    pub local_root: PathBuf,
}

impl std::fmt::Debug for BackupJobContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("BackupJobContext")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Operation {
    Backup,
    Restore,
}

#[derive(Debug, Clone)]
pub struct ScheduledBackupHandler {
    ctx: Arc<BackupJobContext>,
    operation: Operation,
}

fn run_id(job: &JobRecord) -> AppResult<Id> {
    job.payload
        .get("run_id")
        .and_then(|value| value.as_str())
        .and_then(|value| Uuid::parse_str(value).ok())
        .map(Id::from)
        .ok_or_else(|| AppError::validation("JOB_PAYLOAD_INVALID"))
}

impl ScheduledBackupHandler {
    pub fn backups(ctx: BackupJobContext) -> Self {
        Self {
            ctx: Arc::new(ctx),
            operation: Operation::Backup,
        }
    }

    #[must_use]
    pub fn restores(&self) -> Self {
        Self {
            ctx: self.ctx.clone(),
            operation: Operation::Restore,
        }
    }

    async fn execute(&self, job: &JobRecord, id: Id) -> AppResult<RunOutcome> {
        let work = Work::load(&self.ctx, job, id).await?;
        self.ctx.runs.start_run(id).await?;
        match self.operation {
            Operation::Backup => work.back_up().await,
            Operation::Restore => work.restore().await,
        }
    }
}

#[async_trait]
impl JobHandler for ScheduledBackupHandler {
    fn kind(&self) -> &'static str {
        match self.operation {
            Operation::Backup => BACKUP_RUN_JOB,
            Operation::Restore => BACKUP_RESTORE_JOB,
        }
    }

    async fn run(&self, job: &JobRecord, _cancel: CancellationToken) -> AppResult<()> {
        let id = run_id(job)?;
        let result = self.execute(job, id).await;
        let outcome = match &result {
            Ok(outcome) => outcome.clone(),
            Err(error) => RunOutcome {
                status: "failed",
                object_key: None,
                size_bytes: None,
                error_code: Some(error.code().to_owned()),
            },
        };
        if let Err(error) = self.ctx.runs.finish_run(id, outcome).await {
            tracing::warn!(code = error.code(), "backup run not updated");
        }
        result.map(|_| ())
    }
}
