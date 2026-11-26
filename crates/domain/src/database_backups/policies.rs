use ferrodock_core::{AppResult, Id};
use ferrodock_db::backups::{DuePolicy, PolicyRecord};
use serde_json::json;
use time::OffsetDateTime;

use super::model::{Policy, PolicyInput};
use super::runs::RunRequest;
use super::service::DatabaseBackupService;
use super::{schedule, validate};
use crate::audit::Actor;
use crate::auth::{Principal, Role};

impl DatabaseBackupService {
    pub async fn policy(
        &self,
        principal: &Principal,
        database_id: Id,
    ) -> AppResult<Option<Policy>> {
        self.database(principal, database_id).await?;
        let record = self.parts.policies.policy(database_id).await?;
        Ok(record.map(Into::into))
    }

    pub async fn set_policy(
        &self,
        actor: Actor<'_>,
        database_id: Id,
        input: PolicyInput,
    ) -> AppResult<Policy> {
        actor.principal.require(Role::Developer)?;
        self.database(actor.principal, database_id).await?;
        self.destination(actor.principal, input.destination_id)
            .await?;
        let schedule = schedule::normalize(&input.schedule)?;
        let record = PolicyRecord {
            database_id,
            destination_id: input.destination_id,
            next_run_at: schedule::next_after(&schedule, OffsetDateTime::now_utc())?,
            retention: validate::retention(input.retention)?,
            enabled: input.enabled,
            schedule,
        };
        self.parts.policies.save_policy(record.clone()).await?;
        let metadata = json!({ "schedule": record.schedule, "retention": record.retention, "enabled": record.enabled });
        self.parts
            .audit
            .change(actor, "backup_policy.updated", database_id, metadata)
            .await;
        Ok(record.into())
    }

    pub async fn remove_policy(&self, actor: Actor<'_>, database_id: Id) -> AppResult<()> {
        actor.principal.require(Role::Developer)?;
        self.database(actor.principal, database_id).await?;
        self.parts.policies.remove_policy(database_id).await?;
        self.parts
            .audit
            .change(actor, "backup_policy.removed", database_id, json!({}))
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
            .advance_policy(policy.database_id, policy.next_run_at, next)
            .await
    }
}
