use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use ferrodock_core::crypto::SecretBox;
use ferrodock_core::{AppError, AppResult, Id};
use ferrodock_db::DatabaseRepo;
use ferrodock_db::databases::{DATABASE_DEPLOY_JOB, DatabaseRecord};
use ferrodock_db::jobs::JobRecord;
use ferrodock_domain::databases::{DatabaseStatus, container_spec};
use ferrodock_infra::deploy_log::{DeployLog, DeployLogs};
use ferrodock_infra::docker::{DATABASE_LABEL, DatabaseSpec, DockerEngine};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::jobs::JobHandler;

const READINESS_TIMEOUT: Duration = Duration::from_secs(300);

#[derive(Clone)]
pub struct DatabaseContext {
    pub docker: DockerEngine,
    pub repo: Arc<dyn DatabaseRepo>,
    pub secrets: SecretBox,
    pub logs: DeployLogs,
}

impl std::fmt::Debug for DatabaseContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("DatabaseContext")
    }
}

#[derive(Debug, Clone)]
pub struct DatabaseHandler {
    ctx: DatabaseContext,
}

fn database_id(job: &JobRecord) -> AppResult<Id> {
    job.payload
        .get("database_id")
        .and_then(|value| value.as_str())
        .and_then(|value| Uuid::parse_str(value).ok())
        .map(Id::from)
        .ok_or_else(|| AppError::validation("JOB_PAYLOAD_INVALID"))
}

impl DatabaseHandler {
    pub fn new(ctx: DatabaseContext) -> Self {
        Self { ctx }
    }

    async fn record(&self, job: &JobRecord, id: Id) -> AppResult<DatabaseRecord> {
        let not_found = || AppError::not_found("DATABASE_NOT_FOUND");
        let organization_id = job.organization_id.ok_or_else(not_found)?;
        self.ctx
            .repo
            .find(organization_id, id)
            .await?
            .ok_or_else(not_found)
    }

    async fn deploy(
        &self,
        record: &DatabaseRecord,
        log: &DeployLog,
        cancel: &CancellationToken,
    ) -> AppResult<()> {
        let docker = &self.ctx.docker;
        let spec = container_spec(record, &self.ctx.secrets)?;
        docker.ensure_network().await?;
        docker.pull_image(&spec.image, log, cancel).await?;
        let owner = record.id.to_string();
        docker
            .ensure_volume(&spec.volume, (DATABASE_LABEL, &owner))
            .await?;
        self.replace_containers(record.id, log).await?;
        log.step(&format!("Starting container {}", spec.name)).await;
        self.start_and_verify(&spec, log, cancel).await
    }

    async fn replace_containers(&self, id: Id, log: &DeployLog) -> AppResult<()> {
        let docker = &self.ctx.docker;
        for container in docker.database_containers(Some(&id.to_string())).await? {
            log.step("Removing previous container").await;
            if container.running {
                docker.stop_workload(&container.id).await?;
            }
            docker.remove_workload(&container.id).await?;
        }
        Ok(())
    }

    async fn start_and_verify(
        &self,
        spec: &DatabaseSpec,
        log: &DeployLog,
        cancel: &CancellationToken,
    ) -> AppResult<()> {
        let docker = &self.ctx.docker;
        let id = docker.run_database(spec).await?;
        log.step("Waiting for the database to accept connections")
            .await;
        let ready = docker
            .wait_ready(&id, None, READINESS_TIMEOUT, cancel)
            .await;
        if ready.is_err() && !cancel.is_cancelled() {
            log.step("Database did not become ready; last output:")
                .await;
            for line in docker.recent_output(&id).await {
                log.line(&line).await;
            }
            let _ = docker.remove_workload(&id).await;
        }
        ready
    }

    async fn settle(&self, id: Id, result: &AppResult<()>, cancel: &CancellationToken) {
        let (status, code) = match result {
            Ok(()) => (DatabaseStatus::Running, None),
            Err(_) if cancel.is_cancelled() => return,
            Err(error) => (DatabaseStatus::Failed, Some(error.code())),
        };
        if let Err(error) = self.ctx.repo.set_status(id, status.as_str(), code).await {
            tracing::warn!(code = error.code(), "database status not saved");
        }
    }
}

#[async_trait]
impl JobHandler for DatabaseHandler {
    fn kind(&self) -> &'static str {
        DATABASE_DEPLOY_JOB
    }

    async fn run(&self, job: &JobRecord, cancel: CancellationToken) -> AppResult<()> {
        let id = database_id(job)?;
        let record = self.record(job, id).await?;
        self.ctx.logs.remove(id).await;
        let log = self.ctx.logs.open(id).await?;
        let result = self.deploy(&record, &log, &cancel).await;
        match &result {
            Ok(()) => log.step("Database is ready").await,
            Err(_) if cancel.is_cancelled() => log.step("Deployment interrupted").await,
            Err(error) => log.step(&format!("Failed: {}", error.code())).await,
        }
        log.flush().await;
        self.settle(id, &result, &cancel).await;
        result
    }
}
