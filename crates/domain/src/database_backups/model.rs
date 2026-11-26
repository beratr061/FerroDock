use ferrodock_core::Id;
use ferrodock_db::backups::{DestinationRecord, PolicyRecord, RunRecord};
use time::OffsetDateTime;
use zeroize::Zeroizing;

#[derive(Debug, Clone)]
pub struct S3Details {
    pub endpoint: Option<String>,
    pub region: String,
    pub bucket: String,
    pub access_key_id: String,
    pub path_style: bool,
}

#[derive(Debug, Clone)]
pub struct Destination {
    pub id: Id,
    pub name: String,
    pub path_prefix: String,
    pub s3: Option<S3Details>,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone)]
pub struct S3Input {
    pub endpoint: Option<String>,
    pub region: String,
    pub bucket: String,
    pub access_key_id: String,
    pub secret_access_key: Zeroizing<String>,
    pub path_style: bool,
}

#[derive(Debug, Clone)]
pub struct NewDestinationInput {
    pub name: String,
    pub path_prefix: Option<String>,
    pub s3: Option<S3Input>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Policy {
    pub destination_id: Id,
    pub schedule: String,
    pub retention: u16,
    pub enabled: bool,
    pub next_run_at: OffsetDateTime,
}

#[derive(Debug, Clone)]
pub struct PolicyInput {
    pub destination_id: Id,
    pub schedule: String,
    pub retention: u16,
    pub enabled: bool,
}

#[derive(Debug, Clone)]
pub struct BackupRun {
    pub id: Id,
    pub destination_id: Option<Id>,
    pub operation: String,
    pub trigger: String,
    pub source_run_id: Option<Id>,
    pub status: String,
    pub size_bytes: Option<i64>,
    pub error_code: Option<String>,
    pub created_at: OffsetDateTime,
    pub finished_at: Option<OffsetDateTime>,
}

impl From<DestinationRecord> for Destination {
    fn from(record: DestinationRecord) -> Self {
        Self {
            id: record.id,
            name: record.name,
            path_prefix: record.path_prefix,
            s3: record.s3.map(|s3| S3Details {
                endpoint: s3.endpoint,
                region: s3.region,
                bucket: s3.bucket,
                access_key_id: s3.access_key_id,
                path_style: s3.path_style,
            }),
            created_at: record.created_at,
        }
    }
}

impl From<PolicyRecord> for Policy {
    fn from(record: PolicyRecord) -> Self {
        Self {
            destination_id: record.destination_id,
            schedule: record.schedule,
            retention: record.retention,
            enabled: record.enabled,
            next_run_at: record.next_run_at,
        }
    }
}

impl From<RunRecord> for BackupRun {
    fn from(record: RunRecord) -> Self {
        Self {
            id: record.id,
            destination_id: record.destination_id,
            operation: record.operation,
            trigger: record.trigger,
            source_run_id: record.source_run_id,
            status: record.status,
            size_bytes: record.size_bytes,
            error_code: record.error_code,
            created_at: record.created_at,
            finished_at: record.finished_at,
        }
    }
}
