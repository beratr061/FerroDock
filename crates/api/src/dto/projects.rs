use ferrodock_domain::projects::{Environment, Project};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, ToSchema)]
pub struct EnvironmentDto {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    #[serde(with = "time::serde::rfc3339")]
    #[schema(value_type = String, format = DateTime)]
    pub created_at: OffsetDateTime,
}

impl From<Environment> for EnvironmentDto {
    fn from(environment: Environment) -> Self {
        Self {
            id: environment.id.as_uuid(),
            name: environment.name,
            slug: environment.slug,
            created_at: environment.created_at,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ProjectDto {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub environments: Vec<EnvironmentDto>,
    #[serde(with = "time::serde::rfc3339")]
    #[schema(value_type = String, format = DateTime)]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    #[schema(value_type = String, format = DateTime)]
    pub updated_at: OffsetDateTime,
}

impl From<Project> for ProjectDto {
    fn from(project: Project) -> Self {
        Self {
            id: project.id.as_uuid(),
            name: project.name,
            slug: project.slug,
            description: project.description,
            environments: project.environments.into_iter().map(Into::into).collect(),
            created_at: project.created_at,
            updated_at: project.updated_at,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateProjectRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateProjectRequest {
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateEnvironmentRequest {
    pub name: String,
}
