mod artifact;
mod context;
mod release;

use std::sync::Arc;

use async_trait::async_trait;
use ferrodock_core::{AppError, AppResult, Id};

use ferrodock_db::jobs::JobRecord;
use ferrodock_infra::deploy_log::DeployLog;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use self::artifact::{Job, Prepared};
pub use self::context::{DeployContext, KEPT_IMAGES, Names};
use crate::jobs::JobHandler;

pub use ferrodock_db::deployments::DEPLOYMENT_JOB;

#[derive(Debug, Clone)]
pub struct DeployHandler {
    ctx: Arc<DeployContext>,
}

fn deployment_id(job: &JobRecord) -> AppResult<Id> {
    job.payload
        .get("deployment_id")
        .and_then(|value| value.as_str())
        .and_then(|value| Uuid::parse_str(value).ok())
        .map(Id::from)
        .ok_or_else(|| AppError::validation("JOB_PAYLOAD_INVALID"))
}

impl DeployHandler {
    pub fn new(ctx: DeployContext) -> Self {
        Self { ctx: Arc::new(ctx) }
    }

    async fn deploy(
        &self,
        job: &JobRecord,
        log: &DeployLog,
        cancel: &CancellationToken,
    ) -> AppResult<()> {
        let ctx = &self.ctx;
        let not_found = || AppError::not_found("DEPLOYMENT_NOT_FOUND");
        let deployment = ctx
            .deployments
            .find(deployment_id(job)?)
            .await?
            .ok_or_else(not_found)?;
        let organization_id = job.organization_id.ok_or_else(not_found)?;
        let application = ctx
            .applications
            .find(organization_id, deployment.application_id)
            .await?
            .ok_or_else(|| AppError::not_found("APPLICATION_NOT_FOUND"))?;
        let names = Names::new(application.id, deployment.id, &application.slug);
        let variables = ctx.variables(application.id).await?;
        let run = Job {
            ctx,
            application: &application,
            deployment: &deployment,
            names: &names,
            variables: &variables,
            log,
            cancel,
        };
        let artifact = match run.prepare().await? {
            Prepared::Compose(artifact) => artifact,
            Prepared::Image(artifact) => {
                run.release(&artifact.image_id).await?;
                artifact
            }
        };
        let current = artifact.image_ref.clone();
        ctx.deployments
            .record_artifact(deployment.id, artifact)
            .await?;
        self.prune(application.id, &current).await;
        Ok(())
    }

    async fn prune(&self, application_id: Id, current: &str) {
        let Ok(successful) = self.ctx.deployments.successful(application_id).await else {
            return;
        };
        let references: Vec<String> = successful.into_iter().filter_map(|d| d.image_ref).collect();
        let kept: Vec<&str> = std::iter::once(current)
            .chain(references.iter().map(String::as_str))
            .take(KEPT_IMAGES)
            .collect();
        for reference in references
            .iter()
            .filter(|r| r.starts_with("fd/") && !kept.contains(&r.as_str()))
        {
            self.ctx.docker.remove_image(reference).await;
        }
    }
}

fn describe(error: &AppError) -> String {
    match error.params().get("exit_code") {
        Some(code) => format!("Failed: {} (exit code {code})", error.code()),
        None => format!("Failed: {}", error.code()),
    }
}

#[async_trait]
impl JobHandler for DeployHandler {
    fn kind(&self) -> &'static str {
        DEPLOYMENT_JOB
    }

    async fn run(&self, job: &JobRecord, cancel: CancellationToken) -> AppResult<()> {
        let log = self.ctx.logs.open(deployment_id(job)?).await?;
        let result = self.deploy(job, &log, &cancel).await;
        match &result {
            Ok(()) => log.step("Deployment finished").await,
            Err(_) if cancel.is_cancelled() => log.step("Deployment cancelled").await,
            Err(error) => log.step(&describe(error)).await,
        }
        log.flush().await;
        result
    }
}
