use async_trait::async_trait;
use ferrodock_core::{AppResult, Id};
use time::OffsetDateTime;

use super::{PgStore, db_error};
use crate::SessionRepo;
use crate::records::{NewSession, SessionRecord};

#[async_trait]
impl SessionRepo for PgStore {
    async fn create(&self, session: NewSession) -> AppResult<()> {
        sqlx::query!(
            "INSERT INTO sessions
             (id, user_id, token_hash, active_organization_id, ip_address, user_agent, expires_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
            session.id.as_uuid(),
            session.user_id.as_uuid(),
            session.token_hash,
            session.active_organization_id.map(|id| id.as_uuid()),
            session.ip_address,
            session.user_agent,
            session.expires_at
        )
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(db_error)
    }

    async fn find_active(
        &self,
        token_hash: &[u8],
        now: OffsetDateTime,
    ) -> AppResult<Option<SessionRecord>> {
        let row = sqlx::query!(
            "SELECT id, user_id, active_organization_id, expires_at, last_seen_at
             FROM sessions WHERE token_hash = $1 AND expires_at > $2",
            token_hash,
            now
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(db_error)?;
        Ok(row.map(|r| SessionRecord {
            id: r.id.into(),
            user_id: r.user_id.into(),
            active_organization_id: r.active_organization_id.map(Id::from),
            expires_at: r.expires_at,
            last_seen_at: r.last_seen_at,
        }))
    }

    async fn extend(
        &self,
        id: Id,
        now: OffsetDateTime,
        expires_at: OffsetDateTime,
    ) -> AppResult<()> {
        sqlx::query!(
            "UPDATE sessions SET last_seen_at = $2, expires_at = $3 WHERE id = $1",
            id.as_uuid(),
            now,
            expires_at
        )
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(db_error)
    }

    async fn delete(&self, id: Id) -> AppResult<()> {
        sqlx::query!("DELETE FROM sessions WHERE id = $1", id.as_uuid())
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(db_error)
    }

    async fn delete_expired(&self, now: OffsetDateTime) -> AppResult<u64> {
        sqlx::query!("DELETE FROM sessions WHERE expires_at <= $1", now)
            .execute(&self.pool)
            .await
            .map(|result| result.rows_affected())
            .map_err(db_error)
    }
}
