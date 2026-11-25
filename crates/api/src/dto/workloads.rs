use ferrodock_infra::docker::WorkloadContainer;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, ToSchema)]
pub struct WorkloadDto {
    pub id: String,
    pub name: String,
    pub running: bool,
    pub deployment_id: Option<String>,
}

impl From<WorkloadContainer> for WorkloadDto {
    fn from(container: WorkloadContainer) -> Self {
        Self {
            id: container.id,
            name: container.name,
            running: container.running,
            deployment_id: container.deployment_id,
        }
    }
}
