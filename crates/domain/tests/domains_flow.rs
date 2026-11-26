use std::path::PathBuf;
use std::sync::Arc;

use ferrodock_core::crypto::SecretBox;
use ferrodock_core::events::{DomainEvent, EventBus};
use ferrodock_core::{AppResult, Id};
use ferrodock_db::pg::PgStore;
use ferrodock_db::{ApplicationRepo, DomainRepo};
use ferrodock_domain::applications::{
    ApplicationService, NewApplicationInput, Runtime, SourceInput,
};
use ferrodock_domain::audit::{Actor, AuditLog};
use ferrodock_domain::auth::{AuthRepos, AuthService, Principal, SetupInput};
use ferrodock_domain::domains::{BasicAuthInput, DomainService, DomainSettings, NewDomainInput};
use ferrodock_domain::projects::{NewProjectInput, ProjectService};
use ferrodock_infra::docker::{DockerEngine, application_key};
use ferrodock_infra::proxy::ProxyRoutes;
use sqlx::PgPool;
use time::Duration;
use zeroize::Zeroizing;

struct SilentBus;

impl EventBus for SilentBus {
    fn publish(&self, _event: DomainEvent) {}
}

async fn owner(store: &Arc<PgStore>) -> AppResult<Principal> {
    let repos = AuthRepos {
        identity: store.clone(),
        sessions: store.clone(),
        api_keys: store.clone(),
        audit: store.clone(),
        login_attempts: store.clone(),
    };
    let auth = AuthService::new(repos, Arc::new(SilentBus), Duration::hours(1));
    let setup = SetupInput {
        email: "owner@example.com".to_owned(),
        name: "Owner".to_owned(),
        password: Zeroizing::new("a-very-long-password".to_owned()),
        organization_name: "Acme".to_owned(),
        locale: "en".to_owned(),
        setup_token: None,
        ip_address: None,
        user_agent: None,
    };
    let signed_in = auth.complete_setup(setup).await?;
    auth.authenticate_session(&signed_in.token).await
}

async fn two_applications(store: &Arc<PgStore>, actor: Actor<'_>) -> AppResult<(Id, Id)> {
    let audit = AuditLog::new(store.clone());
    let project_input = NewProjectInput {
        name: "Shop".to_owned(),
        description: None,
    };
    let project = ProjectService::new(store.clone(), audit.clone())
        .create(actor, project_input)
        .await?;
    let runtime = Runtime {
        docker: DockerEngine::connect_local()?,
        proxy: ProxyRoutes::new(None),
    };
    let service = ApplicationService::new(
        store.clone(),
        SecretBox::from_key(&[5u8; 32])?,
        audit,
        runtime,
    );
    let mut ids = Vec::new();
    for name in ["web", "api"] {
        let input = NewApplicationInput {
            name: name.to_owned(),
            description: None,
            source: SourceInput::Image("nginx".to_owned()),
            strategy: None,
        };
        ids.push(
            service
                .create(actor, project.environments[0].id, input)
                .await?
                .id,
        );
    }
    Ok((ids[0], ids[1]))
}

fn domain(host: &str) -> NewDomainInput {
    NewDomainInput {
        host: host.to_owned(),
        path_prefix: None,
        container_port: 8080,
        https: true,
        service: None,
        basic_auth: None,
    }
}

#[sqlx::test(migrator = "ferrodock_db::pg::MIGRATOR")]
async fn domains_publish_routes_and_stay_unique(pool: PgPool) -> AppResult<()> {
    let store = Arc::new(PgStore::from_pool(pool));
    let principal = owner(&store).await?;
    let actor = Actor {
        principal: &principal,
        ip_address: None,
    };
    let (web, api) = two_applications(&store, actor).await?;
    let dir: PathBuf = std::env::temp_dir().join(format!("fd-proxy-{}", Id::new()));
    tokio::fs::create_dir_all(&dir)
        .await
        .map_err(ferrodock_core::AppError::internal)?;
    let settings = DomainSettings {
        public_ip: None,
        panel_host: Some("panel.example.com".to_owned()),
    };
    let repos = (
        store.clone() as Arc<dyn DomainRepo>,
        store.clone() as Arc<dyn ApplicationRepo>,
    );
    let domains = DomainService::new(
        repos,
        ProxyRoutes::new(Some(dir.clone())),
        settings,
        AuditLog::new(store.clone()),
    );

    let created = domains.add(actor, web, domain("Shop.Example.com")).await?;
    assert_eq!(created.host, "shop.example.com");
    let file = dir.join(format!("app-{}.yml", application_key(web)));
    let routes = tokio::fs::read_to_string(&file).await.unwrap_or_default();
    assert!(routes.contains("Host(`shop.example.com`)"));

    let mut guarded = domain("admin.example.com");
    guarded.basic_auth = Some(BasicAuthInput {
        username: "ops".to_owned(),
        password: Zeroizing::new("long-enough-secret".to_owned()),
    });
    let protected = domains.add(actor, web, guarded).await?;
    assert_eq!(protected.basic_auth_user.as_deref(), Some("ops"));
    let routes = tokio::fs::read_to_string(&file).await.unwrap_or_default();
    assert!(routes.contains("\"ops:$2"));
    assert!(!routes.contains("long-enough-secret"));
    domains.remove(actor, web, protected.id).await?;

    let elsewhere = domains.add(actor, api, domain("shop.example.com")).await;
    assert_eq!(elsewhere.err().map(|e| e.code()), Some("DOMAIN_TAKEN"));
    let reserved = domains.add(actor, api, domain("panel.example.com")).await;
    assert_eq!(reserved.err().map(|e| e.code()), Some("DOMAIN_RESERVED"));
    let ip = domains.add(actor, api, domain("192.0.2.1")).await;
    assert_eq!(ip.err().map(|e| e.code()), Some("DOMAIN_HOST_IP"));

    domains.remove(actor, web, created.id).await?;
    assert!(!tokio::fs::try_exists(&file).await.unwrap_or(true));
    let _ = tokio::fs::remove_dir_all(&dir).await;
    Ok(())
}
