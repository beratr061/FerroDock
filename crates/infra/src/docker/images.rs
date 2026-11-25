use bollard::models::NetworkCreateRequest;
use bollard::query_parameters::{CreateImageOptionsBuilder, RemoveImageOptionsBuilder};
use ferrodock_core::{AppError, AppResult};
use futures_util::StreamExt;
use tokio_util::sync::CancellationToken;

use super::{DockerEngine, docker_error, validate_image};
use crate::deploy_log::DeployLog;
use crate::process::cancelled;

pub const WORKLOAD_NETWORK: &str = "ferrodock-network";

fn with_default_tag(reference: &str) -> String {
    let name = reference.rsplit('/').next().unwrap_or(reference);
    if name.contains(':') || name.contains('@') {
        reference.to_owned()
    } else {
        format!("{reference}:latest")
    }
}

impl DockerEngine {
    pub async fn ensure_network(&self) -> AppResult<()> {
        if self
            .client
            .inspect_network(WORKLOAD_NETWORK, None)
            .await
            .is_ok()
        {
            return Ok(());
        }
        let request = NetworkCreateRequest {
            name: WORKLOAD_NETWORK.to_owned(),
            driver: Some("bridge".to_owned()),
            ..NetworkCreateRequest::default()
        };
        self.client
            .create_network(request)
            .await
            .map(|_| ())
            .map_err(docker_error)
    }

    pub async fn pull_image(
        &self,
        reference: &str,
        log: &DeployLog,
        cancel: &CancellationToken,
    ) -> AppResult<()> {
        validate_image(reference)?;
        let image = with_default_tag(reference);
        log.step(&format!("Pulling {image}")).await;
        let options = CreateImageOptionsBuilder::new().from_image(&image).build();
        let mut stream = self.client.create_image(Some(options), None, None);
        let mut last_status = String::new();
        loop {
            let next = tokio::select! {
                () = cancel.cancelled() => return Err(cancelled()),
                next = stream.next() => next,
            };
            let Some(item) = next else { return Ok(()) };
            let info =
                item.map_err(|error| AppError::validation("IMAGE_PULL_FAILED").with_source(error))?;
            let status = info.status.unwrap_or_default();
            if status != last_status && info.progress_detail.is_none_or(|p| p.total.is_none()) {
                log.line(&status).await;
                last_status = status;
            }
        }
    }

    pub async fn image_id(&self, reference: &str) -> AppResult<String> {
        self.client
            .inspect_image(reference)
            .await
            .map_err(docker_error)?
            .id
            .ok_or_else(|| AppError::validation("IMAGE_NOT_FOUND"))
    }

    pub async fn remove_image(&self, reference: &str) {
        let options = RemoveImageOptionsBuilder::new().noprune(false).build();
        if let Err(error) = self
            .client
            .remove_image(reference, Some(options), None)
            .await
        {
            tracing::debug!(%error, reference, "image not removed");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adds_latest_only_when_untagged() {
        assert_eq!(with_default_tag("nginx"), "nginx:latest");
        assert_eq!(
            with_default_tag("registry:5000/app"),
            "registry:5000/app:latest"
        );
        assert_eq!(with_default_tag("nginx:1.29"), "nginx:1.29");
        assert_eq!(with_default_tag("nginx@sha256:abc"), "nginx@sha256:abc");
    }
}
