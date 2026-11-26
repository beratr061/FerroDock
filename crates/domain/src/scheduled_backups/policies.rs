use ferrodock_core::AppResult;
use ferrodock_db::backups::{BackupTarget, DuePolicy, PolicyRecord};
use serde_json::json;
use time::OffsetDateTime;

use super::model::{Policy, PolicyInput};
use super::runs::RunRequest;
use super::service::ScheduledBackupService;
use super::targets::{describe, resource_id};
use super::{schedule, validate};
use crate::audit::Actor;
use crate::auth::{Principal, Role};

impl ScheduledBackupService {
    pub async fn policy(
        &self,
        principal: &Principal,
        target: &BackupTarget,
    ) -> AppResult<Option<Policy>> {
        self.authorize(principal, target).await?;
        let record = self.parts.policies.policy(target).await?;
        Ok(record.map(Into::into))
    }

    pub async fn set_policy(
        &self,
        actor: Actor<'_>,
        target: BackupTarget,
        input: PolicyInput,
    ) -> AppResult<Policy> {
        actor.principal.require(Role::Developer)?;
        self.authorize(actor.principal, &target).await?;
        self.destination(actor.principal, input.destination_id)
            .await?;
        let schedule = schedule::normalize(&input.schedule)?;
        let record = PolicyRecord {
            destination_id: input.destination_id,
            next_run_at: schedule::next_after(&schedule, OffsetDateTime::now_utc())?,
            retention: validate::retention(input.retention)?,
            enabled: input.enabled,
            schedule,
            target,
        };
        self.parts.policies.save_policy(record.clone()).await?;
        let mut metadata = describe(&record.target);
        metadata["schedule"] = json!(record.schedule);
        metadata["retention"] = json!(record.retention);
        metadata["enabled"] = json!(record.enabled);
        let resource = resource_id(&record.target);
        self.parts
            .audit
            .change(actor, "backup_policy.updated", resource, metadata)
            .await;
        Ok(record.into())
    }

    pub async fn remove_policy(&self, actor: Actor<'_>, target: &BackupTarget) -> AppResult<()> {
        actor.principal.require(Role::Developer)?;
        self.authorize(actor.principal, target).await?;
        self.parts.policies.remove_policy(target).await?;
        self.parts
            .audit
            .change(
                actor,
                "backup_policy.removed",
                resource_id(target),
                describe(target),
            )
            .await;
        Ok(())
    }

    pub async fn run_due(&self, now: OffsetDateTime) -> AppResult<usize> {
        let mut queued = 0;
        for due in self.parts.policies.due_policies(now).await? {
            if self.claim(&due, now).await? {
                let request = RunRequest::scheduled(&due);
                self.enqueue(request).await?;
                queued += 1;
            }
        }
        Ok(queued)
    }

    async fn claim(&self, due: &DuePolicy, now: OffsetDateTime) -> AppResult<bool> {
        let policy = &due.policy;
        let next = schedule::next_after(&policy.schedule, now)?;
        self.parts
            .policies
            .advance_policy(&policy.target, policy.next_run_at, next)
            .await
    }
}
