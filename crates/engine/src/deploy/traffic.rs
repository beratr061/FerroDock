use ferrodock_core::{AppError, AppResult};
use ferrodock_db::deployments::DeploymentArtifact;
use ferrodock_domain::domains::publish_routes;

use super::artifact::{Job, Prepared};

impl Job<'_> {
    pub async fn deliver(&self) -> AppResult<DeploymentArtifact> {
        match self.prepare().await? {
            Prepared::Compose(artifact) => {
                self.publish_routes(None).await?;
                Ok(artifact)
            }
            Prepared::Image(artifact) => {
                self.release(&artifact.image_id).await?;
                Ok(artifact)
            }
        }
    }

    pub async fn probe_port(&self) -> Option<u16> {
        let domains = self.ctx.domains.list(self.application.id).await.ok()?;
        domains.first().map(|domain| domain.container_port)
    }

    pub async fn publish_routes(&self, active_container: Option<&str>) -> AppResult<()> {
        let mut application = self.application.clone();
        if let Some(container) = active_container {
            application.active_container = Some(container.to_owned());
        }
        let domains = self.ctx.domains.list(application.id).await?;
        if domains.is_empty() {
            return Ok(());
        }
        publish_routes(&self.ctx.proxy, &application, domains)
            .await
            .map_err(|error| AppError::validation("PROXY_WRITE_FAILED").with_source(error))
    }

    pub async fn switch_traffic(&self, container: &str) -> AppResult<()> {
        self.log
            .step(&format!("Routing traffic to {container}"))
            .await;
        let applications = &self.ctx.applications;
        let previous = self.application.active_container.as_deref();
        applications
            .set_active_container(self.application.id, Some(container))
            .await?;
        if let Err(error) = self.publish_routes(Some(container)).await {
            let _ = applications
                .set_active_container(self.application.id, previous)
                .await;
            return Err(error);
        }
        Ok(())
    }
}
