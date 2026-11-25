use async_trait::async_trait;
use ferrodock_core::{AppResult, Id};
use serde_json::Value;
use time::OffsetDateTime;

pub const LOCAL_QUEUE: &str = "server:local";
pub const PANEL_BACKUP_JOB: &str = "panel.backup";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobStatus {
    Queued,
    Running,
    Succeeded,
    Failed,
    Cancelled,
}

impl JobStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }

    pub fn parse(value: &str) -> Self {
        match value {
            "queued" => Self::Queued,
            "running" => Self::Running,
            "succeeded" => Self::Succeeded,
            "cancelled" => Self::Cancelled,
            _ => Self::Failed,
        }
    }
}

#[derive(Debug, Clone)]
pub struct NewJob {
    pub organization_id: Option<Id>,
    pub kind: String,
    pub queue: String,
    pub group_key: Option<String>,
    pub payload: Value,
}

#[derive(Debug, Clone)]
pub struct JobRecord {
    pub id: Id,
    pub organization_id: Option<Id>,
    pub kind: String,
    pub queue: String,
    pub group_key: Option<String>,
    pub payload: Value,
    pub status: JobStatus,
    pub attempts: i32,
    pub error_code: Option<String>,
    pub created_at: OffsetDateTime,
    pub started_at: Option<OffsetDateTime>,
    pub finished_at: Option<OffsetDateTime>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JobOutcome {
    Succeeded,
    Failed { code: String },
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CancelResult {
    Cancelled,
    Requested,
    NotCancellable,
}

#[async_trait]
pub trait JobQueue: Send + Sync + 'static {
    async fn enqueue(&self, job: NewJob) -> AppResult<JobRecord>;
    async fn claim(
        &self,
        queue: &str,
        worker: &str,
        concurrency: i64,
    ) -> AppResult<Option<JobRecord>>;
    async fn complete(&self, id: Id, outcome: JobOutcome) -> AppResult<()>;
    async fn request_cancel(&self, id: Id) -> AppResult<CancelResult>;
    async fn cancel_requested(&self, id: Id) -> AppResult<bool>;
    async fn fail_interrupted(&self, queue: &str) -> AppResult<u64>;
    async fn find(&self, id: Id) -> AppResult<Option<JobRecord>>;
}
