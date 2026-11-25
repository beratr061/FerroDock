use bollard::models::ContainerSummary;
use bollard::query_parameters::{
    ListContainersOptionsBuilder, RestartContainerOptionsBuilder, StopContainerOptionsBuilder,
};
use ferrodock_core::{AppError, AppResult};
use serde::{Deserialize, Serialize};

use super::{
    COMPOSE_PROJECT_LABEL, CONTROL_PLANE_PROJECT, DockerEngine, MANAGED_LABEL, docker_error,
    is_not_modified, validate_reference,
};

const STOP_TIMEOUT_SECONDS: i32 = 10;
const SHORT_ID_LEN: usize = 12;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContainerAction {
    Start,
    Stop,
    Restart,
}

#[derive(Debug, Clone, Serialize)]
pub struct ContainerInfo {
    pub id: String,
    pub short_id: String,
    pub name: String,
    pub image: String,
    pub state: String,
    pub status: String,
    pub created: i64,
    pub managed: bool,
    pub protected: bool,
    pub compose_project: Option<String>,
}

impl From<ContainerSummary> for ContainerInfo {
    fn from(summary: ContainerSummary) -> Self {
        let id = summary.id.unwrap_or_default();
        let labels = summary.labels.unwrap_or_default();
        let name = summary
            .names
            .and_then(|names| names.into_iter().next())
            .map(|name| name.trim_start_matches('/').to_owned())
            .unwrap_or_default();
        Self {
            short_id: id.chars().take(SHORT_ID_LEN).collect(),
            id,
            name,
            image: summary.image.unwrap_or_default(),
            state: summary
                .state
                .map(|state| state.to_string())
                .unwrap_or_default(),
            status: summary.status.unwrap_or_default(),
            created: summary.created.unwrap_or_default(),
            managed: labels
                .get(MANAGED_LABEL)
                .is_some_and(|value| value == "true"),
            protected: labels
                .get(COMPOSE_PROJECT_LABEL)
                .is_some_and(|p| p == CONTROL_PLANE_PROJECT),
            compose_project: labels.get(COMPOSE_PROJECT_LABEL).cloned(),
        }
    }
}

impl DockerEngine {
    pub async fn list_containers(&self, include_stopped: bool) -> AppResult<Vec<ContainerInfo>> {
        let options = ListContainersOptionsBuilder::new()
            .all(include_stopped)
            .build();
        let mut containers: Vec<ContainerInfo> = self
            .client
            .list_containers(Some(options))
            .await
            .map_err(docker_error)?
            .into_iter()
            .map(ContainerInfo::from)
            .collect();
        containers.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(containers)
    }

    pub async fn apply(&self, reference: &str, action: ContainerAction) -> AppResult<()> {
        let reference = validate_reference(reference)?;
        self.ensure_not_protected(reference).await?;
        let result = match action {
            ContainerAction::Start => self.client.start_container(reference, None).await,
            ContainerAction::Stop => {
                let options = StopContainerOptionsBuilder::new()
                    .t(STOP_TIMEOUT_SECONDS)
                    .build();
                self.client.stop_container(reference, Some(options)).await
            }
            ContainerAction::Restart => {
                let options = RestartContainerOptionsBuilder::new()
                    .t(STOP_TIMEOUT_SECONDS)
                    .build();
                self.client
                    .restart_container(reference, Some(options))
                    .await
            }
        };
        match result {
            Err(error) if !is_not_modified(&error) => Err(docker_error(error)),
            _ => Ok(()),
        }
    }

    async fn ensure_not_protected(&self, reference: &str) -> AppResult<()> {
        let details = self
            .client
            .inspect_container(reference, None)
            .await
            .map_err(docker_error)?;
        let project = details
            .config
            .and_then(|config| config.labels)
            .and_then(|labels| labels.get(COMPOSE_PROJECT_LABEL).cloned());
        if project.as_deref() == Some(CONTROL_PLANE_PROJECT) {
            return Err(AppError::forbidden().with_param("reason", "CONTAINER_PROTECTED"));
        }
        Ok(())
    }
}
