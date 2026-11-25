use async_trait::async_trait;
use ferrodock_core::{AppResult, Id};
use time::OffsetDateTime;

#[derive(Debug, Clone)]
pub struct VariableRecord {
    pub key: String,
    pub value_enc: String,
    pub secret: bool,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone)]
pub struct NewVariable {
    pub key: String,
    pub value_enc: String,
    pub secret: bool,
}

#[async_trait]
pub trait VariableRepo: Send + Sync + 'static {
    async fn list(&self, application_id: Id) -> AppResult<Vec<VariableRecord>>;
    async fn replace(&self, application_id: Id, variables: Vec<NewVariable>) -> AppResult<()>;
}
