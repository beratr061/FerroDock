use async_trait::async_trait;
use ferrodock_core::{AppResult, Id};
use time::OffsetDateTime;
use uuid::Uuid;

use super::{PgStore, db_error};
use crate::backups::{BackupDestinationRepo, DestinationRecord, S3Settings};

struct DestinationRow {
    id: Uuid,
    organization_id: Uuid,
    name: String,
    kind: String,
    endpoint: Option<String>,
    region: Option<String>,
    bucket: Option<String>,
    path_prefix: String,
    access_key_id: Option<String>,
    secret_access_key_enc: Option<String>,
    path_style: bool,
    created_at: OffsetDateTime,
}

impl From<DestinationRow> for DestinationRecord {
    fn from(row: DestinationRow) -> Self {
        let s3 = match (row.kind.as_str(), row.region, row.bucket) {
            ("s3", Some(region), Some(bucket)) => Some(S3Settings {
                endpoint: row.endpoint,
                region,
                bucket,
                access_key_id: row.access_key_id.unwrap_or_default(),
                secret_access_key_enc: row.secret_access_key_enc.unwrap_or_default(),
                path_style: row.path_style,
            }),
            _ => None,
        };
        Self {
            id: row.id.into(),
            organization_id: row.organization_id.into(),
            name: row.name,
            path_prefix: row.path_prefix,
            s3,
            created_at: row.created_at,
        }
    }
}

#[async_trait]
impl BackupDestinationRepo for PgStore {
    async fn create_destination(&self, destination: DestinationRecord) -> AppResult<()> {
        let s3 = destination.s3.as_ref();
        sqlx::query!(
            "INSERT INTO backup_destinations (id, organization_id, name, kind, endpoint, region,
             bucket, path_prefix, access_key_id, secret_access_key_enc, path_style)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)",
            destination.id.as_uuid(),
            destination.organization_id.as_uuid(),
            destination.name,
            if s3.is_some() { "s3" } else { "local" },
            s3.and_then(|s3| s3.endpoint.as_deref()),
            s3.map(|s3| s3.region.as_str()),
            s3.map(|s3| s3.bucket.as_str()),
            destination.path_prefix,
            s3.map(|s3| s3.access_key_id.as_str()),
            s3.map(|s3| s3.secret_access_key_enc.as_str()),
            s3.is_some_and(|s3| s3.path_style)
        )
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(db_error)
    }

    async fn destinations(&self, organization_id: Id) -> AppResult<Vec<DestinationRecord>> {
        let rows = sqlx::query_as!(
            DestinationRow,
            "SELECT id, organization_id, name, kind, endpoint, region, bucket, path_prefix,
             access_key_id, secret_access_key_enc, path_style, created_at
             FROM backup_destinations WHERE organization_id = $1 ORDER BY name",
            organization_id.as_uuid()
        )
        .fetch_all(&self.pool)
        .await
        .map_err(db_error)?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn destination(
        &self,
        organization_id: Id,
        id: Id,
    ) -> AppResult<Option<DestinationRecord>> {
        let row = sqlx::query_as!(
            DestinationRow,
            "SELECT id, organization_id, name, kind, endpoint, region, bucket, path_prefix,
             access_key_id, secret_access_key_enc, path_style, created_at
             FROM backup_destinations WHERE organization_id = $1 AND id = $2",
            organization_id.as_uuid(),
            id.as_uuid()
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(db_error)?;
        Ok(row.map(Into::into))
    }

    async fn destination_in_use(&self, id: Id) -> AppResult<bool> {
        sqlx::query_scalar!(
            "SELECT EXISTS (SELECT 1 FROM backup_policies WHERE destination_id = $1) AS \"used!\"",
            id.as_uuid()
        )
        .fetch_one(&self.pool)
        .await
        .map_err(db_error)
    }

    async fn delete_destination(&self, organization_id: Id, id: Id) -> AppResult<bool> {
        let deleted = sqlx::query!(
            "DELETE FROM backup_destinations WHERE organization_id = $1 AND id = $2",
            organization_id.as_uuid(),
            id.as_uuid()
        )
        .execute(&self.pool)
        .await
        .map_err(db_error)?;
        Ok(deleted.rows_affected() > 0)
    }
}
