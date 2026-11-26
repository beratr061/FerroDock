use async_trait::async_trait;
use ferrodock_core::{AppResult, Id};

use super::{PgStore, db_error};
use crate::webhooks::{WebhookRecord, WebhookRepo, WebhookTarget};

#[async_trait]
impl WebhookRepo for PgStore {
    async fn find(&self, application_id: Id) -> AppResult<Option<WebhookRecord>> {
        sqlx::query!(
            "SELECT application_id, secret_enc, created_at, last_delivery_at, last_outcome
             FROM application_webhooks WHERE application_id = $1",
            application_id.as_uuid()
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(db_error)
        .map(|row| {
            row.map(|row| WebhookRecord {
                application_id: row.application_id.into(),
                secret_enc: row.secret_enc,
                created_at: row.created_at,
                last_delivery_at: row.last_delivery_at,
                last_outcome: row.last_outcome,
            })
        })
    }

    async fn target(&self, application_id: Id) -> AppResult<Option<WebhookTarget>> {
        sqlx::query!(
            "SELECT p.organization_id, a.id, w.secret_enc, a.git_branch
             FROM application_webhooks w
             JOIN applications a ON a.id = w.application_id
             JOIN environments e ON e.id = a.environment_id
             JOIN projects p ON p.id = e.project_id
             WHERE w.application_id = $1",
            application_id.as_uuid()
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(db_error)
        .map(|row| {
            row.map(|row| WebhookTarget {
                organization_id: row.organization_id.into(),
                application_id: row.id.into(),
                secret_enc: row.secret_enc,
                git_branch: row.git_branch,
            })
        })
    }

    async fn save(&self, application_id: Id, secret_enc: &str) -> AppResult<WebhookRecord> {
        sqlx::query!(
            "INSERT INTO application_webhooks (application_id, secret_enc) VALUES ($1, $2)
             ON CONFLICT (application_id) DO UPDATE SET secret_enc = EXCLUDED.secret_enc,
             created_at = now(), last_delivery_at = NULL, last_outcome = NULL",
            application_id.as_uuid(),
            secret_enc
        )
        .execute(&self.pool)
        .await
        .map_err(db_error)?;
        self.find(application_id)
            .await?
            .ok_or_else(|| ferrodock_core::AppError::not_found("WEBHOOK_NOT_FOUND"))
    }

    async fn remove(&self, application_id: Id) -> AppResult<bool> {
        sqlx::query!(
            "DELETE FROM application_webhooks WHERE application_id = $1",
            application_id.as_uuid()
        )
        .execute(&self.pool)
        .await
        .map(|done| done.rows_affected() > 0)
        .map_err(db_error)
    }

    async fn record_delivery(&self, application_id: Id, outcome: &str) -> AppResult<()> {
        sqlx::query!(
            "UPDATE application_webhooks SET last_delivery_at = now(), last_outcome = $2
             WHERE application_id = $1",
            application_id.as_uuid(),
            outcome
        )
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(db_error)
    }
}
