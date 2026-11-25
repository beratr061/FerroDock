use std::str::FromStr;

use ferrodock_core::events::DomainEvent;
use ferrodock_core::{AppError, AppResult, Id};
use ferrodock_db::records::{ApiKeyRecord, NewApiKey};
use serde::Serialize;
use time::{Duration, OffsetDateTime};
use zeroize::Zeroizing;

use super::model::{Principal, Role};
use super::service::{AuditContext, AuthService};
use super::tokens;
use crate::text::clean_name;

const MAX_EXPIRY_DAYS: u32 = 3650;
const USAGE_WRITE_INTERVAL: Duration = Duration::minutes(5);

#[derive(Debug)]
pub struct NewApiKeyInput {
    pub name: String,
    pub role: Role,
    pub expires_in_days: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ApiKeySummary {
    pub id: Id,
    pub name: String,
    pub prefix: String,
    pub role: Role,
    pub last_used_at: Option<OffsetDateTime>,
    pub expires_at: Option<OffsetDateTime>,
    pub created_at: OffsetDateTime,
}

#[derive(Debug)]
pub struct CreatedApiKey {
    pub key: Zeroizing<String>,
    pub summary: ApiKeySummary,
}

fn key_role(record: &ApiKeyRecord) -> AppResult<Role> {
    record
        .scopes
        .first()
        .map_or(Err(AppError::unauthorized()), |scope| Role::from_str(scope))
}

fn summarize(record: ApiKeyRecord) -> AppResult<ApiKeySummary> {
    Ok(ApiKeySummary {
        role: key_role(&record)?,
        id: record.id,
        name: record.name,
        prefix: record.prefix,
        last_used_at: record.last_used_at,
        expires_at: record.expires_at,
        created_at: record.created_at,
    })
}

impl AuthService {
    pub async fn create_api_key(
        &self,
        principal: &Principal,
        input: NewApiKeyInput,
    ) -> AppResult<CreatedApiKey> {
        if input.role > principal.role {
            return Err(AppError::forbidden().with_param("reason", "ROLE_EXCEEDS_MEMBERSHIP"));
        }
        let expires_at = match input.expires_in_days {
            Some(days) if days == 0 || days > MAX_EXPIRY_DAYS => {
                return Err(AppError::validation("API_KEY_EXPIRY_INVALID"));
            }
            Some(days) => Some(OffsetDateTime::now_utc() + Duration::days(i64::from(days))),
            None => None,
        };
        let key = tokens::new_api_key();
        let record = self
            .repos
            .api_keys
            .create(NewApiKey {
                id: Id::new(),
                user_id: principal.user_id,
                organization_id: principal.organization_id,
                name: clean_name(&input.name, "API_KEY_NAME_INVALID")?,
                prefix: tokens::display_prefix(&key),
                key_hash: tokens::digest(&key),
                scopes: vec![input.role.as_str().to_owned()],
                expires_at,
            })
            .await?;
        self.after_key_change(principal, record.id, "api_key.created")
            .await;
        Ok(CreatedApiKey {
            key,
            summary: summarize(record)?,
        })
    }

    pub async fn list_api_keys(&self, principal: &Principal) -> AppResult<Vec<ApiKeySummary>> {
        let records = self
            .repos
            .api_keys
            .list(principal.user_id, principal.organization_id)
            .await?;
        records.into_iter().map(summarize).collect()
    }

    pub async fn revoke_api_key(&self, principal: &Principal, id: Id) -> AppResult<()> {
        if !self.repos.api_keys.revoke(id, principal.user_id).await? {
            return Err(AppError::not_found("API_KEY_NOT_FOUND"));
        }
        self.after_key_change(principal, id, "api_key.revoked")
            .await;
        Ok(())
    }

    pub async fn authenticate_api_key(&self, raw_key: &str) -> AppResult<Principal> {
        if !raw_key.starts_with(tokens::API_KEY_PREFIX) {
            return Err(AppError::unauthorized());
        }
        let now = OffsetDateTime::now_utc();
        let record = self
            .repos
            .api_keys
            .find_active(&tokens::digest(raw_key), now)
            .await?
            .ok_or_else(AppError::unauthorized)?;
        let (organization_id, member_role) = self
            .role_in(record.user_id, Some(record.organization_id))
            .await?;
        let stale = record
            .last_used_at
            .is_none_or(|used| now - used > USAGE_WRITE_INTERVAL);
        if stale {
            self.repos.api_keys.mark_used(record.id, now).await?;
        }
        Ok(Principal {
            user_id: record.user_id,
            organization_id,
            role: member_role.min(key_role(&record)?),
            session_id: None,
            api_key_id: Some(record.id),
        })
    }

    async fn after_key_change(&self, principal: &Principal, key_id: Id, action: &'static str) {
        let event = if action == "api_key.created" {
            DomainEvent::ApiKeyCreated {
                api_key_id: key_id,
                user_id: principal.user_id,
            }
        } else {
            DomainEvent::ApiKeyRevoked {
                api_key_id: key_id,
                user_id: principal.user_id,
            }
        };
        self.events.publish(event);
        let context = AuditContext {
            organization_id: Some(principal.organization_id),
            actor_id: Some(principal.user_id),
            ip_address: None,
        };
        self.audit(&context, action, "api_key", Some(key_id)).await;
    }
}
