use async_trait::async_trait;
use ferrodock_core::{AppError, AppResult, Id};
use time::OffsetDateTime;
use uuid::Uuid;

use super::jobs::insert_job;
use super::{PgStore, db_error};
use crate::deployments::{
    DeploymentArtifact, DeploymentKind, DeploymentRecord, DeploymentRepo, NewDeployment,
};
use crate::jobs::JobStatus;

struct DeploymentRow {
    id: Uuid,
    application_id: Uuid,
    job_id: Uuid,
    kind: String,
    rollback_of: Option<Uuid>,
    commit_sha: Option<String>,
    image_ref: Option<String>,
    image_id: Option<String>,
    created_by: Option<Uuid>,
    created_at: OffsetDateTime,
    status: String,
    error_code: Option<String>,
    started_at: Option<OffsetDateTime>,
    finished_at: Option<OffsetDateTime>,
}

impl From<DeploymentRow> for DeploymentRecord {
    fn from(row: DeploymentRow) -> Self {
        Self {
            id: row.id.into(),
            application_id: row.application_id.into(),
            job_id: row.job_id.into(),
            kind: DeploymentKind::parse(&row.kind),
            rollback_of: row.rollback_of.map(Id::from),
            commit_sha: row.commit_sha,
            image_ref: row.image_ref,
            image_id: row.image_id,
            created_by: row.created_by.map(Id::from),
            created_at: row.created_at,
            status: JobStatus::parse(&row.status),
            error_code: row.error_code,
            started_at: row.started_at,
            finished_at: row.finished_at,
        }
    }
}

#[async_trait]
impl DeploymentRepo for PgStore {
    async fn create(&self, deployment: NewDeployment) -> AppResult<DeploymentRecord> {
        let mut tx = self.pool.begin().await.map_err(db_error)?;
        let job = insert_job(&mut *tx, deployment.job).await?;
        sqlx::query!(
            "INSERT INTO deployments (id, application_id, job_id, kind, rollback_of, created_by)
             VALUES ($1, $2, $3, $4, $5, $6)",
            deployment.id.as_uuid(),
            deployment.application_id.as_uuid(),
            job.id.as_uuid(),
            deployment.kind.as_str(),
            deployment.rollback_of.map(|id| id.as_uuid()),
            deployment.created_by.map(|id| id.as_uuid())
        )
        .execute(&mut *tx)
        .await
        .map_err(db_error)?;
        tx.commit().await.map_err(db_error)?;
        self.find(deployment.id)
            .await?
            .ok_or_else(|| AppError::not_found("DEPLOYMENT_NOT_FOUND"))
    }

    async fn list(&self, application_id: Id, limit: i64) -> AppResult<Vec<DeploymentRecord>> {
        sqlx::query_as!(
            DeploymentRow,
            "SELECT d.id, d.application_id, d.job_id, d.kind, d.rollback_of, d.commit_sha,
             d.image_ref, d.image_id, d.created_by, d.created_at, j.status, j.error_code,
             j.started_at, j.finished_at
             FROM deployments d JOIN jobs j ON j.id = d.job_id
             WHERE d.application_id = $1 ORDER BY d.created_at DESC, d.id DESC LIMIT $2",
            application_id.as_uuid(),
            limit
        )
        .fetch_all(&self.pool)
        .await
        .map(|rows| rows.into_iter().map(DeploymentRecord::from).collect())
        .map_err(db_error)
    }

    async fn find(&self, id: Id) -> AppResult<Option<DeploymentRecord>> {
        sqlx::query_as!(
            DeploymentRow,
            "SELECT d.id, d.application_id, d.job_id, d.kind, d.rollback_of, d.commit_sha,
             d.image_ref, d.image_id, d.created_by, d.created_at, j.status, j.error_code,
             j.started_at, j.finished_at
             FROM deployments d JOIN jobs j ON j.id = d.job_id WHERE d.id = $1",
            id.as_uuid()
        )
        .fetch_optional(&self.pool)
        .await
        .map(|row| row.map(DeploymentRecord::from))
        .map_err(db_error)
    }

    async fn find_in_organization(
        &self,
        organization_id: Id,
        id: Id,
    ) -> AppResult<Option<DeploymentRecord>> {
        sqlx::query_as!(
            DeploymentRow,
            "SELECT d.id, d.application_id, d.job_id, d.kind, d.rollback_of, d.commit_sha,
             d.image_ref, d.image_id, d.created_by, d.created_at, j.status, j.error_code,
             j.started_at, j.finished_at
             FROM deployments d
             JOIN jobs j ON j.id = d.job_id
             JOIN applications a ON a.id = d.application_id
             JOIN environments e ON e.id = a.environment_id
             JOIN projects p ON p.id = e.project_id
             WHERE d.id = $1 AND p.organization_id = $2",
            id.as_uuid(),
            organization_id.as_uuid()
        )
        .fetch_optional(&self.pool)
        .await
        .map(|row| row.map(DeploymentRecord::from))
        .map_err(db_error)
    }

    async fn record_artifact(&self, id: Id, artifact: DeploymentArtifact) -> AppResult<()> {
        sqlx::query!(
            "UPDATE deployments SET commit_sha = $2, image_ref = $3, image_id = $4 WHERE id = $1",
            id.as_uuid(),
            artifact.commit_sha,
            artifact.image_ref,
            artifact.image_id
        )
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(db_error)
    }

    async fn successful(&self, application_id: Id) -> AppResult<Vec<DeploymentRecord>> {
        sqlx::query_as!(
            DeploymentRow,
            "SELECT d.id, d.application_id, d.job_id, d.kind, d.rollback_of, d.commit_sha,
             d.image_ref, d.image_id, d.created_by, d.created_at, j.status, j.error_code,
             j.started_at, j.finished_at
             FROM deployments d JOIN jobs j ON j.id = d.job_id
             WHERE d.application_id = $1 AND j.status = 'succeeded' AND d.image_id IS NOT NULL
             ORDER BY j.finished_at DESC NULLS LAST, d.id DESC",
            application_id.as_uuid()
        )
        .fetch_all(&self.pool)
        .await
        .map(|rows| rows.into_iter().map(DeploymentRecord::from).collect())
        .map_err(db_error)
    }
}
