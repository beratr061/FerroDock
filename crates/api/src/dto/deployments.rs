use ferrodock_domain::deployments::{Deployment, DeploymentKind, DeploymentStatus, LogChunk};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum DeploymentStatusDto {
    Queued,
    Running,
    Succeeded,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Copy, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum DeploymentKindDto {
    Deploy,
    Rollback,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DeploymentDto {
    pub id: Uuid,
    pub application_id: Uuid,
    pub kind: DeploymentKindDto,
    pub status: DeploymentStatusDto,
    pub error_code: Option<String>,
    pub rollback_of: Option<Uuid>,
    pub commit_sha: Option<String>,
    pub image_ref: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    #[schema(value_type = String, format = DateTime)]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339::option")]
    #[schema(value_type = Option<String>, format = DateTime)]
    pub started_at: Option<OffsetDateTime>,
    #[serde(with = "time::serde::rfc3339::option")]
    #[schema(value_type = Option<String>, format = DateTime)]
    pub finished_at: Option<OffsetDateTime>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DeploymentLogDto {
    pub text: String,
    pub next_offset: u64,
    pub status: DeploymentStatusDto,
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct DeploymentLogQuery {
    pub offset: Option<u64>,
}

impl From<DeploymentStatus> for DeploymentStatusDto {
    fn from(status: DeploymentStatus) -> Self {
        match status {
            DeploymentStatus::Queued => Self::Queued,
            DeploymentStatus::Running => Self::Running,
            DeploymentStatus::Succeeded => Self::Succeeded,
            DeploymentStatus::Failed => Self::Failed,
            DeploymentStatus::Cancelled => Self::Cancelled,
        }
    }
}

impl From<Deployment> for DeploymentDto {
    fn from(deployment: Deployment) -> Self {
        Self {
            id: deployment.id.as_uuid(),
            application_id: deployment.application_id.as_uuid(),
            kind: match deployment.kind {
                DeploymentKind::Deploy => DeploymentKindDto::Deploy,
                DeploymentKind::Rollback => DeploymentKindDto::Rollback,
            },
            status: deployment.status.into(),
            error_code: deployment.error_code,
            rollback_of: deployment.rollback_of.map(|id| id.as_uuid()),
            commit_sha: deployment.commit_sha,
            image_ref: deployment.image_ref,
            created_at: deployment.created_at,
            started_at: deployment.started_at,
            finished_at: deployment.finished_at,
        }
    }
}

impl DeploymentLogDto {
    pub fn new(deployment: &Deployment, chunk: LogChunk) -> Self {
        Self {
            text: chunk.text,
            next_offset: chunk.next_offset,
            status: deployment.status.into(),
        }
    }
}
