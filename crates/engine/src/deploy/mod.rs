mod artifact;
mod cleanup;
mod context;
mod release;
mod storage;
mod traffic;

use std::sync::Arc;

use async_trait::async_trait;
use ferrodock_core::{AppError, AppResult, Id};

use ferrodock_db::jobs::JobRecord;
use ferrodock_infra::deploy_log::DeployLog;
use ferrodock_infra::realtime::LogSignal;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use self::artifact::Job;
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
        ctx.announce(Some(organization_id), application.id, deployment.id);
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
        let delivered = run.deliver().await;
        let current = self.record(deployment.id, delivered).await;
        self.clean_up(application.id, current.as_ref().ok()).await;
        current.map(|_| ())
    }
}

fn describe(error: &AppError) -> String {
    let details: Vec<String> = error
        .params()
        .iter()
        .filter(|(key, _)| key.as_str() != "detail")
        .map(|(key, value)| match value {
            serde_json::Value::String(text) => format!("{key} {text}"),
            other => format!("{key} {other}"),
        })
        .collect();
    if details.is_empty() {
        format!("Failed: {}", error.code())
    } else {
        format!("Failed: {} ({})", error.code(), details.join(", "))
    }
}

async fn report_failure(log: &DeployLog, error: &AppError) {
    log.step(&describe(error)).await;
    let detail = error
        .params()
        .get("detail")
        .and_then(serde_json::Value::as_str);
    for line in detail.unwrap_or_default().lines() {
        log.line(line).await;
    }
}

#[async_trait]
impl JobHandler for DeployHandler {
    fn kind(&self) -> &'static str {
        DEPLOYMENT_JOB
    }

    async fn run(&self, job: &JobRecord, cancel: CancellationToken) -> AppResult<()> {
        let id = deployment_id(job)?;
        let signal = LogSignal::new(self.ctx.hub.clone(), job.organization_id, id);
        let log = self.ctx.logs.open(id).await?.with_signal(signal);
        let result = self.deploy(job, &log, &cancel).await;
        match &result {
            Ok(()) => log.step("Deployment finished").await,
            Err(_) if cancel.is_cancelled() => log.step("Deployment cancelled").await,
            Err(error) => report_failure(&log, error).await,
        }
        log.flush().await;
        result
    }

    async fn finished(&self, job: &JobRecord) {
        let Ok(id) = deployment_id(job) else { return };
        if let Ok(Some(deployment)) = self.ctx.deployments.find(id).await {
            self.ctx.announce(
                job.organization_id,
                deployment.application_id,
                deployment.id,
            );
        }
    }
}
