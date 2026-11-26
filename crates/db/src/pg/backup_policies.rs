use async_trait::async_trait;
use ferrodock_core::{AppError, AppResult, Id};
use time::OffsetDateTime;
use uuid::Uuid;

use super::{PgStore, db_error};
use crate::backups::{BackupPolicyRepo, DuePolicy, PolicyRecord};

const DUE_BATCH: i64 = 50;

struct PolicyRow {
    database_id: Uuid,
    destination_id: Uuid,
    schedule: String,
    retention: i32,
    enabled: bool,
    next_run_at: OffsetDateTime,
}

struct DueRow {
    database_id: Uuid,
    destination_id: Uuid,
    schedule: String,
    retention: i32,
    enabled: bool,
    next_run_at: OffsetDateTime,
    organization_id: Uuid,
}

impl TryFrom<PolicyRow> for PolicyRecord {
    type Error = AppError;

    fn try_from(row: PolicyRow) -> AppResult<Self> {
        Ok(Self {
            database_id: row.database_id.into(),
            destination_id: row.destination_id.into(),
            schedule: row.schedule,
            retention: u16::try_from(row.retention).map_err(AppError::internal)?,
            enabled: row.enabled,
            next_run_at: row.next_run_at,
        })
    }
}

impl TryFrom<DueRow> for DuePolicy {
    type Error = AppError;

    fn try_from(row: DueRow) -> AppResult<Self> {
        let policy = PolicyRow {
            database_id: row.database_id,
            destination_id: row.destination_id,
            schedule: row.schedule,
            retention: row.retention,
            enabled: row.enabled,
            next_run_at: row.next_run_at,
        };
        Ok(Self {
            policy: policy.try_into()?,
            organization_id: row.organization_id.into(),
        })
    }
}

#[async_trait]
impl BackupPolicyRepo for PgStore {
    async fn policy(&self, database_id: Id) -> AppResult<Option<PolicyRecord>> {
        let row = sqlx::query_as!(
            PolicyRow,
            "SELECT database_id, destination_id, schedule, retention, enabled, next_run_at
             FROM backup_policies WHERE database_id = $1",
            database_id.as_uuid()
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(db_error)?;
        row.map(PolicyRecord::try_from).transpose()
    }

    async fn save_policy(&self, policy: PolicyRecord) -> AppResult<()> {
        sqlx::query!(
            "INSERT INTO backup_policies (database_id, destination_id, schedule, retention,
             enabled, next_run_at)
             VALUES ($1, $2, $3, $4, $5, $6)
             ON CONFLICT (database_id) DO UPDATE SET destination_id = EXCLUDED.destination_id,
             schedule = EXCLUDED.schedule, retention = EXCLUDED.retention,
             enabled = EXCLUDED.enabled, next_run_at = EXCLUDED.next_run_at, updated_at = now()",
            policy.database_id.as_uuid(),
            policy.destination_id.as_uuid(),
            policy.schedule,
            i32::from(policy.retention),
            policy.enabled,
            policy.next_run_at
        )
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(db_error)
    }

    async fn remove_policy(&self, database_id: Id) -> AppResult<()> {
        sqlx::query!(
            "DELETE FROM backup_policies WHERE database_id = $1",
            database_id.as_uuid()
        )
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(db_error)
    }

    async fn due_policies(&self, now: OffsetDateTime) -> AppResult<Vec<DuePolicy>> {
        let rows = sqlx::query_as!(
            DueRow,
            "SELECT b.database_id, b.destination_id, b.schedule, b.retention, b.enabled,
             b.next_run_at, p.organization_id
             FROM backup_policies b
             JOIN databases d ON d.id = b.database_id
             JOIN environments e ON e.id = d.environment_id
             JOIN projects p ON p.id = e.project_id
             WHERE b.enabled AND b.next_run_at <= $1
             ORDER BY b.next_run_at LIMIT $2",
            now,
            DUE_BATCH
        )
        .fetch_all(&self.pool)
        .await
        .map_err(db_error)?;
        rows.into_iter().map(DuePolicy::try_from).collect()
    }

    async fn advance_policy(
        &self,
        database_id: Id,
        expected: OffsetDateTime,
        next: OffsetDateTime,
    ) -> AppResult<bool> {
        let updated = sqlx::query!(
            "UPDATE backup_policies SET next_run_at = $3
             WHERE database_id = $1 AND next_run_at = $2",
            database_id.as_uuid(),
            expected,
            next
        )
        .execute(&self.pool)
        .await
        .map_err(db_error)?;
        Ok(updated.rows_affected() == 1)
    }
}
