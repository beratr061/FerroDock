use async_trait::async_trait;
use ferrodock_core::{AppError, AppResult, Id};
use time::OffsetDateTime;
use uuid::Uuid;

use super::{PgStore, db_error};
use crate::backups::{BackupRunRepo, BackupTarget, NewRun, RunOutcome, RunRecord};

struct RunRow {
    id: Uuid,
    database_id: Option<Uuid>,
    application_id: Option<Uuid>,
    volume: Option<String>,
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

impl TryFrom<RunRow> for RunRecord {
    type Error = AppError;

    fn try_from(row: RunRow) -> AppResult<Self> {
        let target = BackupTarget::from_columns(row.database_id, row.application_id, row.volume)
            .ok_or_else(|| AppError::internal("backup run without a target"))?;
        Ok(Self {
            id: row.id.into(),
            target,
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
        })
    }
}

fn records(rows: Vec<RunRow>) -> AppResult<Vec<RunRecord>> {
    rows.into_iter().map(RunRecord::try_from).collect()
}

#[async_trait]
impl BackupRunRepo for PgStore {
    async fn create_run(&self, run: NewRun) -> AppResult<()> {
        let (database_id, application_id, volume) = run.target.columns();
        sqlx::query!(
            "INSERT INTO backup_runs (id, database_id, application_id, volume, destination_id,
             operation, trigger, source_run_id, created_by)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
            run.id.as_uuid(),
            database_id,
            application_id,
            volume,
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

    async fn runs(&self, target: &BackupTarget, limit: i64) -> AppResult<Vec<RunRecord>> {
        let (database_id, application_id, volume) = target.columns();
        let rows = sqlx::query_as!(
            RunRow,
            "SELECT id, database_id, application_id, volume, destination_id, operation, trigger,
             source_run_id, status, object_key, size_bytes, error_code, created_at, finished_at
             FROM backup_runs
             WHERE database_id IS NOT DISTINCT FROM $1
               AND application_id IS NOT DISTINCT FROM $2 AND volume IS NOT DISTINCT FROM $3
             ORDER BY created_at DESC LIMIT $4",
            database_id,
            application_id,
            volume,
            limit
        )
        .fetch_all(&self.pool)
        .await
        .map_err(db_error)?;
        records(rows)
    }

    async fn run(&self, id: Id) -> AppResult<Option<RunRecord>> {
        let row = sqlx::query_as!(
            RunRow,
            "SELECT id, database_id, application_id, volume, destination_id, operation, trigger,
             source_run_id, status, object_key, size_bytes, error_code, created_at, finished_at
             FROM backup_runs WHERE id = $1",
            id.as_uuid()
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(db_error)?;
        row.map(RunRecord::try_from).transpose()
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
        target: &BackupTarget,
        destination_id: Id,
    ) -> AppResult<Vec<RunRecord>> {
        let (database_id, application_id, volume) = target.columns();
        let rows = sqlx::query_as!(
            RunRow,
            "SELECT id, database_id, application_id, volume, destination_id, operation, trigger,
             source_run_id, status, object_key, size_bytes, error_code, created_at, finished_at
             FROM backup_runs
             WHERE database_id IS NOT DISTINCT FROM $1
               AND application_id IS NOT DISTINCT FROM $2 AND volume IS NOT DISTINCT FROM $3
               AND destination_id = $4 AND operation = 'backup'
               AND status = 'succeeded' AND object_key IS NOT NULL
             ORDER BY created_at DESC",
            database_id,
            application_id,
            volume,
            destination_id.as_uuid()
        )
        .fetch_all(&self.pool)
        .await
        .map_err(db_error)?;
        records(rows)
    }

    async fn delete_run(&self, id: Id) -> AppResult<()> {
        sqlx::query!("DELETE FROM backup_runs WHERE id = $1", id.as_uuid())
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(db_error)
    }
}
