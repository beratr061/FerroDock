use std::sync::Arc;

use async_trait::async_trait;
use ferrodock_api::{ApiSettings, AppState, HealthProbe};
use ferrodock_core::AppResult;
use ferrodock_core::config::{AppConfig, ProxyMode};
use ferrodock_core::crypto::SecretBox;
use ferrodock_core::events::{DomainEvent, EventBus};
use ferrodock_db::pg::PgStore;
use ferrodock_domain::applications::{ApplicationService, Runtime};
use ferrodock_domain::audit::AuditLog;
use ferrodock_domain::auth::{AuthRepos, AuthService};
use ferrodock_domain::deployments::{DeploymentRepos, DeploymentService};
use ferrodock_domain::domains::{DomainService, DomainSettings};
use ferrodock_domain::projects::ProjectService;
use ferrodock_domain::variables::VariableService;
use ferrodock_engine::deploy::DeployContext;
use ferrodock_infra::deploy_log::DeployLogs;
use ferrodock_infra::docker::DockerEngine;
use ferrodock_infra::proxy::ProxyRoutes;
use ferrodock_infra::realtime::Hub;
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
    pub deploy: DeployContext,
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

fn auth_service(config: &AppConfig, store: &PgStore) -> AuthService {
    let shared = Arc::new(store.clone());
    let repos = AuthRepos {
        identity: shared.clone(),
        sessions: shared.clone(),
        api_keys: shared.clone(),
        audit: shared.clone(),
        login_attempts: shared,
    };
    let session_ttl = Duration::hours(i64::from(config.session_ttl_hours));
    AuthService::new(repos, Arc::new(TracingEventBus), session_ttl)
        .with_setup_token(config.setup_token.clone())
}

fn api_settings(config: &AppConfig) -> ApiSettings {
    ApiSettings {
        secure_cookies: config.secure_cookies,
        public_origin: config.public_url.clone(),
        trust_forwarded_for: config.trust_forwarded_for,
        version: env!("CARGO_PKG_VERSION"),
    }
}

fn deploy_context(
    config: &AppConfig,
    store: &PgStore,
    secrets: SecretBox,
) -> AppResult<DeployContext> {
    Ok(DeployContext {
        docker: DockerEngine::connect_local()?,
        applications: Arc::new(store.clone()),
        deployments: Arc::new(store.clone()),
        variables: Arc::new(store.clone()),
        secrets,
        logs: DeployLogs::new(config.data_dir.join("deployments")),
        data_dir: config.data_dir.clone(),
        hub: Hub::new(),
    })
}

fn proxy_routes(config: &AppConfig) -> ProxyRoutes {
    let enabled = config.proxy == ProxyMode::Traefik;
    ProxyRoutes::new(enabled.then(|| config.proxy_dir.clone()))
}

fn runtime(config: &AppConfig, deploy: &DeployContext) -> Runtime {
    Runtime {
        docker: deploy.docker.clone(),
        proxy: proxy_routes(config),
    }
}

fn panel_host(config: &AppConfig) -> Option<String> {
    let url = config.public_url.as_deref()?;
    let rest = url.split_once("://").map_or(url, |(_, rest)| rest);
    let host = rest.split(['/', ':']).next()?;
    Some(host.to_ascii_lowercase())
}

fn domain_service(config: &AppConfig, store: &PgStore, audit: AuditLog) -> DomainService {
    let settings = DomainSettings {
        public_ip: config.public_ip,
        panel_host: panel_host(config),
    };
    let repos = (
        Arc::new(store.clone()) as Arc<dyn ferrodock_db::DomainRepo>,
        Arc::new(store.clone()) as Arc<dyn ferrodock_db::ApplicationRepo>,
    );
    DomainService::new(repos, proxy_routes(config), settings, audit)
}

fn deployment_service(
    store: &PgStore,
    deploy: &DeployContext,
    audit: AuditLog,
) -> DeploymentService {
    let repos = DeploymentRepos {
        deployments: Arc::new(store.clone()),
        applications: Arc::new(store.clone()),
        jobs: Arc::new(store.clone()),
    };
    DeploymentService::new(repos, deploy.logs.clone(), audit, deploy.hub.clone())
}

pub async fn build(config: &AppConfig) -> AppResult<Wired> {
    let secrets = SecretBox::load_or_create(&config.master_key_path)?;
    let store = connect(config).await?;
    let deploy = deploy_context(config, &store, secrets.clone())?;
    let audit = AuditLog::new(Arc::new(store.clone()));
    let state = AppState {
        auth: auth_service(config, &store),
        docker: deploy.docker.clone(),
        hub: deploy.hub.clone(),
        audit: audit.clone(),
        projects: ProjectService::new(Arc::new(store.clone()), audit.clone()),
        applications: ApplicationService::new(
            Arc::new(store.clone()),
            secrets,
            audit.clone(),
            runtime(config, &deploy),
        ),
        domains: domain_service(config, &store, audit.clone()),
        variables: VariableService::new(
            (Arc::new(store.clone()), Arc::new(store.clone())),
            deploy.secrets.clone(),
            audit.clone(),
        ),
        deployments: deployment_service(&store, &deploy, audit),
        health: Arc::new(DatabaseProbe(store.clone())),
        settings: Arc::new(api_settings(config)),
    };
    Ok(Wired {
        state,
        store,
        deploy,
    })
}
