use async_trait::async_trait;
use ferrodock_core::{AppResult, Id};
use time::OffsetDateTime;

use super::{PgStore, db_error};
use crate::LoginAttemptRepo;
use crate::records::FailedAttempt;

#[async_trait]
impl LoginAttemptRepo for PgStore {
    async fn source_locked_until(
        &self,
        user_id: Id,
        source: &str,
        now: OffsetDateTime,
    ) -> AppResult<Option<OffsetDateTime>> {
        sqlx::query_scalar!(
            "SELECT locked_until FROM login_attempts
             WHERE user_id = $1 AND source = $2 AND locked_until > $3",
            user_id.as_uuid(),
            source,
            now
        )
        .fetch_optional(&self.pool)
        .await
        .map(Option::flatten)
        .map_err(db_error)
    }

    async fn account_failures_since(&self, user_id: Id, since: OffsetDateTime) -> AppResult<i64> {
        sqlx::query_scalar!(
            r#"SELECT COALESCE(SUM(failures), 0)::bigint AS "total!" FROM login_attempts
             WHERE user_id = $1 AND window_started_at >= $2"#,
            user_id.as_uuid(),
            since
        )
        .fetch_one(&self.pool)
        .await
        .map_err(db_error)
    }

    async fn record_failure(&self, attempt: FailedAttempt) -> AppResult<()> {
        sqlx::query!(
            "INSERT INTO login_attempts (user_id, source, failures, window_started_at, locked_until)
             VALUES ($1, $2, 1, $3::timestamptz, CASE WHEN 1 >= $5::integer THEN $6::timestamptz END)
             ON CONFLICT (user_id, source) DO UPDATE SET
               failures = CASE WHEN login_attempts.window_started_at < $4::timestamptz THEN 1
                               ELSE login_attempts.failures + 1 END,
               window_started_at = CASE WHEN login_attempts.window_started_at < $4::timestamptz THEN $3::timestamptz
                                        ELSE login_attempts.window_started_at END,
               locked_until = CASE
                 WHEN (CASE WHEN login_attempts.window_started_at < $4::timestamptz THEN 1
                            ELSE login_attempts.failures + 1 END) >= $5::integer THEN $6::timestamptz
                 ELSE login_attempts.locked_until END",
            attempt.user_id.as_uuid(),
            attempt.source,
            attempt.now,
            attempt.window_start,
            attempt.max_failures,
            attempt.lock_until
        )
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(db_error)
    }

    async fn clear(&self, user_id: Id, source: &str) -> AppResult<()> {
        sqlx::query!(
            "DELETE FROM login_attempts WHERE user_id = $1 AND source = $2",
            user_id.as_uuid(),
            source
        )
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(db_error)
    }

    async fn purge_before(&self, before: OffsetDateTime) -> AppResult<u64> {
        sqlx::query!(
            "DELETE FROM login_attempts
             WHERE window_started_at < $1 AND (locked_until IS NULL OR locked_until < $1)",
            before
        )
        .execute(&self.pool)
        .await
        .map(|result| result.rows_affected())
        .map_err(db_error)
    }
}
