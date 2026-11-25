use ferrodock_core::{AppResult, Id};
use ferrodock_db::records::FailedAttempt;
use time::{Duration, OffsetDateTime};

use super::service::AuthService;

const SOURCE_MAX_FAILURES: i32 = 5;
const ACCOUNT_MAX_FAILURES: i64 = 100;
const WINDOW: Duration = Duration::minutes(15);
const LOCKOUT: Duration = Duration::minutes(15);
const UNKNOWN_SOURCE: &str = "unknown";

pub(super) fn source_key(ip_address: Option<&str>) -> &str {
    ip_address.unwrap_or(UNKNOWN_SOURCE)
}

impl AuthService {
    pub(super) async fn is_throttled(&self, user_id: Id, source: &str) -> AppResult<bool> {
        let now = OffsetDateTime::now_utc();
        let attempts = &self.repos.login_attempts;
        if attempts
            .source_locked_until(user_id, source, now)
            .await?
            .is_some()
        {
            return Ok(true);
        }
        let account_failures = attempts
            .account_failures_since(user_id, now - WINDOW)
            .await?;
        Ok(account_failures >= ACCOUNT_MAX_FAILURES)
    }

    pub(super) async fn record_login_failure(&self, user_id: Id, source: &str) -> AppResult<()> {
        let now = OffsetDateTime::now_utc();
        self.repos
            .login_attempts
            .record_failure(FailedAttempt {
                user_id,
                source: source.to_owned(),
                now,
                window_start: now - WINDOW,
                max_failures: SOURCE_MAX_FAILURES,
                lock_until: now + LOCKOUT,
            })
            .await
    }

    pub(super) async fn clear_login_failures(&self, user_id: Id, source: &str) -> AppResult<()> {
        self.repos.login_attempts.clear(user_id, source).await
    }

    pub async fn purge_stale_login_attempts(&self) -> AppResult<u64> {
        let cutoff = OffsetDateTime::now_utc() - WINDOW - LOCKOUT;
        self.repos.login_attempts.purge_before(cutoff).await
    }
}
