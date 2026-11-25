use ferrodock_infra::backup_store::BackupFile;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use utoipa::ToSchema;
use uuid::Uuid;

use super::Password;

#[derive(Debug, Serialize, ToSchema)]
pub struct BackupDto {
    pub name: String,
    pub size: u64,
    #[serde(with = "time::serde::rfc3339")]
    #[schema(value_type = String, format = DateTime)]
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BackupRequestedDto {
    pub job_id: Uuid,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct MasterKeyRequest {
    #[schema(value_type = String, format = Password)]
    pub password: Password,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MasterKeyDto {
    pub master_key: String,
}

impl From<BackupFile> for BackupDto {
    fn from(file: BackupFile) -> Self {
        Self {
            name: file.name,
            size: file.size,
            created_at: file.created_at,
        }
    }
}
