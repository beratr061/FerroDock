use std::collections::HashMap;

use bollard::models::{ContainerCreateBody, HostConfig};
use bollard::query_parameters::{
    CreateContainerOptionsBuilder, CreateImageOptionsBuilder, RemoveContainerOptionsBuilder,
};
use ferrodock_core::{AppError, AppResult};
use futures_util::StreamExt;

use super::{DockerEngine, MANAGED_LABEL, docker_error, validate_image};

pub const UPDATER_NAME: &str = "ferrodock-updater";
const DOCKER_SOCKET: &str = "/var/run/docker.sock";

#[derive(Debug, Clone)]
pub struct UpdaterSpec {
    pub image: String,
    pub args: Vec<String>,
    pub config_dir: String,
}

fn updater_body(spec: &UpdaterSpec) -> ContainerCreateBody {
    ContainerCreateBody {
        image: Some(spec.image.clone()),
        cmd: Some(spec.args.clone()),
        user: Some("0:0".to_owned()),
        labels: Some(HashMap::from([(
            MANAGED_LABEL.to_owned(),
            "true".to_owned(),
        )])),
        host_config: Some(HostConfig {
            binds: Some(vec![
                format!("{DOCKER_SOCKET}:{DOCKER_SOCKET}"),
                format!("{0}:{0}", spec.config_dir),
            ]),
            ..HostConfig::default()
        }),
        ..ContainerCreateBody::default()
    }
}

impl DockerEngine {
    pub async fn ensure_image(&self, reference: &str) -> AppResult<()> {
        validate_image(reference)?;
        if self.client.inspect_image(reference).await.is_ok() {
            return Ok(());
        }
        let options = CreateImageOptionsBuilder::new()
            .from_image(reference)
            .build();
        let mut stream = self.client.create_image(Some(options), None, None);
        while let Some(item) = stream.next().await {
            item.map_err(|error| AppError::validation("IMAGE_PULL_FAILED").with_source(error))?;
        }
        Ok(())
    }

    pub async fn container_image(&self, id: &str) -> AppResult<Option<String>> {
        let details = self
            .client
            .inspect_container(id, None)
            .await
            .map_err(docker_error)?;
        Ok(details.config.and_then(|config| config.image))
    }

    pub async fn updater_running(&self) -> AppResult<bool> {
        match self.client.inspect_container(UPDATER_NAME, None).await {
            Ok(details) => Ok(details
                .state
                .and_then(|state| state.running)
                .unwrap_or(false)),
            Err(bollard::errors::Error::DockerResponseServerError {
                status_code: 404, ..
            }) => Ok(false),
            Err(error) => Err(docker_error(error)),
        }
    }

    pub async fn start_updater(&self, spec: &UpdaterSpec) -> AppResult<()> {
        let remove = RemoveContainerOptionsBuilder::new().force(true).build();
        let _ = self
            .client
            .remove_container(UPDATER_NAME, Some(remove))
            .await;
        let options = CreateContainerOptionsBuilder::new()
            .name(UPDATER_NAME)
            .build();
        self.client
            .create_container(Some(options), updater_body(spec))
            .await
            .map_err(docker_error)?;
        self.start_workload(UPDATER_NAME).await
    }
}
