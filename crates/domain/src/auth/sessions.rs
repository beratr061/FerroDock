use ferrodock_core::events::DomainEvent;
use ferrodock_core::{AppError, AppResult, Id};
use ferrodock_db::records::{NewSession, UserRecord};
use time::{Duration, OffsetDateTime};
use zeroize::Zeroizing;

use super::input::{SignInInput, normalize_email};
use super::model::{CurrentUser, Principal};
use super::service::{AuditContext, AuthService};
use super::throttle::source_key;
use super::{password, tokens};

const SLIDING_REFRESH: Duration = Duration::hours(1);

#[derive(Debug)]
pub struct SignedIn {
    pub token: Zeroizing<String>,
    pub expires_at: OffsetDateTime,
    pub user: CurrentUser,
}

fn invalid_credentials() -> AppError {
    AppError::unauthorized().with_param("reason", "INVALID_CREDENTIALS")
}

impl AuthService {
    pub async fn sign_in(&self, input: SignInInput) -> AppResult<SignedIn> {
        let user = self.verify_credentials(&input).await?;
        let (organization_id, _) = self.role_in(user.id, None).await?;
        self.start_session(user.id, organization_id, input.ip_address, input.user_agent)
            .await
    }

    pub(super) async fn start_session(
        &self,
        user_id: Id,
        organization_id: Id,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> AppResult<SignedIn> {
        let token = tokens::new_session_token();
        let expires_at = OffsetDateTime::now_utc() + self.session_ttl;
        self.repos
            .sessions
            .create(NewSession {
                id: Id::new(),
                user_id,
                token_hash: tokens::digest(&token),
                active_organization_id: Some(organization_id),
                ip_address: ip_address.clone(),
                user_agent,
                expires_at,
            })
            .await?;
        self.events.publish(DomainEvent::UserSignedIn { user_id });
        let context = AuditContext {
            organization_id: Some(organization_id),
            actor_id: Some(user_id),
            ip_address: ip_address.as_deref(),
        };
        self.audit(&context, "auth.sign_in", "user", Some(user_id))
            .await;
        let user = self.load_current_user(user_id, organization_id).await?;
        Ok(SignedIn {
            token,
            expires_at,
            user,
        })
    }

    async fn verify_credentials(&self, input: &SignInInput) -> AppResult<UserRecord> {
        let found = match normalize_email(&input.email) {
            Ok(email) => self.repos.identity.find_user_by_email(&email).await?,
            Err(_) => None,
        };
        let Some(user) = found else {
            password::dummy_verify(input.password.clone()).await;
            return Err(invalid_credentials());
        };
        let source = source_key(input.ip_address.as_deref());
        if self.is_throttled(user.id, source).await? {
            password::dummy_verify(input.password.clone()).await;
            return Err(invalid_credentials());
        }
        if !password::verify(input.password.clone(), user.password_hash.clone()).await? {
            self.record_login_failure(user.id, source).await?;
            return Err(invalid_credentials());
        }
        self.clear_login_failures(user.id, source).await?;
        Ok(user)
    }

    pub async fn authenticate_session(&self, token: &str) -> AppResult<Principal> {
        let now = OffsetDateTime::now_utc();
        let session = self
            .repos
            .sessions
            .find_active(&tokens::digest(token), now)
            .await?
            .ok_or_else(AppError::unauthorized)?;
        let (organization_id, role) = self
            .role_in(session.user_id, session.active_organization_id)
            .await?;
        if now - session.last_seen_at > SLIDING_REFRESH {
            self.repos
                .sessions
                .extend(session.id, now, now + self.session_ttl)
                .await?;
        }
        Ok(Principal {
            user_id: session.user_id,
            organization_id,
            role,
            session_id: Some(session.id),
            api_key_id: None,
        })
    }

    pub async fn sign_out(&self, principal: &Principal) -> AppResult<()> {
        if let Some(session_id) = principal.session_id {
            self.repos.sessions.delete(session_id).await?;
            self.events.publish(DomainEvent::UserSignedOut {
                user_id: principal.user_id,
            });
        }
        Ok(())
    }

    pub async fn purge_expired_sessions(&self) -> AppResult<u64> {
        self.repos
            .sessions
            .delete_expired(OffsetDateTime::now_utc())
            .await
    }
}
