use std::sync::Arc;

use ferrodock_core::{AppError, AppResult, Id};
use ferrodock_db::ApplicationRepo;
use ferrodock_infra::docker::{DockerEngine, ShellSession, ShellSize, ShellTarget};
use serde_json::json;
use uuid::Uuid;

use crate::audit::{Actor, AuditLog, AuditRecord};
use crate::auth::Role;

#[derive(Clone)]
pub struct TerminalService {
    docker: DockerEngine,
    applications: Arc<dyn ApplicationRepo>,
    audit: AuditLog,
}

impl std::fmt::Debug for TerminalService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("TerminalService")
    }
}

impl TerminalService {
    pub fn new(
        docker: DockerEngine,
        applications: Arc<dyn ApplicationRepo>,
        audit: AuditLog,
    ) -> Self {
        Self {
            docker,
            applications,
            audit,
        }
    }

    async fn authorize(&self, actor: Actor<'_>, target: &ShellTarget) -> AppResult<Option<Id>> {
        let Some(label) = &target.application_id else {
            actor.principal.require(Role::Admin)?;
            return Ok(None);
        };
        actor.principal.require(Role::Developer)?;
        let not_found = || AppError::not_found("CONTAINER_NOT_FOUND");
        let id = Uuid::parse_str(label)
            .map(Id::from)
            .map_err(|_| not_found())?;
        self.applications
            .find(actor.principal.organization_id, id)
            .await?
            .ok_or_else(not_found)?;
        Ok(Some(id))
    }

    pub async fn open(
        &self,
        actor: Actor<'_>,
        reference: &str,
        size: ShellSize,
    ) -> AppResult<ShellSession> {
        let target = self.docker.shell_target(reference).await?;
        let application_id = self.authorize(actor, &target).await?;
        let session = self.docker.open_shell(&target, size).await?;
        let record = AuditRecord {
            action: "terminal.opened",
            resource_type: "container",
            resource_id: application_id,
            metadata: json!({ "container": target.container_id }),
            ip_address: actor.ip_address,
        };
        self.audit.record(actor.principal, record).await;
        Ok(session)
    }

    pub async fn resize(&self, exec_id: &str, size: ShellSize) -> AppResult<()> {
        self.docker.resize_shell(exec_id, size).await
    }

    pub async fn exit_code(&self, exec_id: &str) -> Option<i64> {
        self.docker.shell_exit_code(exec_id).await
    }
}
