use std::sync::Arc;

use ferrodock_core::{AppError, AppResult, Id};
use ferrodock_db::applications::ApplicationRecord;
use ferrodock_db::runtime_config::RuntimeConfigRecord;
use ferrodock_db::{ApplicationRepo, RuntimeConfigRepo};
use serde_json::{Value, json};

use super::validate::{RuntimeConfigInput, validate};
use crate::audit::{Actor, AuditLog};
use crate::auth::{Principal, Role};
use crate::domains::is_compose;

#[derive(Clone)]
pub struct RuntimeConfigService {
    configs: Arc<dyn RuntimeConfigRepo>,
    applications: Arc<dyn ApplicationRepo>,
    audit: AuditLog,
}

impl std::fmt::Debug for RuntimeConfigService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("RuntimeConfigService")
    }
}

fn port_taken(error: AppError) -> AppError {
    if error.code() == "ALREADY_EXISTS" {
        AppError::conflict("RUNTIME_PORT_TAKEN")
    } else {
        error
    }
}

fn summary(config: &RuntimeConfigRecord) -> Value {
    let mounts: Vec<&str> = config.mounts.iter().map(|m| m.target.as_str()).collect();
    let ports: Vec<String> = config
        .ports
        .iter()
        .map(|p| format!("{}/{}", p.published, p.protocol))
        .collect();
    json!({ "mounts": mounts, "ports": ports })
}

impl RuntimeConfigService {
    pub fn new(
        repos: (Arc<dyn RuntimeConfigRepo>, Arc<dyn ApplicationRepo>),
        audit: AuditLog,
    ) -> Self {
        let (configs, applications) = repos;
        Self {
            configs,
            applications,
            audit,
        }
    }

    pub async fn get(
        &self,
        principal: &Principal,
        application_id: Id,
    ) -> AppResult<RuntimeConfigRecord> {
        self.owned(principal, application_id).await?;
        self.configs.runtime_config(application_id).await
    }

    pub async fn replace(
        &self,
        actor: Actor<'_>,
        application_id: Id,
        input: RuntimeConfigInput,
    ) -> AppResult<RuntimeConfigRecord> {
        actor.principal.require(Role::Developer)?;
        let application = self.owned(actor.principal, application_id).await?;
        let config = validate(&input)?;
        let empty = config.mounts.is_empty() && config.ports.is_empty();
        if is_compose(&application) && !empty {
            return Err(AppError::validation("RUNTIME_COMPOSE_UNSUPPORTED"));
        }
        if config.mounts.iter().any(|mount| mount.kind == "bind") {
            actor.principal.require(Role::Admin)?;
        }
        self.configs
            .replace_runtime_config(application_id, config.clone())
            .await
            .map_err(port_taken)?;
        self.audit
            .change(actor, "runtime.updated", application_id, summary(&config))
            .await;
        Ok(config)
    }

    async fn owned(&self, principal: &Principal, id: Id) -> AppResult<ApplicationRecord> {
        self.applications
            .find(principal.organization_id, id)
            .await?
            .ok_or_else(|| AppError::not_found("APPLICATION_NOT_FOUND"))
    }
}
