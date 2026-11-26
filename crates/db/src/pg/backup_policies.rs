use async_trait::async_trait;
use ferrodock_core::{AppError, AppResult};
use time::OffsetDateTime;
use uuid::Uuid;

use super::{PgStore, db_error};
use crate::backups::{BackupPolicyRepo, BackupTarget, DuePolicy, PolicyRecord};

const DUE_BATCH: i64 = 50;

struct PolicyRow {
    database_id: Option<Uuid>,
    application_id: Option<Uuid>,
    volume: Option<String>,
    destination_id: Uuid,
    schedule: String,
    retention: i32,
    enabled: bool,
    next_run_at: OffsetDateTime,
}

struct DueRow {
    database_id: Option<Uuid>,
    application_id: Option<Uuid>,
    volume: Option<String>,
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
        let target = BackupTarget::from_columns(row.database_id, row.application_id, row.volume)
            .ok_or_else(|| AppError::internal("backup policy without a target"))?;
        Ok(Self {
            target,
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
            application_id: row.application_id,
            volume: row.volume,
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
    async fn policy(&self, target: &BackupTarget) -> AppResult<Option<PolicyRecord>> {
        let (database_id, application_id, volume) = target.columns();
        let row = sqlx::query_as!(
            PolicyRow,
            "SELECT database_id, application_id, volume, destination_id, schedule, retention,
             enabled, next_run_at
             FROM backup_policies
             WHERE database_id IS NOT DISTINCT FROM $1
               AND application_id IS NOT DISTINCT FROM $2 AND volume IS NOT DISTINCT FROM $3",
            database_id,
            application_id,
            volume
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(db_error)?;
        row.map(PolicyRecord::try_from).transpose()
    }

    async fn save_policy(&self, policy: PolicyRecord) -> AppResult<()> {
        let (database_id, application_id, volume) = policy.target.columns();
        let mut tx = self.pool.begin().await.map_err(db_error)?;
        sqlx::query!(
            "DELETE FROM backup_policies
             WHERE database_id IS NOT DISTINCT FROM $1
               AND application_id IS NOT DISTINCT FROM $2 AND volume IS NOT DISTINCT FROM $3",
            database_id,
            application_id,
            volume
        )
        .execute(&mut *tx)
        .await
        .map_err(db_error)?;
        sqlx::query!(
            "INSERT INTO backup_policies (database_id, application_id, volume, destination_id,
             schedule, retention, enabled, next_run_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
            database_id,
            application_id,
            volume,
            policy.destination_id.as_uuid(),
            policy.schedule,
            i32::from(policy.retention),
            policy.enabled,
            policy.next_run_at
        )
        .execute(&mut *tx)
        .await
        .map_err(db_error)?;
        tx.commit().await.map_err(db_error)
    }

    async fn remove_policy(&self, target: &BackupTarget) -> AppResult<()> {
        let (database_id, application_id, volume) = target.columns();
        sqlx::query!(
            "DELETE FROM backup_policies
             WHERE database_id IS NOT DISTINCT FROM $1
               AND application_id IS NOT DISTINCT FROM $2 AND volume IS NOT DISTINCT FROM $3",
            database_id,
            application_id,
            volume
        )
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(db_error)
    }

    async fn due_policies(&self, now: OffsetDateTime) -> AppResult<Vec<DuePolicy>> {
        let rows = sqlx::query_as!(
            DueRow,
            "SELECT b.database_id, b.application_id, b.volume, b.destination_id, b.schedule,
             b.retention, b.enabled, b.next_run_at, p.organization_id
             FROM backup_policies b
             LEFT JOIN databases d ON d.id = b.database_id
             LEFT JOIN applications a ON a.id = b.application_id
             JOIN environments e ON e.id = COALESCE(d.environment_id, a.environment_id)
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
        target: &BackupTarget,
        expected: OffsetDateTime,
        next: OffsetDateTime,
    ) -> AppResult<bool> {
        let (database_id, application_id, volume) = target.columns();
        let updated = sqlx::query!(
            "UPDATE backup_policies SET next_run_at = $5
             WHERE database_id IS NOT DISTINCT FROM $1
               AND application_id IS NOT DISTINCT FROM $2 AND volume IS NOT DISTINCT FROM $3
               AND next_run_at = $4",
            database_id,
            application_id,
            volume,
            expected,
            next
        )
        .execute(&self.pool)
        .await
        .map_err(db_error)?;
        Ok(updated.rows_affected() == 1)
    }
}
