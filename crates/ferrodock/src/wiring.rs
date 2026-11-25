use std::sync::Arc;

use async_trait::async_trait;
use ferrodock_api::{ApiSettings, AppState, HealthProbe};
use ferrodock_core::AppResult;
use ferrodock_core::config::AppConfig;
use ferrodock_core::crypto::SecretBox;
use ferrodock_core::events::{DomainEvent, EventBus};
use ferrodock_db::pg::PgStore;
use ferrodock_domain::applications::ApplicationService;
use ferrodock_domain::audit::AuditLog;
use ferrodock_domain::auth::{AuthRepos, AuthService};
use ferrodock_domain::projects::ProjectService;
use time::Duration;

#[derive(Debug)]
struct TracingEventBus;

impl EventBus for TracingEventBus {
    fn publish(&self, event: DomainEvent) {
        tracing::info!(?event, "domain event");
    }
}

#[derive(Debug)]
struct DatabaseProbe(PgStore);

#[async_trait]
impl HealthProbe for DatabaseProbe {
    async fn database(&self) -> AppResult<()> {
        self.0.ping().await
    }
}

#[derive(Debug)]
pub struct Wired {
    pub state: AppState,
    pub store: PgStore,
}

pub async fn connect(config: &AppConfig) -> AppResult<PgStore> {
    let store = PgStore::connect(
        config.database_url.expose(),
        config.database_max_connections,
    )
    .await?;
    store.migrate().await?;
    Ok(store)
}

pub async fn migrate_only(config: &AppConfig) -> AppResult<()> {
    let store = connect(config).await?;
    store.close().await;
    tracing::info!("database migrations applied");
    Ok(())
}

pub async fn build(config: &AppConfig) -> AppResult<Wired> {
    let secrets = SecretBox::load_or_create(&config.master_key_path)?;
    let store = connect(config).await?;
    let shared = Arc::new(store.clone());
    let repos = AuthRepos {
        identity: shared.clone(),
        sessions: shared.clone(),
        api_keys: shared.clone(),
        audit: shared.clone(),
        login_attempts: shared,
    };
    let session_ttl = Duration::hours(i64::from(config.session_ttl_hours));
    let auth = AuthService::new(repos, Arc::new(TracingEventBus), session_ttl)
        .with_setup_token(config.setup_token.clone());
    let settings = ApiSettings {
        secure_cookies: config.secure_cookies,
        public_origin: config.public_url.clone(),
        trust_forwarded_for: config.trust_forwarded_for,
        version: env!("CARGO_PKG_VERSION"),
    };
    let audit = AuditLog::new(Arc::new(store.clone()));
    let state = AppState {
        auth,
        docker: ferrodock_infra::docker::DockerEngine::connect_local()?,
        audit: audit.clone(),
        projects: ProjectService::new(Arc::new(store.clone()), audit.clone()),
        applications: ApplicationService::new(Arc::new(store.clone()), secrets, audit),
        health: Arc::new(DatabaseProbe(store.clone())),
        settings: Arc::new(settings),
    };
    Ok(Wired { state, store })
}
