use async_trait::async_trait;
use ferrodock_core::{AppResult, Id};
use time::OffsetDateTime;

#[derive(Debug, Clone)]
pub struct WebhookRecord {
    pub application_id: Id,
    pub secret_enc: String,
    pub created_at: OffsetDateTime,
    pub last_delivery_at: Option<OffsetDateTime>,
    pub last_outcome: Option<String>,
}

#[derive(Debug, Clone)]
pub struct WebhookTarget {
    pub organization_id: Id,
    pub application_id: Id,
    pub secret_enc: String,
    pub git_branch: Option<String>,
}

#[async_trait]
pub trait WebhookRepo: Send + Sync + 'static {
    async fn find(&self, application_id: Id) -> AppResult<Option<WebhookRecord>>;
    async fn target(&self, application_id: Id) -> AppResult<Option<WebhookTarget>>;
    async fn save(&self, application_id: Id, secret_enc: &str) -> AppResult<WebhookRecord>;
    async fn remove(&self, application_id: Id) -> AppResult<bool>;
    async fn record_delivery(&self, application_id: Id, outcome: &str) -> AppResult<()>;
}
