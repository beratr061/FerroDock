use ferrodock_domain::auth::{ApiKeySummary, CreatedApiKey};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use utoipa::ToSchema;
use uuid::Uuid;

use super::RoleDto;

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateApiKeyRequest {
    pub name: String,
    pub role: RoleDto,
    pub expires_in_days: Option<u32>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ApiKeyDto {
    pub id: Uuid,
    pub name: String,
    pub prefix: String,
    pub role: RoleDto,
    #[serde(with = "time::serde::rfc3339::option")]
    #[schema(value_type = Option<String>, format = DateTime)]
    pub last_used_at: Option<OffsetDateTime>,
    #[serde(with = "time::serde::rfc3339::option")]
    #[schema(value_type = Option<String>, format = DateTime)]
    pub expires_at: Option<OffsetDateTime>,
    #[serde(with = "time::serde::rfc3339")]
    #[schema(value_type = String, format = DateTime)]
    pub created_at: OffsetDateTime,
}

impl From<ApiKeySummary> for ApiKeyDto {
    fn from(summary: ApiKeySummary) -> Self {
        Self {
            id: summary.id.as_uuid(),
            name: summary.name,
            prefix: summary.prefix,
            role: summary.role.into(),
            last_used_at: summary.last_used_at,
            expires_at: summary.expires_at,
            created_at: summary.created_at,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CreatedApiKeyResponse {
    pub key: String,
    pub api_key: ApiKeyDto,
}

impl From<CreatedApiKey> for CreatedApiKeyResponse {
    fn from(created: CreatedApiKey) -> Self {
        Self {
            key: created.key.to_string(),
            api_key: created.summary.into(),
        }
    }
}
