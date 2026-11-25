use async_trait::async_trait;
use ferrodock_core::{AppResult, Id};

use super::{PgStore, db_error};
use crate::AuditRepo;
use crate::records::NewAuditEntry;

#[async_trait]
impl AuditRepo for PgStore {
    async fn record(&self, entry: NewAuditEntry) -> AppResult<()> {
        sqlx::query!(
            "INSERT INTO audit_logs
             (id, organization_id, actor_id, action, resource_type, resource_id, metadata, ip_address)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
            Id::new().as_uuid(),
            entry.organization_id.map(|id| id.as_uuid()),
            entry.actor_id.map(|id| id.as_uuid()),
            entry.action,
            entry.resource_type,
            entry.resource_id.map(|id| id.as_uuid()),
            entry.metadata,
            entry.ip_address
        )
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(db_error)
    }
}
