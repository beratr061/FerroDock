use std::sync::Arc;

use ferrodock_core::Id;
use ferrodock_db::AuditRepo;
use ferrodock_db::records::NewAuditEntry;
use serde_json::Value;

use crate::auth::Principal;

#[derive(Debug, Clone, Copy)]
pub struct Actor<'a> {
    pub principal: &'a Principal,
    pub ip_address: Option<&'a str>,
}

#[derive(Debug)]
pub struct AuditRecord<'a> {
    pub action: &'static str,
    pub resource_type: &'static str,
    pub resource_id: Option<Id>,
    pub metadata: Value,
    pub ip_address: Option<&'a str>,
}

#[derive(Clone)]
pub struct AuditLog {
    repo: Arc<dyn AuditRepo>,
}

impl std::fmt::Debug for AuditLog {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("AuditLog")
    }
}

impl AuditLog {
    pub fn new(repo: Arc<dyn AuditRepo>) -> Self {
        Self { repo }
    }

    pub async fn change(
        &self,
        actor: Actor<'_>,
        action: &'static str,
        resource_id: Id,
        metadata: Value,
    ) {
        let resource_type = action.split_once('.').map_or(action, |(kind, _)| kind);
        let record = AuditRecord {
            action,
            resource_type,
            resource_id: Some(resource_id),
            metadata,
            ip_address: actor.ip_address,
        };
        self.record(actor.principal, record).await;
    }

    pub async fn record(&self, principal: &Principal, record: AuditRecord<'_>) {
        let entry = NewAuditEntry {
            organization_id: Some(principal.organization_id),
            actor_id: Some(principal.user_id),
            action: record.action,
            resource_type: record.resource_type,
            resource_id: record.resource_id,
            metadata: record.metadata,
            ip_address: record.ip_address.map(str::to_owned),
        };
        if let Err(error) = self.repo.record(entry).await {
            tracing::warn!(
                code = error.code(),
                action = record.action,
                "audit write failed"
            );
        }
    }
}
