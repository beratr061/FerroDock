use async_trait::async_trait;
use ferrodock_core::{AppError, AppResult, Id};

use super::application_rows::{ApplicationRow, into_records, source_columns};
use super::{PgStore, db_error};
use crate::applications::{ApplicationFields, ApplicationRecord, ApplicationRepo, NewApplication};

#[async_trait]
impl ApplicationRepo for PgStore {
    async fn environment_project(
        &self,
        organization_id: Id,
        environment_id: Id,
    ) -> AppResult<Option<Id>> {
        sqlx::query_scalar!(
            "SELECT e.project_id FROM environments e JOIN projects p ON p.id = e.project_id
             WHERE e.id = $1 AND p.organization_id = $2",
            environment_id.as_uuid(),
            organization_id.as_uuid()
        )
        .fetch_optional(&self.pool)
        .await
        .map(|id| id.map(Id::from))
        .map_err(db_error)
    }

    async fn create(&self, application: NewApplication) -> AppResult<ApplicationRecord> {
        let fields = &application.fields;
        let source = source_columns(&fields.source);
        let key = fields.deploy_key.as_ref();
        sqlx::query!(
            "INSERT INTO applications (id, environment_id, name, slug, description, source_kind,
             git_url, git_branch, build_kind, build_path, build_context, image,
             deploy_key_public, deploy_key_private_enc, strategy)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)",
            application.id.as_uuid(),
            application.environment_id.as_uuid(),
            fields.name,
            application.slug,
            fields.description,
            source.kind,
            source.git_url,
            source.git_branch,
            source.build_kind,
            source.build_path,
            source.build_context,
            source.image,
            key.map(|key| key.public.as_str()),
            key.map(|key| key.private_enc.as_str()),
            fields.strategy
        )
        .execute(&self.pool)
        .await
        .map_err(db_error)?;
        self.load(application.id).await
    }

    async fn list_for_project(
        &self,
        organization_id: Id,
        project_id: Id,
    ) -> AppResult<Vec<ApplicationRecord>> {
        let rows = sqlx::query_as!(
            ApplicationRow,
            "SELECT a.id, a.environment_id, e.project_id, a.name, a.slug, a.description,
             a.source_kind, a.git_url, a.git_branch, a.build_kind, a.build_path, a.build_context,
             a.image, a.deploy_key_public, a.deploy_key_private_enc, a.strategy,
             a.active_container, a.compose_trusted, a.created_at, a.updated_at
             FROM applications a
             JOIN environments e ON e.id = a.environment_id
             JOIN projects p ON p.id = e.project_id
             WHERE p.organization_id = $1 AND p.id = $2
             ORDER BY lower(a.name), a.id",
            organization_id.as_uuid(),
            project_id.as_uuid()
        )
        .fetch_all(&self.pool)
        .await
        .map_err(db_error)?;
        into_records(rows)
    }

    async fn find(&self, organization_id: Id, id: Id) -> AppResult<Option<ApplicationRecord>> {
        let row = sqlx::query_as!(
            ApplicationRow,
            "SELECT a.id, a.environment_id, e.project_id, a.name, a.slug, a.description,
             a.source_kind, a.git_url, a.git_branch, a.build_kind, a.build_path, a.build_context,
             a.image, a.deploy_key_public, a.deploy_key_private_enc, a.strategy,
             a.active_container, a.compose_trusted, a.created_at, a.updated_at
             FROM applications a
             JOIN environments e ON e.id = a.environment_id
             JOIN projects p ON p.id = e.project_id
             WHERE p.organization_id = $1 AND a.id = $2",
            organization_id.as_uuid(),
            id.as_uuid()
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(db_error)?;
        row.map(ApplicationRecord::try_from).transpose()
    }

    async fn update(&self, id: Id, fields: ApplicationFields) -> AppResult<ApplicationRecord> {
        let source = source_columns(&fields.source);
        let key = fields.deploy_key.as_ref();
        sqlx::query!(
            "UPDATE applications SET name = $2, description = $3, source_kind = $4, git_url = $5,
             git_branch = $6, build_kind = $7, build_path = $8, build_context = $9, image = $10,
             deploy_key_public = $11, deploy_key_private_enc = $12, strategy = $13,
             updated_at = now()
             WHERE id = $1",
            id.as_uuid(),
            fields.name,
            fields.description,
            source.kind,
            source.git_url,
            source.git_branch,
            source.build_kind,
            source.build_path,
            source.build_context,
            source.image,
            key.map(|key| key.public.as_str()),
            key.map(|key| key.private_enc.as_str()),
            fields.strategy
        )
        .execute(&self.pool)
        .await
        .map_err(db_error)?;
        self.load(id).await
    }

    async fn set_active_container(&self, id: Id, container: Option<&str>) -> AppResult<()> {
        sqlx::query!(
            "UPDATE applications SET active_container = $2 WHERE id = $1",
            id.as_uuid(),
            container
        )
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(db_error)
    }

    async fn set_compose_trusted(&self, id: Id, trusted: bool) -> AppResult<()> {
        sqlx::query!(
            "UPDATE applications SET compose_trusted = $2, updated_at = now() WHERE id = $1",
            id.as_uuid(),
            trusted
        )
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(db_error)
    }

    async fn delete(&self, id: Id) -> AppResult<bool> {
        sqlx::query!("DELETE FROM applications WHERE id = $1", id.as_uuid())
            .execute(&self.pool)
            .await
            .map(|result| result.rows_affected() > 0)
            .map_err(db_error)
    }
}

impl PgStore {
    async fn load(&self, id: Id) -> AppResult<ApplicationRecord> {
        let row = sqlx::query_as!(
            ApplicationRow,
            "SELECT a.id, a.environment_id, e.project_id, a.name, a.slug, a.description,
             a.source_kind, a.git_url, a.git_branch, a.build_kind, a.build_path, a.build_context,
             a.image, a.deploy_key_public, a.deploy_key_private_enc, a.strategy,
             a.active_container, a.compose_trusted, a.created_at, a.updated_at
             FROM applications a JOIN environments e ON e.id = a.environment_id
             WHERE a.id = $1",
            id.as_uuid()
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(db_error)?
        .ok_or_else(|| AppError::not_found("APPLICATION_NOT_FOUND"))?;
        ApplicationRecord::try_from(row)
    }
}
