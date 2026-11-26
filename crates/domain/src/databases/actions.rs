use ferrodock_core::{AppError, AppResult, Id};
use ferrodock_db::databases::ResourceLimits;
use ferrodock_infra::deploy_log::LogChunk;
use ferrodock_infra::docker::{LogQuery, LogStream};
use serde_json::json;

use super::drivers::{connection_url, driver};
use super::model::{Database, DatabaseAction, DatabaseCredentials, DatabaseStatus};
use super::service::DatabaseService;
use super::spec::{credentials, reveal};
use super::validate;
use crate::audit::Actor;
use crate::auth::{Principal, Role};

fn port_taken(error: AppError) -> AppError {
    if error.code() == "ALREADY_EXISTS" {
        AppError::conflict("DATABASE_PORT_TAKEN")
    } else {
        error
    }
}

impl DatabaseService {
    pub async fn act(
        &self,
        actor: Actor<'_>,
        id: Id,
        action: DatabaseAction,
    ) -> AppResult<Database> {
        actor.principal.require(Role::Developer)?;
        let record = self.owned(actor.principal, id).await?;
        let containers = self
            .parts
            .docker
            .database_containers(Some(&id.to_string()))
            .await?;
        let docker = &self.parts.docker;
        let (status, audit_action) = match action {
            DatabaseAction::Deploy => {
                self.enqueue_deploy(actor.principal, id).await?;
                (DatabaseStatus::Deploying, "database.deployed")
            }
            DatabaseAction::Start if containers.is_empty() => {
                self.enqueue_deploy(actor.principal, id).await?;
                (DatabaseStatus::Deploying, "database.started")
            }
            DatabaseAction::Start => {
                for container in &containers {
                    docker.start_workload(&container.id).await?;
                }
                (DatabaseStatus::Running, "database.started")
            }
            DatabaseAction::Stop => {
                for container in &containers {
                    docker.stop_workload(&container.id).await?;
                }
                (DatabaseStatus::Stopped, "database.stopped")
            }
        };
        self.parts
            .repo
            .set_status(id, status.as_str(), None)
            .await?;
        self.changed(actor, audit_action, &record).await;
        self.get(actor.principal, id).await
    }

    pub async fn set_external_port(
        &self,
        actor: Actor<'_>,
        id: Id,
        port: Option<u16>,
    ) -> AppResult<Database> {
        actor.principal.require(Role::Developer)?;
        let record = self.owned(actor.principal, id).await?;
        let port = validate::external_port(port)?;
        if record.external_port != port {
            self.parts
                .repo
                .set_external_port(id, port)
                .await
                .map_err(port_taken)?;
            self.changed(actor, "database.port_changed", &record).await;
            self.enqueue_deploy(actor.principal, id).await?;
        }
        self.get(actor.principal, id).await
    }

    pub async fn set_limits(
        &self,
        actor: Actor<'_>,
        id: Id,
        limits: ResourceLimits,
    ) -> AppResult<Database> {
        actor.principal.require(Role::Developer)?;
        let record = self.owned(actor.principal, id).await?;
        let limits = validate::limits(limits)?;
        if record.limits != limits {
            self.parts.repo.set_limits(id, limits).await?;
            self.changed(actor, "database.limits_changed", &record)
                .await;
            self.enqueue_deploy(actor.principal, id).await?;
        }
        self.get(actor.principal, id).await
    }

    pub async fn credentials(&self, actor: Actor<'_>, id: Id) -> AppResult<DatabaseCredentials> {
        actor.principal.require(Role::Developer)?;
        let record = self.owned(actor.principal, id).await?;
        let engine = driver(&record.engine)?;
        let opened = reveal(&self.parts.secrets, &record)?;
        let login = credentials(&record, &opened);
        let database = Database::from_record(record.clone(), false)?;
        let external_url = record.external_port.zip(self.parts.public_host.as_deref());
        let result = DatabaseCredentials {
            username: record.username.clone(),
            password: opened.password.to_string(),
            root_password: opened.root_password.as_deref().cloned(),
            internal_url: connection_url(
                engine,
                &login,
                &database.internal_host,
                database.internal_port,
            ),
            external_url: external_url
                .map(|(port, host)| connection_url(engine, &login, host, port)),
        };
        self.parts
            .audit
            .change(actor, "database.credentials_viewed", id, json!({}))
            .await;
        Ok(result)
    }

    pub async fn deploy_log(
        &self,
        principal: &Principal,
        id: Id,
        offset: u64,
    ) -> AppResult<LogChunk> {
        self.owned(principal, id).await?;
        self.parts.logs.read(id, offset).await
    }

    pub async fn container_logs(
        &self,
        principal: &Principal,
        id: Id,
        tail: u32,
    ) -> AppResult<LogStream> {
        principal.require(Role::Developer)?;
        self.owned(principal, id).await?;
        let container = self
            .parts
            .docker
            .database_containers(Some(&id.to_string()))
            .await?
            .into_iter()
            .next()
            .ok_or_else(|| AppError::not_found("CONTAINER_NOT_FOUND"))?;
        self.parts
            .docker
            .container_logs(&container.id, LogQuery { tail, follow: true })
    }
}
