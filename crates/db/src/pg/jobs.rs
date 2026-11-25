use async_trait::async_trait;
use ferrodock_core::{AppResult, Id};
use serde_json::Value;
use time::OffsetDateTime;
use uuid::Uuid;

use super::{PgStore, db_error};
use crate::jobs::{CancelResult, JobOutcome, JobQueue, JobRecord, JobStatus, NewJob};

struct JobRow {
    id: Uuid,
    organization_id: Option<Uuid>,
    kind: String,
    queue: String,
    group_key: Option<String>,
    payload: Value,
    status: String,
    attempts: i32,
    error_code: Option<String>,
    created_at: OffsetDateTime,
    started_at: Option<OffsetDateTime>,
    finished_at: Option<OffsetDateTime>,
}

impl From<JobRow> for JobRecord {
    fn from(row: JobRow) -> Self {
        Self {
            id: row.id.into(),
            organization_id: row.organization_id.map(Id::from),
            kind: row.kind,
            queue: row.queue,
            group_key: row.group_key,
            payload: row.payload,
            status: JobStatus::parse(&row.status),
            attempts: row.attempts,
            error_code: row.error_code,
            created_at: row.created_at,
            started_at: row.started_at,
            finished_at: row.finished_at,
        }
    }
}

fn outcome_parts(outcome: &JobOutcome) -> (&'static str, Option<&str>) {
    match outcome {
        JobOutcome::Succeeded => (JobStatus::Succeeded.as_str(), None),
        JobOutcome::Failed { code } => (JobStatus::Failed.as_str(), Some(code.as_str())),
        JobOutcome::Cancelled => (JobStatus::Cancelled.as_str(), None),
    }
}

pub(super) async fn insert_job<'e>(
    executor: impl sqlx::Executor<'e, Database = sqlx::Postgres>,
    job: NewJob,
) -> AppResult<JobRecord> {
    sqlx::query_as!(
        JobRow,
        "INSERT INTO jobs (id, organization_id, kind, queue, group_key, payload, status)
         VALUES ($1, $2, $3, $4, $5, $6, 'queued')
         RETURNING id, organization_id, kind, queue, group_key, payload, status, attempts,
                   error_code, created_at, started_at, finished_at",
        Id::new().as_uuid(),
        job.organization_id.map(|id| id.as_uuid()),
        job.kind,
        job.queue,
        job.group_key,
        job.payload
    )
    .fetch_one(executor)
    .await
    .map(JobRecord::from)
    .map_err(db_error)
}

#[async_trait]
impl JobQueue for PgStore {
    async fn enqueue(&self, job: NewJob) -> AppResult<JobRecord> {
        insert_job(&self.pool, job).await
    }

    async fn claim(
        &self,
        queue: &str,
        worker: &str,
        concurrency: i64,
    ) -> AppResult<Option<JobRecord>> {
        let mut tx = self.pool.begin().await.map_err(db_error)?;
        sqlx::query!("SELECT pg_advisory_xact_lock(hashtext($1))", queue)
            .execute(&mut *tx)
            .await
            .map_err(db_error)?;
        let claimed = sqlx::query_as!(
            JobRow,
            "UPDATE jobs SET status = 'running', attempts = attempts + 1, locked_by = $2,
                    started_at = now()
             WHERE id = (
                 SELECT j.id FROM jobs j
                 WHERE j.queue = $1 AND j.status = 'queued'
                   AND (SELECT count(*) FROM jobs r WHERE r.queue = $1 AND r.status = 'running') < $3
                   AND (j.group_key IS NULL OR NOT EXISTS (
                        SELECT 1 FROM jobs g WHERE g.group_key = j.group_key
                          AND (g.status = 'running'
                               OR (g.status = 'queued' AND g.created_at < j.created_at))))
                 ORDER BY j.created_at
                 LIMIT 1
                 FOR UPDATE SKIP LOCKED)
             RETURNING id, organization_id, kind, queue, group_key, payload, status, attempts,
                       error_code, created_at, started_at, finished_at",
            queue,
            worker,
            concurrency
        )
        .fetch_optional(&mut *tx)
        .await
        .map_err(db_error)?;
        tx.commit().await.map_err(db_error)?;
        Ok(claimed.map(JobRecord::from))
    }

    async fn complete(&self, id: Id, outcome: JobOutcome) -> AppResult<()> {
        let (status, code) = outcome_parts(&outcome);
        sqlx::query!(
            "UPDATE jobs SET status = $2, error_code = $3, finished_at = now(), locked_by = NULL
             WHERE id = $1 AND status = 'running'",
            id.as_uuid(),
            status,
            code
        )
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(db_error)
    }

    async fn request_cancel(&self, id: Id) -> AppResult<CancelResult> {
        let status = sqlx::query_scalar!(
            "UPDATE jobs SET
                 status = CASE WHEN status = 'queued' THEN 'cancelled' ELSE status END,
                 finished_at = CASE WHEN status = 'queued' THEN now() ELSE finished_at END,
                 cancel_requested = cancel_requested OR status = 'running'
             WHERE id = $1 AND status IN ('queued', 'running')
             RETURNING status",
            id.as_uuid()
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(db_error)?;
        Ok(match status.as_deref() {
            Some("cancelled") => CancelResult::Cancelled,
            Some(_) => CancelResult::Requested,
            None => CancelResult::NotCancellable,
        })
    }

    async fn cancel_requested(&self, id: Id) -> AppResult<bool> {
        sqlx::query_scalar!(
            "SELECT cancel_requested FROM jobs WHERE id = $1",
            id.as_uuid()
        )
        .fetch_optional(&self.pool)
        .await
        .map(|value| value.unwrap_or(false))
        .map_err(db_error)
    }

    async fn fail_interrupted(&self, queue: &str) -> AppResult<u64> {
        sqlx::query!(
            "UPDATE jobs SET status = 'failed', error_code = 'JOB_INTERRUPTED',
                    finished_at = now(), locked_by = NULL
             WHERE queue = $1 AND status = 'running'",
            queue
        )
        .execute(&self.pool)
        .await
        .map(|result| result.rows_affected())
        .map_err(db_error)
    }

    async fn active_count(&self, kind: &str) -> AppResult<i64> {
        sqlx::query_scalar!(
            "SELECT count(*) AS \"count!\" FROM jobs
             WHERE kind = $1 AND status IN ('queued', 'running')",
            kind
        )
        .fetch_one(&self.pool)
        .await
        .map_err(db_error)
    }

    async fn find(&self, id: Id) -> AppResult<Option<JobRecord>> {
        sqlx::query_as!(
            JobRow,
            "SELECT id, organization_id, kind, queue, group_key, payload, status, attempts,
                    error_code, created_at, started_at, finished_at
             FROM jobs WHERE id = $1",
            id.as_uuid()
        )
        .fetch_optional(&self.pool)
        .await
        .map(|row| row.map(JobRecord::from))
        .map_err(db_error)
    }
}

#[derive(Debug)]
pub struct JobSignals {
    listener: sqlx::postgres::PgListener,
}

impl JobSignals {
    pub async fn next(&mut self) -> AppResult<String> {
        self.listener
            .recv()
            .await
            .map(|notification| notification.payload().to_owned())
            .map_err(db_error)
    }
}

impl PgStore {
    pub async fn job_signals(&self) -> AppResult<JobSignals> {
        let mut listener = sqlx::postgres::PgListener::connect_with(&self.pool)
            .await
            .map_err(db_error)?;
        listener.listen("ferrodock_jobs").await.map_err(db_error)?;
        Ok(JobSignals { listener })
    }
}
