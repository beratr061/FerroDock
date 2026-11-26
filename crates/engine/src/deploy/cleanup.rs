use ferrodock_core::{AppResult, Id};
use ferrodock_db::deployments::DeploymentArtifact;

use super::{DeployHandler, KEPT_IMAGES};

impl DeployHandler {
    pub(super) async fn record(
        &self,
        deployment_id: Id,
        delivered: AppResult<DeploymentArtifact>,
    ) -> AppResult<String> {
        let artifact = delivered?;
        let current = artifact.image_ref.clone();
        self.ctx
            .deployments
            .record_artifact(deployment_id, artifact)
            .await?;
        Ok(current)
    }

    pub(super) async fn clean_up(&self, application_id: Id, current: Option<&String>) {
        let docker = &self.ctx.docker;
        let kept = self.kept_images(application_id, current).await;
        let Ok(built) = docker.built_images(&application_id.to_string()).await else {
            return;
        };
        for image in built.iter().filter(|image| !kept.contains(image)) {
            docker.remove_image(image).await;
        }
        let reclaimed = docker.prune_unused().await;
        if reclaimed > 0 {
            tracing::info!(reclaimed, "removed unused images and build cache");
        }
    }

    async fn kept_images(&self, application_id: Id, current: Option<&String>) -> Vec<String> {
        let successful = self
            .ctx
            .deployments
            .successful(application_id)
            .await
            .unwrap_or_default();
        current
            .cloned()
            .into_iter()
            .chain(
                successful
                    .into_iter()
                    .filter_map(|deployment| deployment.image_ref),
            )
            .take(KEPT_IMAGES)
            .collect()
    }
}
