use async_trait::async_trait;
use ferrodock_core::{AppResult, Id};
use time::OffsetDateTime;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DomainRecord {
    pub id: Id,
    pub application_id: Id,
    pub host: String,
    pub path_prefix: String,
    pub container_port: u16,
    pub https: bool,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone)]
pub struct NewDomain {
    pub id: Id,
    pub application_id: Id,
    pub host: String,
    pub path_prefix: String,
    pub container_port: u16,
    pub https: bool,
}

#[async_trait]
pub trait DomainRepo: Send + Sync + 'static {
    async fn create(&self, domain: NewDomain) -> AppResult<DomainRecord>;
    async fn list(&self, application_id: Id) -> AppResult<Vec<DomainRecord>>;
    async fn delete(&self, application_id: Id, id: Id) -> AppResult<bool>;
}
