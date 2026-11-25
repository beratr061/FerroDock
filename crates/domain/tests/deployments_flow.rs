use std::sync::Arc;

use ferrodock_core::crypto::SecretBox;
use ferrodock_core::events::{DomainEvent, EventBus};
use ferrodock_core::{AppResult, Id};
use ferrodock_db::DeploymentRepo;
use ferrodock_db::deployments::DeploymentArtifact;
use ferrodock_db::pg::PgStore;
use ferrodock_domain::applications::{
    ApplicationService, NewApplicationInput, Runtime, SourceInput,
};
use ferrodock_domain::audit::{Actor, AuditLog};
use ferrodock_domain::auth::{AuthRepos, AuthService, Principal, SetupInput};
use ferrodock_domain::deployments::{DeploymentRepos, DeploymentService, DeploymentStatus};
use ferrodock_domain::projects::{NewProjectInput, ProjectService};
use ferrodock_infra::deploy_log::DeployLogs;
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

async fn application(store: &Arc<PgStore>, principal: &Principal) -> AppResult<Id> {
    let audit = AuditLog::new(store.clone());
    let actor = Actor {
        principal,
        ip_address: None,
    };
    let project_input = NewProjectInput {
        name: "Shop".to_owned(),
        description: None,
    };
    let project = ProjectService::new(store.clone(), audit.clone())
        .create(actor, project_input)
        .await?;
    let applications = ApplicationService::new(
        store.clone(),
        SecretBox::from_key(&[3u8; 32])?,
        audit,
        runtime()?,
    );
    let input = NewApplicationInput {
        name: "web".to_owned(),
        description: None,
        source: SourceInput::Image("nginx:1.29-alpine".to_owned()),
        strategy: None,
    };
    Ok(applications
        .create(actor, project.environments[0].id, input)
        .await?
        .id)
}

fn service(store: &Arc<PgStore>) -> DeploymentService {
    let repos = DeploymentRepos {
        deployments: store.clone(),
        applications: store.clone(),
        jobs: store.clone(),
    };
    let logs = DeployLogs::new(std::env::temp_dir().join(format!("fd-test-{}", Id::new())));
    DeploymentService::new(
        repos,
        logs,
        AuditLog::new(store.clone()),
        ferrodock_infra::realtime::Hub::new(),
    )
}

#[sqlx::test(migrator = "ferrodock_db::pg::MIGRATOR")]
async fn deployments_queue_cancel_and_guard_rollbacks(pool: PgPool) -> AppResult<()> {
    let store = Arc::new(PgStore::from_pool(pool));
    let principal = owner(&store).await?;
    let app_id = application(&store, &principal).await?;
    let deployments = service(&store);
    let actor = Actor {
        principal: &principal,
        ip_address: None,
    };

    let queued = deployments.deploy(actor, app_id).await?;
    assert_eq!(queued.status, DeploymentStatus::Queued);
    let early = deployments.rollback(actor, queued.id).await;
    assert_eq!(
        early.err().map(|e| e.code()),
        Some("ROLLBACK_TARGET_UNAVAILABLE")
    );
    let cancelled = deployments.cancel(actor, queued.id).await?;
    assert_eq!(cancelled.status, DeploymentStatus::Cancelled);
    let again = deployments.cancel(actor, queued.id).await;
    assert_eq!(
        again.err().map(|e| e.code()),
        Some("DEPLOYMENT_NOT_CANCELLABLE")
    );

    let artifact = DeploymentArtifact {
        commit_sha: None,
        image_ref: "nginx:1.29-alpine".to_owned(),
        image_id: "sha256:abc".to_owned(),
    };
    store.record_artifact(queued.id, artifact).await?;
    assert_eq!(deployments.list(&principal, app_id).await?.len(), 1);

    let stranger = Principal {
        organization_id: Id::new(),
        ..principal.clone()
    };
    assert!(deployments.get(&stranger, queued.id).await.is_err());
    let foreign = Actor {
        principal: &stranger,
        ip_address: None,
    };
    let blocked = deployments.deploy(foreign, app_id).await;
    assert_eq!(
        blocked.err().map(|e| e.code()),
        Some("APPLICATION_NOT_FOUND")
    );
    Ok(())
}

fn runtime() -> AppResult<Runtime> {
    Ok(Runtime {
        docker: ferrodock_infra::docker::DockerEngine::connect_local()?,
        proxy: ferrodock_infra::proxy::ProxyRoutes::new(None),
    })
}
