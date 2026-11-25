use ferrodock_db::records::NewAuditEntry;
use serde_json::Value;

use super::model::Principal;
use super::service::AuthService;

#[derive(Debug)]
pub struct AuditRecord<'a> {
    pub action: &'static str,
    pub resource_type: &'static str,
    pub metadata: Value,
    pub ip_address: Option<&'a str>,
}

impl AuthService {
    pub async fn record_action(&self, principal: &Principal, record: AuditRecord<'_>) {
        let entry = NewAuditEntry {
            organization_id: Some(principal.organization_id),
            actor_id: Some(principal.user_id),
            action: record.action,
            resource_type: record.resource_type,
            resource_id: None,
            metadata: record.metadata,
            ip_address: record.ip_address.map(str::to_owned),
        };
        if let Err(error) = self.repos.audit.record(entry).await {
            tracing::warn!(
                code = error.code(),
                action = record.action,
                "audit write failed"
            );
        }
    }
}
