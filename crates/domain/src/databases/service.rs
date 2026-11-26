use std::collections::HashSet;
use std::sync::Arc;

use ferrodock_core::crypto::SecretBox;
use ferrodock_core::{AppError, AppResult, Id};
use ferrodock_db::databases::{DATABASE_DEPLOY_JOB, DatabaseRecord, NewDatabase};
use ferrodock_db::jobs::{LOCAL_QUEUE, NewJob};
use ferrodock_db::{DatabaseRepo, JobQueue};
use ferrodock_infra::deploy_log::DeployLogs;
use ferrodock_infra::docker::DockerEngine;
use serde_json::json;

use super::drivers::driver;
use super::model::{Database, DatabaseStatus, NewDatabaseInput};
use super::spec::volume;
use super::validate::{self, random_password};
use crate::audit::{Actor, AuditLog};
use crate::auth::{Principal, Role};
use crate::text::{clean_name, slugify};

pub struct DatabaseServiceParts {
    pub repo: Arc<dyn DatabaseRepo>,
    pub jobs: Arc<dyn JobQueue>,
    pub docker: DockerEngine,
    pub secrets: SecretBox,
    pub logs: DeployLogs,
    pub audit: AuditLog,
    pub public_host: Option<String>,
}

#[derive(Clone)]
pub struct DatabaseService {
    pub(super) parts: Arc<DatabaseServiceParts>,
}

impl std::fmt::Debug for DatabaseServiceParts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("DatabaseServiceParts")
    }
}

impl std::fmt::Debug for DatabaseService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("DatabaseService")
    }
}

fn already_exists(error: AppError) -> AppError {
    if error.code() == "ALREADY_EXISTS" {
        AppError::conflict("DATABASE_EXISTS")
    } else {
        error
    }
}

impl DatabaseService {
    pub fn new(parts: DatabaseServiceParts) -> Self {
        Self {
            parts: Arc::new(parts),
        }
    }

    pub async fn list(&self, principal: &Principal, project_id: Id) -> AppResult<Vec<Database>> {
        let records = self
            .parts
            .repo
            .list_for_project(principal.organization_id, project_id)
            .await?;
        let running: HashSet<String> = match self.parts.docker.database_containers(None).await {
            Ok(containers) => containers
                .into_iter()
                .filter(|container| container.running)
                .map(|container| container.database_id)
                .collect(),
            Err(error) => {
                tracing::warn!(code = error.code(), "database containers unavailable");
                HashSet::new()
            }
        };
        records
            .into_iter()
            .map(|record| {
                let up = running.contains(&record.id.to_string());
                Database::from_record(record, up)
            })
            .collect()
    }

    pub async fn get(&self, principal: &Principal, id: Id) -> AppResult<Database> {
        let record = self.owned(principal, id).await?;
        self.describe(record).await
    }

    pub async fn create(
        &self,
        actor: Actor<'_>,
        environment_id: Id,
        input: NewDatabaseInput,
    ) -> AppResult<Database> {
        actor.principal.require(Role::Developer)?;
        let engine = driver(&input.engine)?;
        let name = clean_name(&input.name, "DATABASE_NAME_INVALID")?;
        let slug = slugify(&name);
        let secrets = &self.parts.secrets;
        let root_password_enc = if engine.needs_root_password() {
            Some(secrets.encrypt(random_password().as_bytes())?)
        } else {
            None
        };
        let database = NewDatabase {
            id: Id::new(),
            organization_id: actor.principal.organization_id,
            environment_id,
            version: validate::version(engine, input.version.as_deref())?,
            database_name: validate::database_name(input.database_name.as_deref(), &slug)?,
            username: validate::username(engine, input.username.as_deref())?,
            password_enc: secrets.encrypt(random_password().as_bytes())?,
            root_password_enc,
            engine: engine.engine().to_owned(),
            name,
            slug,
        };
        let record = self
            .parts
            .repo
            .create(database)
            .await
            .map_err(already_exists)?
            .ok_or_else(|| AppError::not_found("ENVIRONMENT_NOT_FOUND"))?;
        self.changed(actor, "database.created", &record).await;
        self.enqueue_deploy(actor.principal, record.id).await?;
        self.get(actor.principal, record.id).await
    }

    pub async fn delete(&self, actor: Actor<'_>, id: Id, remove_data: bool) -> AppResult<()> {
        actor.principal.require(Role::Admin)?;
        let record = self.owned(actor.principal, id).await?;
        let docker = &self.parts.docker;
        for container in docker.database_containers(Some(&id.to_string())).await? {
            docker.remove_workload(&container.id).await?;
        }
        if remove_data {
            docker.remove_volume(&volume(id)).await?;
        }
        if !self.parts.repo.delete(id).await? {
            return Err(AppError::not_found("DATABASE_NOT_FOUND"));
        }
        self.parts.logs.remove(id).await;
        let metadata = json!({ "slug": record.slug, "data_removed": remove_data });
        self.parts
            .audit
            .change(actor, "database.deleted", id, metadata)
            .await;
        Ok(())
    }

    pub(super) async fn enqueue_deploy(&self, principal: &Principal, id: Id) -> AppResult<()> {
        let job = NewJob {
            organization_id: Some(principal.organization_id),
            kind: DATABASE_DEPLOY_JOB.to_owned(),
            queue: LOCAL_QUEUE.to_owned(),
            group_key: Some(format!("database:{id}")),
            payload: json!({ "database_id": id }),
        };
        self.parts.jobs.enqueue(job).await?;
        let status = DatabaseStatus::Deploying.as_str();
        self.parts.repo.set_status(id, status, None).await
    }

    pub(super) async fn describe(&self, record: DatabaseRecord) -> AppResult<Database> {
        let containers = self
            .parts
            .docker
            .database_containers(Some(&record.id.to_string()))
            .await
            .unwrap_or_default();
        let running = containers.iter().any(|container| container.running);
        Database::from_record(record, running)
    }

    pub(super) async fn owned(&self, principal: &Principal, id: Id) -> AppResult<DatabaseRecord> {
        self.parts
            .repo
            .find(principal.organization_id, id)
            .await?
            .ok_or_else(|| AppError::not_found("DATABASE_NOT_FOUND"))
    }

    pub(super) async fn changed(
        &self,
        actor: Actor<'_>,
        action: &'static str,
        record: &DatabaseRecord,
    ) {
        let metadata = json!({
            "environment_id": record.environment_id,
            "slug": record.slug,
            "engine": record.engine,
        });
        self.parts
            .audit
            .change(actor, action, record.id, metadata)
            .await;
    }
}
