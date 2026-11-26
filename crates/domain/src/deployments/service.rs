use std::sync::Arc;

use ferrodock_core::{AppError, AppResult, Id};
use ferrodock_db::deployments::{DEPLOYMENT_JOB, DeploymentKind, DeploymentRecord, NewDeployment};
use ferrodock_db::jobs::{CancelResult, JobStatus, LOCAL_QUEUE, NewJob};
use ferrodock_db::{ApplicationRepo, DeploymentRepo, JobQueue};
use ferrodock_infra::deploy_log::{DeployLogs, LogChunk};
use ferrodock_infra::realtime::{Hub, Signal};
use serde_json::{Value, json};

use crate::audit::{Actor, AuditLog, AuditRecord};
use crate::auth::{Principal, Role};

const HISTORY_LIMIT: i64 = 30;

struct Trigger {
    organization_id: Id,
    created_by: Option<Id>,
    application_id: Id,
}

#[derive(Clone)]
pub struct DeploymentRepos {
    pub deployments: Arc<dyn DeploymentRepo>,
    pub applications: Arc<dyn ApplicationRepo>,
    pub jobs: Arc<dyn JobQueue>,
}

impl std::fmt::Debug for DeploymentRepos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("DeploymentRepos")
    }
}

#[derive(Clone)]
pub struct DeploymentService {
    repos: DeploymentRepos,
    logs: DeployLogs,
    audit: AuditLog,
    hub: Hub,
}

impl std::fmt::Debug for DeploymentService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("DeploymentService")
    }
}

impl DeploymentService {
    pub fn new(repos: DeploymentRepos, logs: DeployLogs, audit: AuditLog, hub: Hub) -> Self {
        Self {
            repos,
            logs,
            audit,
            hub,
        }
    }

    fn announce(&self, organization_id: Id, deployment: &DeploymentRecord) {
        let signal = Signal::DeploymentStatus {
            application_id: deployment.application_id,
            deployment_id: deployment.id,
        };
        self.hub.publish(Some(organization_id), signal);
    }

    pub async fn list(
        &self,
        principal: &Principal,
        application_id: Id,
    ) -> AppResult<Vec<DeploymentRecord>> {
        self.owned_application(principal, application_id).await?;
        self.repos
            .deployments
            .list(application_id, HISTORY_LIMIT)
            .await
    }

    pub async fn get(&self, principal: &Principal, id: Id) -> AppResult<DeploymentRecord> {
        self.repos
            .deployments
            .find_in_organization(principal.organization_id, id)
            .await?
            .ok_or_else(|| AppError::not_found("DEPLOYMENT_NOT_FOUND"))
    }

    pub async fn log(
        &self,
        principal: &Principal,
        id: Id,
        offset: u64,
    ) -> AppResult<(DeploymentRecord, LogChunk)> {
        let deployment = self.get(principal, id).await?;
        let chunk = self.logs.read(deployment.id, offset).await?;
        Ok((deployment, chunk))
    }

    pub async fn deploy(
        &self,
        actor: Actor<'_>,
        application_id: Id,
    ) -> AppResult<DeploymentRecord> {
        actor.principal.require(Role::Developer)?;
        self.owned_application(actor.principal, application_id)
            .await?;
        self.enqueue(actor, application_id, DeploymentKind::Deploy, None)
            .await
    }

    pub async fn rollback(&self, actor: Actor<'_>, target_id: Id) -> AppResult<DeploymentRecord> {
        actor.principal.require(Role::Developer)?;
        let target = self.get(actor.principal, target_id).await?;
        let usable = target.status == JobStatus::Succeeded
            && target.image_id.is_some()
            && target
                .image_ref
                .as_deref()
                .is_some_and(|r| !r.starts_with("compose:"));
        if !usable {
            return Err(AppError::validation("ROLLBACK_TARGET_UNAVAILABLE"));
        }
        self.enqueue(
            actor,
            target.application_id,
            DeploymentKind::Rollback,
            Some(target.id),
        )
        .await
    }

    pub async fn cancel(&self, actor: Actor<'_>, id: Id) -> AppResult<DeploymentRecord> {
        actor.principal.require(Role::Developer)?;
        let deployment = self.get(actor.principal, id).await?;
        if self.repos.jobs.request_cancel(deployment.job_id).await? == CancelResult::NotCancellable
        {
            return Err(AppError::conflict("DEPLOYMENT_NOT_CANCELLABLE"));
        }
        let metadata = json!({ "application_id": deployment.application_id });
        self.audit
            .change(actor, "deployment.cancelled", id, metadata)
            .await;
        let deployment = self.get(actor.principal, id).await?;
        self.announce(actor.principal.organization_id, &deployment);
        Ok(deployment)
    }

    async fn enqueue(
        &self,
        actor: Actor<'_>,
        application_id: Id,
        kind: DeploymentKind,
        rollback_of: Option<Id>,
    ) -> AppResult<DeploymentRecord> {
        let trigger = Trigger {
            organization_id: actor.principal.organization_id,
            created_by: Some(actor.principal.user_id),
            application_id,
        };
        let record = self.queue(trigger, kind, rollback_of).await?;
        let metadata = json!({ "application_id": application_id, "kind": kind.as_str() });
        self.audit
            .change(actor, "deployment.created", record.id, metadata)
            .await;
        Ok(record)
    }

    pub async fn deploy_from_webhook(
        &self,
        organization_id: Id,
        application_id: Id,
        metadata: Value,
    ) -> AppResult<DeploymentRecord> {
        let trigger = Trigger {
            organization_id,
            created_by: None,
            application_id,
        };
        let record = self.queue(trigger, DeploymentKind::Deploy, None).await?;
        let entry = AuditRecord {
            action: "deployment.webhook",
            resource_type: "deployment",
            resource_id: Some(record.id),
            metadata,
            ip_address: None,
        };
        self.audit.system(organization_id, entry).await;
        Ok(record)
    }

    async fn queue(
        &self,
        trigger: Trigger,
        kind: DeploymentKind,
        rollback_of: Option<Id>,
    ) -> AppResult<DeploymentRecord> {
        let id = Id::new();
        let job = NewJob {
            organization_id: Some(trigger.organization_id),
            kind: DEPLOYMENT_JOB.to_owned(),
            queue: LOCAL_QUEUE.to_owned(),
            group_key: Some(trigger.application_id.to_string()),
            payload: json!({ "deployment_id": id }),
        };
        let deployment = NewDeployment {
            id,
            application_id: trigger.application_id,
            kind,
            rollback_of,
            created_by: trigger.created_by,
            job,
        };
        let record = self.repos.deployments.create(deployment).await?;
        self.announce(trigger.organization_id, &record);
        Ok(record)
    }

    async fn owned_application(&self, principal: &Principal, id: Id) -> AppResult<()> {
        self.repos
            .applications
            .find(principal.organization_id, id)
            .await?
            .map(|_| ())
            .ok_or_else(|| AppError::not_found("APPLICATION_NOT_FOUND"))
    }
}
