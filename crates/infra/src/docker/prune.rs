use std::collections::HashMap;

use bollard::query_parameters::{
    ListImagesOptionsBuilder, PruneBuildOptionsBuilder, PruneImagesOptionsBuilder,
};
use ferrodock_core::AppResult;

use super::workloads::APPLICATION_LABEL;
use super::{DockerEngine, docker_error};

const BUILT_IMAGE_PREFIX: &str = "fd/";
const BUILD_CACHE_MAX_AGE: &str = "168h";

impl DockerEngine {
    pub async fn built_images(&self, application_id: &str) -> AppResult<Vec<String>> {
        let filters = HashMap::from([(
            "label",
            vec![format!("{APPLICATION_LABEL}={application_id}")],
        )]);
        let options = ListImagesOptionsBuilder::new().filters(&filters).build();
        let images = self
            .client
            .list_images(Some(options))
            .await
            .map_err(docker_error)?;
        Ok(images
            .into_iter()
            .flat_map(|image| image.repo_tags)
            .filter(|tag| tag.starts_with(BUILT_IMAGE_PREFIX))
            .collect())
    }

    pub async fn prune_unused(&self) -> i64 {
        let dangling = HashMap::from([("dangling", vec!["true"])]);
        let images = self
            .client
            .prune_images(Some(
                PruneImagesOptionsBuilder::new().filters(&dangling).build(),
            ))
            .await
            .map(|response| response.space_reclaimed.unwrap_or(0));
        let stale = HashMap::from([("until", vec![BUILD_CACHE_MAX_AGE])]);
        let cache = self
            .client
            .prune_build(Some(
                PruneBuildOptionsBuilder::new().filters(&stale).build(),
            ))
            .await
            .map(|response| response.space_reclaimed.unwrap_or(0));
        [images, cache]
            .into_iter()
            .map(|result| {
                result.unwrap_or_else(|error| {
                    tracing::debug!(%error, "docker prune skipped");
                    0
                })
            })
            .sum()
    }
}
