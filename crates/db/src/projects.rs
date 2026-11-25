use async_trait::async_trait;
use ferrodock_core::{AppResult, Id};
use time::OffsetDateTime;

#[derive(Debug, Clone)]
pub struct ProjectRecord {
    pub id: Id,
    pub organization_id: Id,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone)]
pub struct EnvironmentRecord {
    pub id: Id,
    pub project_id: Id,
    pub name: String,
    pub slug: String,
    pub created_at: OffsetDateTime,
}

#[derive(Debug)]
pub struct NewProject {
    pub id: Id,
    pub organization_id: Id,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub first_environment: NewEnvironment,
}

#[derive(Debug)]
pub struct NewEnvironment {
    pub id: Id,
    pub project_id: Id,
    pub name: String,
    pub slug: String,
}

#[derive(Debug)]
pub struct ProjectChanges {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnvironmentRemoval {
    Removed,
    NotFound,
    LastRemaining,
}

#[async_trait]
pub trait ProjectRepo: Send + Sync + 'static {
    async fn create(&self, project: NewProject) -> AppResult<(ProjectRecord, EnvironmentRecord)>;
    async fn list(&self, organization_id: Id) -> AppResult<Vec<ProjectRecord>>;
    async fn list_environments(&self, organization_id: Id) -> AppResult<Vec<EnvironmentRecord>>;
    async fn find(&self, organization_id: Id, id: Id) -> AppResult<Option<ProjectRecord>>;
    async fn environments(&self, project_id: Id) -> AppResult<Vec<EnvironmentRecord>>;
    async fn update(
        &self,
        organization_id: Id,
        id: Id,
        changes: ProjectChanges,
    ) -> AppResult<Option<ProjectRecord>>;
    async fn delete(&self, organization_id: Id, id: Id) -> AppResult<bool>;
    async fn create_environment(&self, environment: NewEnvironment)
    -> AppResult<EnvironmentRecord>;
    async fn delete_environment(&self, project_id: Id, id: Id) -> AppResult<EnvironmentRemoval>;
}
