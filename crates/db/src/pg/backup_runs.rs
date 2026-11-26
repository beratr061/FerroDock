use async_trait::async_trait;
use ferrodock_core::{AppResult, Id};
use time::OffsetDateTime;
use uuid::Uuid;

use super::{PgStore, db_error};
use crate::backups::{BackupRunRepo, NewRun, RunOutcome, RunRecord};

struct RunRow {
    id: Uuid,
    database_id: Uuid,
    destination_id: Option<Uuid>,
    operation: String,
    trigger: String,
    source_run_id: Option<Uuid>,
    status: String,
    object_key: Option<String>,
    size_bytes: Option<i64>,
    error_code: Option<String>,
    created_at: OffsetDateTime,
    finished_at: Option<OffsetDateTime>,
}

impl From<RunRow> for RunRecord {
    fn from(row: RunRow) -> Self {
        Self {
            id: row.id.into(),
            database_id: row.database_id.into(),
            destination_id: row.destination_id.map(Id::from),
            operation: row.operation,
            trigger: row.trigger,
            source_run_id: row.source_run_id.map(Id::from),
            status: row.status,
            object_key: row.object_key,
            size_bytes: row.size_bytes,
            error_code: row.error_code,
            created_at: row.created_at,
            finished_at: row.finished_at,
        }
    }
}

#[async_trait]
impl BackupRunRepo for PgStore {
    async fn create_run(&self, run: NewRun) -> AppResult<()> {
        sqlx::query!(
            "INSERT INTO backup_runs (id, database_id, destination_id, operation, trigger,
             source_run_id, created_by)
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
            run.id.as_uuid(),
            run.database_id.as_uuid(),
            run.destination_id.as_uuid(),
            run.operation,
            run.trigger,
            run.source_run_id.map(|id| id.as_uuid()),
            run.created_by.map(|id| id.as_uuid())
        )
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(db_error)
    }

    async fn runs(&self, database_id: Id, limit: i64) -> AppResult<Vec<RunRecord>> {
        let rows = sqlx::query_as!(
            RunRow,
            "SELECT id, database_id, destination_id, operation, trigger, source_run_id, status,
             object_key, size_bytes, error_code, created_at, finished_at
             FROM backup_runs WHERE database_id = $1 ORDER BY created_at DESC LIMIT $2",
            database_id.as_uuid(),
            limit
        )
        .fetch_all(&self.pool)
        .await
        .map_err(db_error)?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn run(&self, id: Id) -> AppResult<Option<RunRecord>> {
        let row = sqlx::query_as!(
            RunRow,
            "SELECT id, database_id, destination_id, operation, trigger, source_run_id, status,
             object_key, size_bytes, error_code, created_at, finished_at
             FROM backup_runs WHERE id = $1",
            id.as_uuid()
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(db_error)?;
        Ok(row.map(Into::into))
    }

    async fn start_run(&self, id: Id) -> AppResult<()> {
        sqlx::query!(
            "UPDATE backup_runs SET status = 'running', error_code = NULL WHERE id = $1",
            id.as_uuid()
        )
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(db_error)
    }

    async fn finish_run(&self, id: Id, outcome: RunOutcome) -> AppResult<()> {
        sqlx::query!(
            "UPDATE backup_runs SET status = $2, object_key = COALESCE($3, object_key),
             size_bytes = COALESCE($4, size_bytes), error_code = $5, finished_at = now()
             WHERE id = $1",
            id.as_uuid(),
            outcome.status,
            outcome.object_key,
            outcome.size_bytes,
            outcome.error_code
        )
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(db_error)
    }

    async fn stored_backups(
        &self,
        database_id: Id,
        destination_id: Id,
    ) -> AppResult<Vec<RunRecord>> {
        let rows = sqlx::query_as!(
            RunRow,
            "SELECT id, database_id, destination_id, operation, trigger, source_run_id, status,
             object_key, size_bytes, error_code, created_at, finished_at
             FROM backup_runs
             WHERE database_id = $1 AND destination_id = $2 AND operation = 'backup'
               AND status = 'succeeded' AND object_key IS NOT NULL
             ORDER BY created_at DESC",
            database_id.as_uuid(),
            destination_id.as_uuid()
        )
        .fetch_all(&self.pool)
        .await
        .map_err(db_error)?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn delete_run(&self, id: Id) -> AppResult<()> {
        sqlx::query!("DELETE FROM backup_runs WHERE id = $1", id.as_uuid())
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(db_error)
    }
}
