mod build;
mod containers;
mod events;
mod exec;
mod image;
mod images;
mod logs;
mod naming;
mod readiness;
mod workloads;

use bollard::Docker;
use ferrodock_core::{AppError, AppResult, ErrorKind};
use serde::Serialize;

pub use build::{ComposeUp, ImageBuild, build_image, compose_down, compose_up};
pub use containers::{ContainerAction, ContainerInfo};
pub use events::{ContainerEvent, EventStream};
pub use exec::{ShellInput, ShellOutput, ShellSession, ShellSize, ShellTarget};
pub use image::validate_image;
pub use images::WORKLOAD_NETWORK;
pub use logs::{LogLine, LogQuery, LogStream};
pub use naming::{application_key, compose_project, network_alias, short_key};
pub use workloads::{APPLICATION_LABEL, DEPLOYMENT_LABEL, WorkloadContainer, WorkloadSpec};

const MAX_REFERENCE_LEN: usize = 128;
pub const MANAGED_LABEL: &str = "ferrodock.managed";
pub const CONTROL_PLANE_PROJECT: &str = "ferrodock";
const COMPOSE_PROJECT_LABEL: &str = "com.docker.compose.project";

#[derive(Debug, Clone)]
pub struct DockerEngine {
    client: Docker,
}

#[derive(Debug, Clone, Serialize)]
pub struct EngineInfo {
    pub version: String,
    pub api_version: String,
    pub os: String,
    pub arch: String,
}

impl DockerEngine {
    pub fn connect_local() -> AppResult<Self> {
        let client = Docker::connect_with_local_defaults().map_err(docker_error)?;
        Ok(Self { client })
    }

    pub async fn info(&self) -> AppResult<EngineInfo> {
        let version = self.client.version().await.map_err(docker_error)?;
        Ok(EngineInfo {
            version: version.version.unwrap_or_default(),
            api_version: version.api_version.unwrap_or_default(),
            os: version.os.unwrap_or_default(),
            arch: version.arch.unwrap_or_default(),
        })
    }
}

pub(crate) fn docker_error(error: bollard::errors::Error) -> AppError {
    match &error {
        bollard::errors::Error::DockerResponseServerError {
            status_code: 404, ..
        } => AppError::not_found("CONTAINER_NOT_FOUND").with_source(error),
        bollard::errors::Error::DockerResponseServerError {
            status_code: 409, ..
        } => AppError::conflict("CONTAINER_STATE_CONFLICT").with_source(error),
        _ => AppError::new(ErrorKind::Unavailable, "DOCKER_UNAVAILABLE").with_source(error),
    }
}

pub(crate) fn is_not_modified(error: &bollard::errors::Error) -> bool {
    matches!(
        error,
        bollard::errors::Error::DockerResponseServerError {
            status_code: 304,
            ..
        }
    )
}

pub fn validate_reference(reference: &str) -> AppResult<&str> {
    let valid = !reference.is_empty()
        && reference.len() <= MAX_REFERENCE_LEN
        && reference.starts_with(|c: char| c.is_ascii_alphanumeric())
        && reference
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '-'));
    if valid {
        Ok(reference)
    } else {
        Err(AppError::validation("CONTAINER_REFERENCE_INVALID"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_ids_and_names_only() {
        assert!(validate_reference("3f2a9c1b7d4e").is_ok());
        assert!(validate_reference("ferrodock-postgres-1").is_ok());
        assert!(validate_reference("../containers").is_err());
        assert!(validate_reference("name/with/slash").is_err());
        assert!(validate_reference("-leading-dash").is_err());
        assert!(validate_reference("").is_err());
        assert!(validate_reference(&"a".repeat(129)).is_err());
    }
}
