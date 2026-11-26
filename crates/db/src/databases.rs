use async_trait::async_trait;
use ferrodock_core::{AppResult, Id};
use time::OffsetDateTime;

pub const DATABASE_DEPLOY_JOB: &str = "database.deploy";

#[derive(Debug, Clone)]
pub struct DatabaseRecord {
    pub id: Id,
    pub environment_id: Id,
    pub project_id: Id,
    pub name: String,
    pub slug: String,
    pub engine: String,
    pub version: String,
    pub database_name: String,
    pub username: String,
    pub password_enc: String,
    pub root_password_enc: Option<String>,
    pub external_port: Option<u16>,
    pub status: String,
    pub error_code: Option<String>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug)]
pub struct NewDatabase {
    pub id: Id,
    pub organization_id: Id,
    pub environment_id: Id,
    pub name: String,
    pub slug: String,
    pub engine: String,
    pub version: String,
    pub database_name: String,
    pub username: String,
    pub password_enc: String,
    pub root_password_enc: Option<String>,
}

#[async_trait]
pub trait DatabaseRepo: Send + Sync + 'static {
    async fn create(&self, database: NewDatabase) -> AppResult<Option<DatabaseRecord>>;
    async fn list_for_project(
        &self,
        organization_id: Id,
        project_id: Id,
    ) -> AppResult<Vec<DatabaseRecord>>;
    async fn find(&self, organization_id: Id, id: Id) -> AppResult<Option<DatabaseRecord>>;
    async fn set_external_port(&self, id: Id, port: Option<u16>) -> AppResult<()>;
    async fn set_status(&self, id: Id, status: &str, error_code: Option<&str>) -> AppResult<()>;
    async fn delete(&self, id: Id) -> AppResult<bool>;
}
