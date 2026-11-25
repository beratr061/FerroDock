use async_trait::async_trait;
use ferrodock_core::{AppError, AppResult, Id};
use time::OffsetDateTime;
use uuid::Uuid;

use super::{PgStore, db_error};
use crate::domains::{DomainRecord, DomainRepo, NewDomain};

struct DomainRow {
    id: Uuid,
    application_id: Uuid,
    host: String,
    path_prefix: String,
    container_port: i32,
    https: bool,
    created_at: OffsetDateTime,
}

impl TryFrom<DomainRow> for DomainRecord {
    type Error = AppError;

    fn try_from(row: DomainRow) -> AppResult<Self> {
        Ok(Self {
            id: row.id.into(),
            application_id: row.application_id.into(),
            host: row.host,
            path_prefix: row.path_prefix,
            container_port: u16::try_from(row.container_port).map_err(AppError::internal)?,
            https: row.https,
            created_at: row.created_at,
        })
    }
}

#[async_trait]
impl DomainRepo for PgStore {
    async fn create(&self, domain: NewDomain) -> AppResult<DomainRecord> {
        let row = sqlx::query_as!(
            DomainRow,
            "INSERT INTO domains (id, application_id, host, path_prefix, container_port, https)
             VALUES ($1, $2, $3, $4, $5, $6)
             RETURNING id, application_id, host, path_prefix, container_port, https, created_at",
            domain.id.as_uuid(),
            domain.application_id.as_uuid(),
            domain.host,
            domain.path_prefix,
            i32::from(domain.container_port),
            domain.https
        )
        .fetch_one(&self.pool)
        .await
        .map_err(db_error)?;
        row.try_into()
    }

    async fn list(&self, application_id: Id) -> AppResult<Vec<DomainRecord>> {
        let rows = sqlx::query_as!(
            DomainRow,
            "SELECT id, application_id, host, path_prefix, container_port, https, created_at
             FROM domains WHERE application_id = $1 ORDER BY host, path_prefix",
            application_id.as_uuid()
        )
        .fetch_all(&self.pool)
        .await
        .map_err(db_error)?;
        rows.into_iter().map(DomainRecord::try_from).collect()
    }

    async fn delete(&self, application_id: Id, id: Id) -> AppResult<bool> {
        sqlx::query!(
            "DELETE FROM domains WHERE application_id = $1 AND id = $2",
            application_id.as_uuid(),
            id.as_uuid()
        )
        .execute(&self.pool)
        .await
        .map(|result| result.rows_affected() > 0)
        .map_err(db_error)
    }
}
