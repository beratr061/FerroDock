use ferrodock_domain::scheduled_backups::{
    BackupRun, Destination, NewDestinationInput, Policy, PolicyInput, S3Input,
};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use utoipa::ToSchema;
use uuid::Uuid;

use super::Password;

#[derive(Debug, Clone, Copy, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum DestinationKindDto {
    Local,
    S3,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BackupDestinationDto {
    pub id: Uuid,
    pub name: String,
    pub kind: DestinationKindDto,
    pub path_prefix: String,
    pub endpoint: Option<String>,
    pub region: Option<String>,
    pub bucket: Option<String>,
    pub access_key_id: Option<String>,
    pub path_style: bool,
    #[serde(with = "time::serde::rfc3339")]
    #[schema(value_type = String, format = DateTime)]
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct S3DestinationRequest {
    pub endpoint: Option<String>,
    pub region: String,
    pub bucket: String,
    pub access_key_id: String,
    #[schema(value_type = String, format = Password)]
    pub secret_access_key: Password,
    #[serde(default)]
    pub path_style: bool,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateBackupDestinationRequest {
    pub name: String,
    pub path_prefix: Option<String>,
    pub s3: Option<S3DestinationRequest>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BackupPolicyDto {
    pub destination_id: Uuid,
    pub schedule: String,
    pub retention: u16,
    pub enabled: bool,
    #[serde(with = "time::serde::rfc3339")]
    #[schema(value_type = String, format = DateTime)]
    pub next_run_at: OffsetDateTime,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct BackupPolicyRequest {
    pub destination_id: Uuid,
    pub schedule: String,
    pub retention: u16,
    pub enabled: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BackupRunDto {
    pub id: Uuid,
    pub destination_id: Option<Uuid>,
    #[schema(example = "backup")]
    pub operation: String,
    #[schema(example = "manual")]
    pub trigger: String,
    pub source_run_id: Option<Uuid>,
    #[schema(example = "succeeded")]
    pub status: String,
    pub size_bytes: Option<i64>,
    pub error_code: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    #[schema(value_type = String, format = DateTime)]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339::option")]
    #[schema(value_type = Option<String>, format = DateTime)]
    pub finished_at: Option<OffsetDateTime>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateBackupRequest {
    pub destination_id: Uuid,
}

impl From<Destination> for BackupDestinationDto {
    fn from(destination: Destination) -> Self {
        let s3 = destination.s3.as_ref();
        Self {
            id: destination.id.as_uuid(),
            name: destination.name.clone(),
            kind: if s3.is_some() {
                DestinationKindDto::S3
            } else {
                DestinationKindDto::Local
            },
            path_prefix: destination.path_prefix.clone(),
            endpoint: s3.and_then(|s3| s3.endpoint.clone()),
            region: s3.map(|s3| s3.region.clone()),
            bucket: s3.map(|s3| s3.bucket.clone()),
            access_key_id: s3.map(|s3| s3.access_key_id.clone()),
            path_style: s3.is_some_and(|s3| s3.path_style),
            created_at: destination.created_at,
        }
    }
}

impl From<CreateBackupDestinationRequest> for NewDestinationInput {
    fn from(request: CreateBackupDestinationRequest) -> Self {
        Self {
            name: request.name,
            path_prefix: request.path_prefix,
            s3: request.s3.map(|s3| S3Input {
                endpoint: s3.endpoint,
                region: s3.region,
                bucket: s3.bucket,
                access_key_id: s3.access_key_id,
                secret_access_key: s3.secret_access_key.into_inner(),
                path_style: s3.path_style,
            }),
        }
    }
}

impl From<Policy> for BackupPolicyDto {
    fn from(policy: Policy) -> Self {
        Self {
            destination_id: policy.destination_id.as_uuid(),
            schedule: policy.schedule,
            retention: policy.retention,
            enabled: policy.enabled,
            next_run_at: policy.next_run_at,
        }
    }
}

impl From<BackupPolicyRequest> for PolicyInput {
    fn from(request: BackupPolicyRequest) -> Self {
        Self {
            destination_id: request.destination_id.into(),
            schedule: request.schedule,
            retention: request.retention,
            enabled: request.enabled,
        }
    }
}

impl From<BackupRun> for BackupRunDto {
    fn from(run: BackupRun) -> Self {
        Self {
            id: run.id.as_uuid(),
            destination_id: run.destination_id.map(|id| id.as_uuid()),
            operation: run.operation,
            trigger: run.trigger,
            source_run_id: run.source_run_id.map(|id| id.as_uuid()),
            status: run.status,
            size_bytes: run.size_bytes,
            error_code: run.error_code,
            created_at: run.created_at,
            finished_at: run.finished_at,
        }
    }
}
