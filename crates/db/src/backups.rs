use async_trait::async_trait;
use ferrodock_core::{AppResult, Id};
use time::OffsetDateTime;

pub const BACKUP_RUN_JOB: &str = "backup.run";
pub const BACKUP_RESTORE_JOB: &str = "backup.restore";

#[derive(Debug, Clone)]
pub struct S3Settings {
    pub endpoint: Option<String>,
    pub region: String,
    pub bucket: String,
    pub access_key_id: String,
    pub secret_access_key_enc: String,
    pub path_style: bool,
}

#[derive(Debug, Clone)]
pub struct DestinationRecord {
    pub id: Id,
    pub organization_id: Id,
    pub name: String,
    pub path_prefix: String,
    pub s3: Option<S3Settings>,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BackupTarget {
    Database(Id),
    Volume { application_id: Id, volume: String },
}

impl BackupTarget {
    pub fn columns(&self) -> (Option<uuid::Uuid>, Option<uuid::Uuid>, Option<&str>) {
        match self {
            Self::Database(id) => (Some(id.as_uuid()), None, None),
            Self::Volume {
                application_id,
                volume,
            } => (None, Some(application_id.as_uuid()), Some(volume.as_str())),
        }
    }

    pub fn from_columns(
        database_id: Option<uuid::Uuid>,
        application_id: Option<uuid::Uuid>,
        volume: Option<String>,
    ) -> Option<Self> {
        match (database_id, application_id, volume) {
            (Some(id), None, None) => Some(Self::Database(id.into())),
            (None, Some(application_id), Some(volume)) => Some(Self::Volume {
                application_id: application_id.into(),
                volume,
            }),
            _ => None,
        }
    }

    pub fn group_key(&self) -> String {
        match self {
            Self::Database(id) => format!("database:{id}"),
            Self::Volume { application_id, .. } => application_id.to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PolicyRecord {
    pub target: BackupTarget,
    pub destination_id: Id,
    pub schedule: String,
    pub retention: u16,
    pub enabled: bool,
    pub next_run_at: OffsetDateTime,
}

#[derive(Debug, Clone)]
pub struct DuePolicy {
    pub policy: PolicyRecord,
    pub organization_id: Id,
}

#[derive(Debug, Clone)]
pub struct RunRecord {
    pub id: Id,
    pub target: BackupTarget,
    pub destination_id: Option<Id>,
    pub operation: String,
    pub trigger: String,
    pub source_run_id: Option<Id>,
    pub status: String,
    pub object_key: Option<String>,
    pub size_bytes: Option<i64>,
    pub error_code: Option<String>,
    pub created_at: OffsetDateTime,
    pub finished_at: Option<OffsetDateTime>,
}

#[derive(Debug, Clone)]
pub struct NewRun {
    pub id: Id,
    pub target: BackupTarget,
    pub destination_id: Id,
    pub operation: &'static str,
    pub trigger: &'static str,
    pub source_run_id: Option<Id>,
    pub created_by: Option<Id>,
}

#[derive(Debug, Clone)]
pub struct RunOutcome {
    pub status: &'static str,
    pub object_key: Option<String>,
    pub size_bytes: Option<i64>,
    pub error_code: Option<String>,
}

#[async_trait]
pub trait BackupDestinationRepo: Send + Sync + 'static {
    async fn create_destination(&self, destination: DestinationRecord) -> AppResult<()>;
    async fn destinations(&self, organization_id: Id) -> AppResult<Vec<DestinationRecord>>;
    async fn destination(
        &self,
        organization_id: Id,
        id: Id,
    ) -> AppResult<Option<DestinationRecord>>;
    async fn destination_in_use(&self, id: Id) -> AppResult<bool>;
    async fn delete_destination(&self, organization_id: Id, id: Id) -> AppResult<bool>;
}

#[async_trait]
pub trait BackupPolicyRepo: Send + Sync + 'static {
    async fn policy(&self, target: &BackupTarget) -> AppResult<Option<PolicyRecord>>;
    async fn save_policy(&self, policy: PolicyRecord) -> AppResult<()>;
    async fn remove_policy(&self, target: &BackupTarget) -> AppResult<()>;
    async fn due_policies(&self, now: OffsetDateTime) -> AppResult<Vec<DuePolicy>>;
    async fn advance_policy(
        &self,
        target: &BackupTarget,
        expected: OffsetDateTime,
        next: OffsetDateTime,
    ) -> AppResult<bool>;
}

#[async_trait]
pub trait BackupRunRepo: Send + Sync + 'static {
    async fn create_run(&self, run: NewRun) -> AppResult<()>;
    async fn runs(&self, target: &BackupTarget, limit: i64) -> AppResult<Vec<RunRecord>>;
    async fn run(&self, id: Id) -> AppResult<Option<RunRecord>>;
    async fn start_run(&self, id: Id) -> AppResult<()>;
    async fn finish_run(&self, id: Id, outcome: RunOutcome) -> AppResult<()>;
    async fn stored_backups(
        &self,
        target: &BackupTarget,
        destination_id: Id,
    ) -> AppResult<Vec<RunRecord>>;
    async fn delete_run(&self, id: Id) -> AppResult<()>;
}
