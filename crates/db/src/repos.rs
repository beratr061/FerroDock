use async_trait::async_trait;
use ferrodock_core::{AppResult, Id};
use time::OffsetDateTime;

use crate::records::{
    ApiKeyRecord, FailedAttempt, MembershipRecord, NewApiKey, NewAuditEntry, NewOwner, NewSession,
    OrganizationRecord, SessionRecord, UserRecord,
};

#[async_trait]
pub trait IdentityRepo: Send + Sync + 'static {
    async fn has_users(&self) -> AppResult<bool>;
    async fn create_owner(&self, owner: NewOwner) -> AppResult<(UserRecord, OrganizationRecord)>;
    async fn find_user_by_email(&self, email_normalized: &str) -> AppResult<Option<UserRecord>>;
    async fn find_user(&self, id: Id) -> AppResult<Option<UserRecord>>;
    async fn memberships(&self, user_id: Id) -> AppResult<Vec<MembershipRecord>>;
}

#[async_trait]
pub trait SessionRepo: Send + Sync + 'static {
    async fn create(&self, session: NewSession) -> AppResult<()>;
    async fn find_active(
        &self,
        token_hash: &[u8],
        now: OffsetDateTime,
    ) -> AppResult<Option<SessionRecord>>;
    async fn extend(
        &self,
        id: Id,
        now: OffsetDateTime,
        expires_at: OffsetDateTime,
    ) -> AppResult<()>;
    async fn delete(&self, id: Id) -> AppResult<()>;
    async fn delete_expired(&self, now: OffsetDateTime) -> AppResult<u64>;
}

#[async_trait]
pub trait ApiKeyRepo: Send + Sync + 'static {
    async fn create(&self, key: NewApiKey) -> AppResult<ApiKeyRecord>;
    async fn list(&self, user_id: Id, organization_id: Id) -> AppResult<Vec<ApiKeyRecord>>;
    async fn find_active(
        &self,
        key_hash: &[u8],
        now: OffsetDateTime,
    ) -> AppResult<Option<ApiKeyRecord>>;
    async fn revoke(&self, id: Id, user_id: Id) -> AppResult<bool>;
    async fn mark_used(&self, id: Id, now: OffsetDateTime) -> AppResult<()>;
}

#[async_trait]
pub trait AuditRepo: Send + Sync + 'static {
    async fn record(&self, entry: NewAuditEntry) -> AppResult<()>;
}

#[async_trait]
pub trait LoginAttemptRepo: Send + Sync + 'static {
    async fn source_locked_until(
        &self,
        user_id: Id,
        source: &str,
        now: OffsetDateTime,
    ) -> AppResult<Option<OffsetDateTime>>;
    async fn account_failures_since(&self, user_id: Id, since: OffsetDateTime) -> AppResult<i64>;
    async fn record_failure(&self, attempt: FailedAttempt) -> AppResult<()>;
    async fn clear(&self, user_id: Id, source: &str) -> AppResult<()>;
    async fn purge_before(&self, before: OffsetDateTime) -> AppResult<u64>;
}
