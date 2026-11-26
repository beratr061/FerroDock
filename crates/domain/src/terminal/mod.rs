mod live;
mod registry;
mod replay;

use std::sync::Arc;

use ferrodock_core::{AppError, AppResult, Id};
use ferrodock_db::ApplicationRepo;
use ferrodock_infra::docker::{DockerEngine, ShellSize, ShellTarget};
use serde_json::json;
use tokio::io::AsyncWriteExt;
use uuid::Uuid;

use crate::audit::{Actor, AuditLog, AuditRecord};
use crate::auth::{Principal, Role};

pub use live::{Attachment, LiveSession, ShellExit, TerminalEvent};
pub use registry::MAX_SESSIONS_PER_USER;

use self::live::Owner;
use self::registry::{SessionRegistry, new_session_id};

#[derive(Clone)]
pub struct TerminalService {
    docker: DockerEngine,
    applications: Arc<dyn ApplicationRepo>,
    audit: AuditLog,
    sessions: SessionRegistry,
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
            sessions: SessionRegistry::default(),
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
    ) -> AppResult<Attachment> {
        let target = self.docker.shell_target(reference).await?;
        let application_id = self.authorize(actor, &target).await?;
        self.sessions.reserve(actor.principal.user_id)?;
        let shell = self.docker.open_shell(&target, size).await?;
        let owner = Owner {
            user_id: actor.principal.user_id,
            organization_id: actor.principal.organization_id,
        };
        let session = LiveSession::new(new_session_id(), owner, shell.exec_id, shell.input);
        let attachment = session.attach();
        tokio::spawn(session.clone().pump(shell.output, self.docker.clone()));
        self.sessions.insert(session);
        let record = AuditRecord {
            action: "terminal.opened",
            resource_type: "container",
            resource_id: application_id,
            metadata: json!({ "container": target.container_id }),
            ip_address: actor.ip_address,
        };
        self.audit.record(actor.principal, record).await;
        Ok(attachment)
    }

    pub fn resume(&self, principal: &Principal, session_id: &str) -> AppResult<Attachment> {
        let session =
            self.sessions
                .find(session_id, principal.user_id, principal.organization_id)?;
        Ok(session.attach())
    }

    pub async fn write(&self, session: &LiveSession, bytes: &[u8]) -> AppResult<()> {
        let mut input = session.input.lock().await;
        input.write_all(bytes).await.map_err(AppError::internal)?;
        input.flush().await.map_err(AppError::internal)
    }

    pub async fn resize(&self, session: &LiveSession, size: ShellSize) -> AppResult<()> {
        self.docker.resize_shell(&session.exec_id, size).await
    }
}
