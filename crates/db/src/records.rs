use ferrodock_core::Id;
use serde_json::Value;
use time::OffsetDateTime;

#[derive(Debug, Clone)]
pub struct UserRecord {
    pub id: Id,
    pub email: String,
    pub name: String,
    pub password_hash: String,
    pub locale: String,
    pub theme: String,
}

#[derive(Debug, Clone)]
pub struct OrganizationRecord {
    pub id: Id,
    pub name: String,
    pub slug: String,
}

#[derive(Debug, Clone)]
pub struct MembershipRecord {
    pub organization: OrganizationRecord,
    pub role: String,
}

#[derive(Debug)]
pub struct NewOwner {
    pub user_id: Id,
    pub email: String,
    pub email_normalized: String,
    pub name: String,
    pub password_hash: String,
    pub locale: String,
    pub organization_id: Id,
    pub organization_name: String,
    pub organization_slug: String,
}

#[derive(Debug)]
pub struct NewSession {
    pub id: Id,
    pub user_id: Id,
    pub token_hash: Vec<u8>,
    pub active_organization_id: Option<Id>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub expires_at: OffsetDateTime,
}

#[derive(Debug, Clone)]
pub struct SessionRecord {
    pub id: Id,
    pub user_id: Id,
    pub active_organization_id: Option<Id>,
    pub expires_at: OffsetDateTime,
    pub last_seen_at: OffsetDateTime,
}

#[derive(Debug)]
pub struct NewApiKey {
    pub id: Id,
    pub user_id: Id,
    pub organization_id: Id,
    pub name: String,
    pub prefix: String,
    pub key_hash: Vec<u8>,
    pub scopes: Vec<String>,
    pub expires_at: Option<OffsetDateTime>,
}

#[derive(Debug, Clone)]
pub struct ApiKeyRecord {
    pub id: Id,
    pub user_id: Id,
    pub organization_id: Id,
    pub name: String,
    pub prefix: String,
    pub scopes: Vec<String>,
    pub last_used_at: Option<OffsetDateTime>,
    pub expires_at: Option<OffsetDateTime>,
    pub created_at: OffsetDateTime,
}

#[derive(Debug)]
pub struct FailedAttempt {
    pub user_id: Id,
    pub source: String,
    pub now: OffsetDateTime,
    pub window_start: OffsetDateTime,
    pub max_failures: i32,
    pub lock_until: OffsetDateTime,
}

#[derive(Debug)]
pub struct NewAuditEntry {
    pub organization_id: Option<Id>,
    pub actor_id: Option<Id>,
    pub action: &'static str,
    pub resource_type: &'static str,
    pub resource_id: Option<Id>,
    pub metadata: Value,
    pub ip_address: Option<String>,
}
