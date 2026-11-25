use serde::Serialize;

use crate::Id;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DomainEvent {
    OrganizationCreated { organization_id: Id, owner_id: Id },
    UserSignedIn { user_id: Id },
    UserSignedOut { user_id: Id },
    ApiKeyCreated { api_key_id: Id, user_id: Id },
    ApiKeyRevoked { api_key_id: Id, user_id: Id },
}

pub trait EventBus: Send + Sync + 'static {
    fn publish(&self, event: DomainEvent);
}
