use async_trait::async_trait;
use ferrodock_core::{AppError, AppResult, Id};
use time::OffsetDateTime;
use uuid::Uuid;

use super::{PgStore, db_error};
use crate::databases::{DatabaseRecord, DatabaseRepo, NewDatabase, ResourceLimits};

fn positive(value: Option<i32>) -> AppResult<Option<u32>> {
    value
        .map(u32::try_from)
        .transpose()
        .map_err(AppError::internal)
}

struct DatabaseRow {
    id: Uuid,
    environment_id: Uuid,
    project_id: Uuid,
    name: String,
    slug: String,
    engine: String,
    version: String,
    database_name: String,
    username: String,
    password_enc: String,
    root_password_enc: Option<String>,
    external_port: Option<i32>,
    memory_limit_mb: Option<i32>,
    cpu_limit_millis: Option<i32>,
    status: String,
    error_code: Option<String>,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
}

impl TryFrom<DatabaseRow> for DatabaseRecord {
    type Error = AppError;

    fn try_from(row: DatabaseRow) -> AppResult<Self> {
        Ok(Self {
            id: row.id.into(),
            environment_id: row.environment_id.into(),
            project_id: row.project_id.into(),
            name: row.name,
            slug: row.slug,
            engine: row.engine,
            version: row.version,
            database_name: row.database_name,
            username: row.username,
            password_enc: row.password_enc,
            root_password_enc: row.root_password_enc,
            external_port: row
                .external_port
                .map(u16::try_from)
                .transpose()
                .map_err(AppError::internal)?,
            limits: ResourceLimits {
                memory_mb: positive(row.memory_limit_mb)?,
                cpu_millis: positive(row.cpu_limit_millis)?,
            },
            status: row.status,
            error_code: row.error_code,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

#[async_trait]
impl DatabaseRepo for PgStore {
    async fn create(&self, database: NewDatabase) -> AppResult<Option<DatabaseRecord>> {
        let inserted = sqlx::query!(
            "INSERT INTO databases (id, environment_id, name, slug, engine, version, database_name,
             username, password_enc, root_password_enc)
             SELECT $1, e.id, $3, $4, $5, $6, $7, $8, $9, $10
             FROM environments e JOIN projects p ON p.id = e.project_id
             WHERE e.id = $2 AND p.organization_id = $11",
            database.id.as_uuid(),
            database.environment_id.as_uuid(),
            database.name,
            database.slug,
            database.engine,
            database.version,
            database.database_name,
            database.username,
            database.password_enc,
            database.root_password_enc,
            database.organization_id.as_uuid()
        )
        .execute(&self.pool)
        .await
        .map_err(db_error)?;
        if inserted.rows_affected() == 0 {
            return Ok(None);
        }
        self.find(database.organization_id, database.id).await
    }

    async fn list_for_project(
        &self,
        organization_id: Id,
        project_id: Id,
    ) -> AppResult<Vec<DatabaseRecord>> {
        let rows = sqlx::query_as!(
            DatabaseRow,
            "SELECT d.id, d.environment_id, e.project_id, d.name, d.slug, d.engine, d.version,
             d.database_name, d.username, d.password_enc, d.root_password_enc, d.external_port,
             d.memory_limit_mb, d.cpu_limit_millis, d.status, d.error_code, d.created_at, d.updated_at
             FROM databases d
             JOIN environments e ON e.id = d.environment_id
             JOIN projects p ON p.id = e.project_id
             WHERE p.organization_id = $1 AND e.project_id = $2
             ORDER BY d.name",
            organization_id.as_uuid(),
            project_id.as_uuid()
        )
        .fetch_all(&self.pool)
        .await
        .map_err(db_error)?;
        rows.into_iter().map(DatabaseRecord::try_from).collect()
    }

    async fn find(&self, organization_id: Id, id: Id) -> AppResult<Option<DatabaseRecord>> {
        let row = sqlx::query_as!(
            DatabaseRow,
            "SELECT d.id, d.environment_id, e.project_id, d.name, d.slug, d.engine, d.version,
             d.database_name, d.username, d.password_enc, d.root_password_enc, d.external_port,
             d.memory_limit_mb, d.cpu_limit_millis, d.status, d.error_code, d.created_at, d.updated_at
             FROM databases d
             JOIN environments e ON e.id = d.environment_id
             JOIN projects p ON p.id = e.project_id
             WHERE p.organization_id = $1 AND d.id = $2",
            organization_id.as_uuid(),
            id.as_uuid()
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(db_error)?;
        row.map(DatabaseRecord::try_from).transpose()
    }

    async fn set_external_port(&self, id: Id, port: Option<u16>) -> AppResult<()> {
        let updated = sqlx::query!(
            "UPDATE databases SET external_port = $2, updated_at = now()
             WHERE id = $1 AND ($2::integer IS NULL OR NOT EXISTS (
                 SELECT 1 FROM application_ports
                 WHERE published_port = $2 AND protocol = 'tcp'))",
            id.as_uuid(),
            port.map(i32::from)
        )
        .execute(&self.pool)
        .await
        .map_err(db_error)?;
        if updated.rows_affected() == 0 {
            return Err(AppError::conflict("ALREADY_EXISTS"));
        }
        Ok(())
    }

    async fn set_limits(&self, id: Id, limits: ResourceLimits) -> AppResult<()> {
        let memory = limits.memory_mb.map(i32::try_from).transpose();
        let cpu = limits.cpu_millis.map(i32::try_from).transpose();
        sqlx::query!(
            "UPDATE databases SET memory_limit_mb = $2, cpu_limit_millis = $3, updated_at = now()
             WHERE id = $1",
            id.as_uuid(),
            memory.map_err(AppError::internal)?,
            cpu.map_err(AppError::internal)?
        )
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(db_error)
    }

    async fn set_status(&self, id: Id, status: &str, error_code: Option<&str>) -> AppResult<()> {
        sqlx::query!(
            "UPDATE databases SET status = $2, error_code = $3, updated_at = now() WHERE id = $1",
            id.as_uuid(),
            status,
            error_code
        )
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(db_error)
    }

    async fn delete(&self, id: Id) -> AppResult<bool> {
        let deleted = sqlx::query!("DELETE FROM databases WHERE id = $1", id.as_uuid())
            .execute(&self.pool)
            .await
            .map_err(db_error)?;
        Ok(deleted.rows_affected() > 0)
    }
}
