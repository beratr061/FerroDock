use std::str::FromStr;
use std::sync::Arc;

use ferrodock_core::events::{DomainEvent, EventBus};
use ferrodock_core::{AppError, AppResult, Id, Secret};
use ferrodock_db::records::{NewAuditEntry, NewOwner};
use ferrodock_db::{ApiKeyRepo, AuditRepo, IdentityRepo, LoginAttemptRepo, SessionRepo};
use serde_json::Value;
use subtle::ConstantTimeEq;
use time::Duration;

use super::input::{SetupInput, clean_locale, normalize_email};
use super::model::{CurrentUser, Membership, Principal, Role};
use super::password;
use super::sessions::SignedIn;
use crate::text::{clean_name, slugify};

#[derive(Clone)]
pub struct AuthRepos {
    pub identity: Arc<dyn IdentityRepo>,
    pub sessions: Arc<dyn SessionRepo>,
    pub api_keys: Arc<dyn ApiKeyRepo>,
    pub audit: Arc<dyn AuditRepo>,
    pub login_attempts: Arc<dyn LoginAttemptRepo>,
}

impl std::fmt::Debug for AuthRepos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("AuthRepos")
    }
}

#[derive(Clone)]
pub struct AuthService {
    pub(super) repos: AuthRepos,
    pub(super) events: Arc<dyn EventBus>,
    pub(super) session_ttl: Duration,
    pub(super) setup_token: Option<Secret<String>>,
}

impl std::fmt::Debug for AuthService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AuthService")
            .field("session_ttl", &self.session_ttl)
            .finish_non_exhaustive()
    }
}

pub(super) struct AuditContext<'a> {
    pub organization_id: Option<Id>,
    pub actor_id: Option<Id>,
    pub ip_address: Option<&'a str>,
}

impl AuthService {
    pub fn new(repos: AuthRepos, events: Arc<dyn EventBus>, session_ttl: Duration) -> Self {
        Self {
            repos,
            events,
            session_ttl,
            setup_token: None,
        }
    }

    #[must_use]
    pub fn with_setup_token(mut self, token: Option<Secret<String>>) -> Self {
        self.setup_token = token.filter(|value| !value.expose().is_empty());
        self
    }

    pub fn setup_token_required(&self) -> bool {
        self.setup_token.is_some()
    }

    fn verify_setup_token(&self, provided: Option<&str>) -> AppResult<()> {
        let Some(expected) = &self.setup_token else {
            return Ok(());
        };
        let matches = provided
            .is_some_and(|value| bool::from(value.as_bytes().ct_eq(expected.expose().as_bytes())));
        if matches {
            Ok(())
        } else {
            Err(AppError::forbidden().with_param("reason", "SETUP_TOKEN_INVALID"))
        }
    }

    pub async fn setup_required(&self) -> AppResult<bool> {
        Ok(!self.repos.identity.has_users().await?)
    }

    pub async fn complete_setup(&self, input: SetupInput) -> AppResult<SignedIn> {
        self.verify_setup_token(input.setup_token.as_deref().map(String::as_str))?;
        password::validate_strength(&input.password)?;
        let organization_name = clean_name(&input.organization_name, "ORGANIZATION_NAME_INVALID")?;
        let owner = NewOwner {
            user_id: Id::new(),
            email: input.email.trim().to_owned(),
            email_normalized: normalize_email(&input.email)?,
            name: clean_name(&input.name, "NAME_INVALID")?,
            password_hash: password::hash(input.password).await?,
            locale: clean_locale(&input.locale),
            organization_id: Id::new(),
            organization_slug: slugify(&organization_name),
            organization_name,
        };
        let (user, organization) = self.repos.identity.create_owner(owner).await?;
        self.events.publish(DomainEvent::OrganizationCreated {
            organization_id: organization.id,
            owner_id: user.id,
        });
        let context = AuditContext {
            organization_id: Some(organization.id),
            actor_id: Some(user.id),
            ip_address: input.ip_address.as_deref(),
        };
        self.audit(
            &context,
            "setup.completed",
            "organization",
            Some(organization.id),
        )
        .await;
        self.start_session(user.id, organization.id, input.ip_address, input.user_agent)
            .await
    }

    pub async fn current_user(&self, principal: &Principal) -> AppResult<CurrentUser> {
        self.load_current_user(principal.user_id, principal.organization_id)
            .await
    }

    pub(super) async fn load_current_user(
        &self,
        user_id: Id,
        organization_id: Id,
    ) -> AppResult<CurrentUser> {
        let user = self
            .repos
            .identity
            .find_user(user_id)
            .await?
            .ok_or_else(AppError::unauthorized)?;
        Ok(CurrentUser {
            id: user.id,
            email: user.email,
            name: user.name,
            locale: user.locale,
            theme: user.theme,
            active_organization_id: organization_id,
            memberships: self.memberships(user_id).await?,
        })
    }

    pub(super) async fn memberships(&self, user_id: Id) -> AppResult<Vec<Membership>> {
        self.repos
            .identity
            .memberships(user_id)
            .await?
            .into_iter()
            .map(|record| {
                Ok(Membership {
                    organization_id: record.organization.id,
                    organization_name: record.organization.name,
                    organization_slug: record.organization.slug,
                    role: Role::from_str(&record.role)?,
                })
            })
            .collect()
    }

    pub(super) async fn role_in(
        &self,
        user_id: Id,
        organization_id: Option<Id>,
    ) -> AppResult<(Id, Role)> {
        let memberships = self.memberships(user_id).await?;
        let chosen = match organization_id {
            Some(id) => memberships.into_iter().find(|m| m.organization_id == id),
            None => memberships.into_iter().next(),
        };
        chosen
            .map(|m| (m.organization_id, m.role))
            .ok_or_else(AppError::unauthorized)
    }

    pub(super) async fn audit(
        &self,
        context: &AuditContext<'_>,
        action: &'static str,
        resource_type: &'static str,
        resource_id: Option<Id>,
    ) {
        let entry = NewAuditEntry {
            organization_id: context.organization_id,
            actor_id: context.actor_id,
            action,
            resource_type,
            resource_id,
            metadata: Value::Object(serde_json::Map::new()),
            ip_address: context.ip_address.map(str::to_owned),
        };
        if let Err(error) = self.repos.audit.record(entry).await {
            tracing::warn!(code = error.code(), action, "audit write failed");
        }
    }
}
