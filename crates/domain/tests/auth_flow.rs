use std::sync::{Arc, Mutex};

use ferrodock_core::events::{DomainEvent, EventBus};
use ferrodock_core::{AppResult, ErrorKind};
use ferrodock_db::pg::PgStore;
use ferrodock_domain::auth::{
    AuthRepos, AuthService, NewApiKeyInput, Role, SetupInput, SignInInput,
};
use sqlx::PgPool;
use time::Duration;
use zeroize::Zeroizing;

#[derive(Default)]
struct RecordingBus(Mutex<Vec<DomainEvent>>);

impl EventBus for RecordingBus {
    fn publish(&self, event: DomainEvent) {
        if let Ok(mut events) = self.0.lock() {
            events.push(event);
        }
    }
}

fn service(pool: PgPool, bus: Arc<RecordingBus>) -> AuthService {
    let store = Arc::new(PgStore::from_pool(pool));
    let repos = AuthRepos {
        identity: store.clone(),
        sessions: store.clone(),
        api_keys: store.clone(),
        audit: store.clone(),
        login_attempts: store,
    };
    AuthService::new(repos, bus, Duration::hours(1))
}

fn setup_input(email: &str) -> SetupInput {
    SetupInput {
        email: email.to_owned(),
        name: "Owner".to_owned(),
        password: Zeroizing::new("a-very-long-password".to_owned()),
        organization_name: "Acme Ops".to_owned(),
        locale: "tr".to_owned(),
        setup_token: None,
        ip_address: None,
        user_agent: None,
    }
}

fn sign_in_from(password: &str, ip: &str) -> SignInInput {
    SignInInput {
        email: "OWNER@example.com".to_owned(),
        password: Zeroizing::new(password.to_owned()),
        ip_address: Some(ip.to_owned()),
        user_agent: None,
    }
}

fn sign_in_input(password: &str) -> SignInInput {
    sign_in_from(password, "198.51.100.7")
}

#[sqlx::test(migrator = "ferrodock_db::pg::MIGRATOR")]
async fn setup_runs_once_and_sessions_authenticate(pool: PgPool) -> AppResult<()> {
    let bus = Arc::new(RecordingBus::default());
    let auth = service(pool, bus.clone());
    assert!(auth.setup_required().await?);
    let owner = auth
        .complete_setup(setup_input("owner@example.com"))
        .await?;
    assert_eq!(owner.user.memberships.len(), 1);
    assert!(auth.authenticate_session(&owner.token).await.is_ok());
    let second = auth.complete_setup(setup_input("other@example.com")).await;
    assert!(second.is_err_and(|error| error.kind() == ErrorKind::Conflict));
    let signed_in = auth.sign_in(sign_in_input("a-very-long-password")).await?;
    let principal = auth.authenticate_session(&signed_in.token).await?;
    assert_eq!(principal.role, Role::Owner);
    auth.sign_out(&principal).await?;
    assert!(auth.authenticate_session(&signed_in.token).await.is_err());
    assert!(bus.0.lock().is_ok_and(|events| events.len() >= 3));
    Ok(())
}

#[sqlx::test(migrator = "ferrodock_db::pg::MIGRATOR")]
async fn failures_lock_only_the_offending_source(pool: PgPool) -> AppResult<()> {
    let auth = service(pool, Arc::new(RecordingBus::default()));
    auth.complete_setup(setup_input("owner@example.com"))
        .await?;
    for _ in 0..5 {
        let attempt = auth
            .sign_in(sign_in_from("wrong-password-123", "203.0.113.50"))
            .await;
        assert!(attempt.is_err());
    }
    let attacker = auth
        .sign_in(sign_in_from("a-very-long-password", "203.0.113.50"))
        .await;
    assert!(attacker.is_err_and(|error| error.kind() == ErrorKind::Unauthorized));
    auth.sign_in(sign_in_from("a-very-long-password", "198.51.100.7"))
        .await?;
    Ok(())
}

#[sqlx::test(migrator = "ferrodock_db::pg::MIGRATOR")]
async fn api_keys_are_scoped_and_revocable(pool: PgPool) -> AppResult<()> {
    let auth = service(pool, Arc::new(RecordingBus::default()));
    auth.complete_setup(setup_input("owner@example.com"))
        .await?;
    let signed_in = auth.sign_in(sign_in_input("a-very-long-password")).await?;
    let owner = auth.authenticate_session(&signed_in.token).await?;
    let input = NewApiKeyInput {
        name: "ci".to_owned(),
        role: Role::Viewer,
        expires_in_days: Some(7),
    };
    let created = auth.create_api_key(&owner, input).await?;
    let via_key = auth.authenticate_api_key(&created.key).await?;
    assert_eq!(via_key.role, Role::Viewer);
    assert_eq!(auth.list_api_keys(&owner).await?.len(), 1);
    auth.revoke_api_key(&owner, created.summary.id).await?;
    assert!(auth.authenticate_api_key(&created.key).await.is_err());
    assert!(auth.authenticate_api_key("fd_live_unknown").await.is_err());
    Ok(())
}

#[sqlx::test(migrator = "ferrodock_db::pg::MIGRATOR")]
async fn setup_token_guards_owner_creation(pool: PgPool) -> AppResult<()> {
    let token = ferrodock_core::Secret::new("install-token-123".to_owned());
    let auth = service(pool, Arc::new(RecordingBus::default())).with_setup_token(Some(token));
    assert!(auth.setup_token_required());
    let missing = auth.complete_setup(setup_input("owner@example.com")).await;
    assert!(missing.is_err_and(|error| error.kind() == ErrorKind::Forbidden));
    let mut wrong = setup_input("owner@example.com");
    wrong.setup_token = Some(Zeroizing::new("install-token-999".to_owned()));
    assert!(auth.complete_setup(wrong).await.is_err());
    let mut valid = setup_input("owner@example.com");
    valid.setup_token = Some(Zeroizing::new("install-token-123".to_owned()));
    auth.complete_setup(valid).await?;
    Ok(())
}
