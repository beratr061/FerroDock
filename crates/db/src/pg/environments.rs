use ferrodock_core::{AppResult, Id};
use sqlx::{Executor, Postgres};
use time::OffsetDateTime;
use uuid::Uuid;

use super::{PgStore, db_error};
use crate::projects::{EnvironmentRecord, EnvironmentRemoval, NewEnvironment};

pub(super) struct EnvironmentRow {
    id: Uuid,
    project_id: Uuid,
    name: String,
    slug: String,
    created_at: OffsetDateTime,
}

impl From<EnvironmentRow> for EnvironmentRecord {
    fn from(row: EnvironmentRow) -> Self {
        Self {
            id: row.id.into(),
            project_id: row.project_id.into(),
            name: row.name,
            slug: row.slug,
            created_at: row.created_at,
        }
    }
}

pub(super) async fn insert<'e>(
    executor: impl Executor<'e, Database = Postgres>,
    environment: NewEnvironment,
) -> AppResult<EnvironmentRecord> {
    sqlx::query_as!(
        EnvironmentRow,
        "INSERT INTO environments (id, project_id, name, slug) VALUES ($1, $2, $3, $4)
         RETURNING id, project_id, name, slug, created_at",
        environment.id.as_uuid(),
        environment.project_id.as_uuid(),
        environment.name,
        environment.slug
    )
    .fetch_one(executor)
    .await
    .map(EnvironmentRecord::from)
    .map_err(db_error)
}

impl PgStore {
    pub(super) async fn organization_environments(
        &self,
        organization_id: Id,
    ) -> AppResult<Vec<EnvironmentRecord>> {
        sqlx::query_as!(
            EnvironmentRow,
            "SELECT e.id, e.project_id, e.name, e.slug, e.created_at
             FROM environments e JOIN projects p ON p.id = e.project_id
             WHERE p.organization_id = $1 ORDER BY e.created_at, e.id",
            organization_id.as_uuid()
        )
        .fetch_all(&self.pool)
        .await
        .map(|rows| rows.into_iter().map(EnvironmentRecord::from).collect())
        .map_err(db_error)
    }

    pub(super) async fn project_environments(
        &self,
        project_id: Id,
    ) -> AppResult<Vec<EnvironmentRecord>> {
        sqlx::query_as!(
            EnvironmentRow,
            "SELECT id, project_id, name, slug, created_at FROM environments
             WHERE project_id = $1 ORDER BY created_at, id",
            project_id.as_uuid()
        )
        .fetch_all(&self.pool)
        .await
        .map(|rows| rows.into_iter().map(EnvironmentRecord::from).collect())
        .map_err(db_error)
    }

    pub(super) async fn remove_environment(
        &self,
        project_id: Id,
        id: Id,
    ) -> AppResult<EnvironmentRemoval> {
        let mut tx = self.pool.begin().await.map_err(db_error)?;
        let siblings = sqlx::query_scalar!(
            "SELECT id FROM environments WHERE project_id = $1 FOR UPDATE",
            project_id.as_uuid()
        )
        .fetch_all(&mut *tx)
        .await
        .map_err(db_error)?;
        let outcome = match (siblings.contains(&id.as_uuid()), siblings.len()) {
            (false, _) => EnvironmentRemoval::NotFound,
            (true, 1) => EnvironmentRemoval::LastRemaining,
            (true, _) => EnvironmentRemoval::Removed,
        };
        if outcome == EnvironmentRemoval::Removed {
            sqlx::query!("DELETE FROM environments WHERE id = $1", id.as_uuid())
                .execute(&mut *tx)
                .await
                .map_err(db_error)?;
        }
        tx.commit().await.map_err(db_error)?;
        Ok(outcome)
    }
}
