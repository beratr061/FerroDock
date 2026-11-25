use async_trait::async_trait;
use ferrodock_core::{AppResult, Id};
use time::OffsetDateTime;
use uuid::Uuid;

use super::{PgStore, db_error};
use crate::ApiKeyRepo;
use crate::records::{ApiKeyRecord, NewApiKey};

#[async_trait]
impl ApiKeyRepo for PgStore {
    async fn create(&self, key: NewApiKey) -> AppResult<ApiKeyRecord> {
        let row = sqlx::query!(
            "INSERT INTO api_keys
             (id, user_id, organization_id, name, prefix, key_hash, scopes, expires_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING created_at",
            key.id.as_uuid(),
            key.user_id.as_uuid(),
            key.organization_id.as_uuid(),
            key.name,
            key.prefix,
            key.key_hash,
            &key.scopes,
            key.expires_at
        )
        .fetch_one(&self.pool)
        .await
        .map_err(db_error)?;
        Ok(ApiKeyRecord {
            id: key.id,
            user_id: key.user_id,
            organization_id: key.organization_id,
            name: key.name,
            prefix: key.prefix,
            scopes: key.scopes,
            last_used_at: None,
            expires_at: key.expires_at,
            created_at: row.created_at,
        })
    }

    async fn list(&self, user_id: Id, organization_id: Id) -> AppResult<Vec<ApiKeyRecord>> {
        sqlx::query_as!(
            ApiKeyRow,
            "SELECT id, user_id, organization_id, name, prefix, scopes, last_used_at,
             expires_at, created_at FROM api_keys
             WHERE user_id = $1 AND organization_id = $2 AND revoked_at IS NULL
             ORDER BY created_at DESC",
            user_id.as_uuid(),
            organization_id.as_uuid()
        )
        .fetch_all(&self.pool)
        .await
        .map(|rows| rows.into_iter().map(ApiKeyRecord::from).collect())
        .map_err(db_error)
    }

    async fn find_active(
        &self,
        key_hash: &[u8],
        now: OffsetDateTime,
    ) -> AppResult<Option<ApiKeyRecord>> {
        sqlx::query_as!(
            ApiKeyRow,
            "SELECT id, user_id, organization_id, name, prefix, scopes, last_used_at,
             expires_at, created_at FROM api_keys
             WHERE key_hash = $1 AND revoked_at IS NULL
             AND (expires_at IS NULL OR expires_at > $2)",
            key_hash,
            now
        )
        .fetch_optional(&self.pool)
        .await
        .map(|row| row.map(ApiKeyRecord::from))
        .map_err(db_error)
    }

    async fn revoke(&self, id: Id, user_id: Id) -> AppResult<bool> {
        sqlx::query!(
            "UPDATE api_keys SET revoked_at = now()
             WHERE id = $1 AND user_id = $2 AND revoked_at IS NULL",
            id.as_uuid(),
            user_id.as_uuid()
        )
        .execute(&self.pool)
        .await
        .map(|result| result.rows_affected() > 0)
        .map_err(db_error)
    }

    async fn mark_used(&self, id: Id, now: OffsetDateTime) -> AppResult<()> {
        sqlx::query!(
            "UPDATE api_keys SET last_used_at = $2 WHERE id = $1",
            id.as_uuid(),
            now
        )
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(db_error)
    }
}

struct ApiKeyRow {
    id: Uuid,
    user_id: Uuid,
    organization_id: Uuid,
    name: String,
    prefix: String,
    scopes: Vec<String>,
    last_used_at: Option<OffsetDateTime>,
    expires_at: Option<OffsetDateTime>,
    created_at: OffsetDateTime,
}

impl From<ApiKeyRow> for ApiKeyRecord {
    fn from(row: ApiKeyRow) -> Self {
        Self {
            id: row.id.into(),
            user_id: row.user_id.into(),
            organization_id: row.organization_id.into(),
            name: row.name,
            prefix: row.prefix,
            scopes: row.scopes,
            last_used_at: row.last_used_at,
            expires_at: row.expires_at,
            created_at: row.created_at,
        }
    }
}
