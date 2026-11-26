use async_trait::async_trait;
use ferrodock_core::{AppResult, Id};
use time::OffsetDateTime;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitSourceRecord {
    pub url: String,
    pub branch: String,
    pub build_kind: String,
    pub build_path: String,
    pub build_context: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SourceRecord {
    Git(GitSourceRecord),
    Image(String),
}

#[derive(Debug, Clone)]
pub struct DeployKeyRecord {
    pub public: String,
    pub private_enc: String,
}

#[derive(Debug, Clone)]
pub struct ApplicationRecord {
    pub id: Id,
    pub environment_id: Id,
    pub project_id: Id,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub source: SourceRecord,
    pub deploy_key: Option<DeployKeyRecord>,
    pub strategy: String,
    pub active_container: Option<String>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone)]
pub struct ApplicationFields {
    pub name: String,
    pub description: String,
    pub source: SourceRecord,
    pub deploy_key: Option<DeployKeyRecord>,
    pub strategy: String,
}

#[derive(Debug)]
pub struct NewApplication {
    pub id: Id,
    pub environment_id: Id,
    pub slug: String,
    pub fields: ApplicationFields,
}

#[async_trait]
pub trait ApplicationRepo: Send + Sync + 'static {
    async fn environment_project(
        &self,
        organization_id: Id,
        environment_id: Id,
    ) -> AppResult<Option<Id>>;
    async fn create(&self, application: NewApplication) -> AppResult<ApplicationRecord>;
    async fn list_for_project(
        &self,
        organization_id: Id,
        project_id: Id,
    ) -> AppResult<Vec<ApplicationRecord>>;
    async fn find(&self, organization_id: Id, id: Id) -> AppResult<Option<ApplicationRecord>>;
    async fn update(&self, id: Id, fields: ApplicationFields) -> AppResult<ApplicationRecord>;
    async fn set_active_container(&self, id: Id, container: Option<&str>) -> AppResult<()>;
    async fn delete(&self, id: Id) -> AppResult<bool>;
}
