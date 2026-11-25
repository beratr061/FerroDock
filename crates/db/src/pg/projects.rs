use async_trait::async_trait;
use ferrodock_core::{AppResult, Id};
use time::OffsetDateTime;
use uuid::Uuid;

use super::{PgStore, db_error, environments};
use crate::projects::{
    EnvironmentRecord, EnvironmentRemoval, NewEnvironment, NewProject, ProjectChanges,
    ProjectRecord, ProjectRepo,
};

struct ProjectRow {
    id: Uuid,
    organization_id: Uuid,
    name: String,
    slug: String,
    description: String,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
}

impl From<ProjectRow> for ProjectRecord {
    fn from(row: ProjectRow) -> Self {
        Self {
            id: row.id.into(),
            organization_id: row.organization_id.into(),
            name: row.name,
            slug: row.slug,
            description: row.description,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[async_trait]
impl ProjectRepo for PgStore {
    async fn create(&self, project: NewProject) -> AppResult<(ProjectRecord, EnvironmentRecord)> {
        let mut tx = self.pool.begin().await.map_err(db_error)?;
        let record = sqlx::query_as!(
            ProjectRow,
            "INSERT INTO projects (id, organization_id, name, slug, description)
             VALUES ($1, $2, $3, $4, $5)
             RETURNING id, organization_id, name, slug, description, created_at, updated_at",
            project.id.as_uuid(),
            project.organization_id.as_uuid(),
            project.name,
            project.slug,
            project.description
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(db_error)?;
        let environment = environments::insert(&mut *tx, project.first_environment).await?;
        tx.commit().await.map_err(db_error)?;
        Ok((record.into(), environment))
    }

    async fn list(&self, organization_id: Id) -> AppResult<Vec<ProjectRecord>> {
        sqlx::query_as!(
            ProjectRow,
            "SELECT id, organization_id, name, slug, description, created_at, updated_at
             FROM projects WHERE organization_id = $1 ORDER BY lower(name), id",
            organization_id.as_uuid()
        )
        .fetch_all(&self.pool)
        .await
        .map(|rows| rows.into_iter().map(ProjectRecord::from).collect())
        .map_err(db_error)
    }

    async fn list_environments(&self, organization_id: Id) -> AppResult<Vec<EnvironmentRecord>> {
        self.organization_environments(organization_id).await
    }

    async fn find(&self, organization_id: Id, id: Id) -> AppResult<Option<ProjectRecord>> {
        sqlx::query_as!(
            ProjectRow,
            "SELECT id, organization_id, name, slug, description, created_at, updated_at
             FROM projects WHERE organization_id = $1 AND id = $2",
            organization_id.as_uuid(),
            id.as_uuid()
        )
        .fetch_optional(&self.pool)
        .await
        .map(|row| row.map(ProjectRecord::from))
        .map_err(db_error)
    }

    async fn environments(&self, project_id: Id) -> AppResult<Vec<EnvironmentRecord>> {
        self.project_environments(project_id).await
    }

    async fn update(
        &self,
        organization_id: Id,
        id: Id,
        changes: ProjectChanges,
    ) -> AppResult<Option<ProjectRecord>> {
        sqlx::query_as!(
            ProjectRow,
            "UPDATE projects SET name = $3, description = $4, updated_at = now()
             WHERE organization_id = $1 AND id = $2
             RETURNING id, organization_id, name, slug, description, created_at, updated_at",
            organization_id.as_uuid(),
            id.as_uuid(),
            changes.name,
            changes.description
        )
        .fetch_optional(&self.pool)
        .await
        .map(|row| row.map(ProjectRecord::from))
        .map_err(db_error)
    }

    async fn delete(&self, organization_id: Id, id: Id) -> AppResult<bool> {
        sqlx::query!(
            "DELETE FROM projects WHERE organization_id = $1 AND id = $2",
            organization_id.as_uuid(),
            id.as_uuid()
        )
        .execute(&self.pool)
        .await
        .map(|result| result.rows_affected() > 0)
        .map_err(db_error)
    }

    async fn create_environment(
        &self,
        environment: NewEnvironment,
    ) -> AppResult<EnvironmentRecord> {
        environments::insert(&self.pool, environment).await
    }

    async fn delete_environment(&self, project_id: Id, id: Id) -> AppResult<EnvironmentRemoval> {
        self.remove_environment(project_id, id).await
    }
}
