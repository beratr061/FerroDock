use async_trait::async_trait;
use ferrodock_core::{AppResult, Id};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MountRecord {
    pub kind: String,
    pub source: String,
    pub target: String,
    pub read_only: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortRecord {
    pub published: u16,
    pub target: u16,
    pub protocol: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RuntimeConfigRecord {
    pub mounts: Vec<MountRecord>,
    pub ports: Vec<PortRecord>,
}

#[async_trait]
pub trait RuntimeConfigRepo: Send + Sync + 'static {
    async fn runtime_config(&self, application_id: Id) -> AppResult<RuntimeConfigRecord>;
    async fn replace_runtime_config(
        &self,
        application_id: Id,
        config: RuntimeConfigRecord,
    ) -> AppResult<()>;
}
