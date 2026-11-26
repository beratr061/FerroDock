use std::sync::Arc;

use ferrodock_core::crypto::SecretBox;
use ferrodock_core::events::{DomainEvent, EventBus};
use ferrodock_core::{AppResult, Id};
use ferrodock_db::pg::PgStore;
use ferrodock_domain::applications::{
    ApplicationService, NewApplicationInput, Runtime, SourceInput,
};
use ferrodock_domain::audit::{Actor, AuditLog};
use ferrodock_domain::auth::{AuthRepos, AuthService, Principal, SetupInput};
use ferrodock_domain::projects::{NewProjectInput, ProjectService};
use ferrodock_infra::docker::DockerEngine;
use ferrodock_infra::proxy::ProxyRoutes;
use time::Duration;
use zeroize::Zeroizing;

struct SilentBus;

impl EventBus for SilentBus {
    fn publish(&self, _event: DomainEvent) {}
}

pub fn secrets() -> AppResult<SecretBox> {
    SecretBox::from_key(&[9u8; 32])
}

pub async fn sign_up_owner(store: &Arc<PgStore>) -> AppResult<Principal> {
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

pub async fn application_in(
    store: &Arc<PgStore>,
    actor: Actor<'_>,
    project: &str,
) -> AppResult<Id> {
    let audit = AuditLog::new(store.clone());
    let project_input = NewProjectInput {
        name: project.to_owned(),
        description: None,
    };
    let project = ProjectService::new(store.clone(), audit.clone())
        .create(actor, project_input)
        .await?;
    let runtime = Runtime {
        docker: DockerEngine::connect_local()?,
        proxy: ProxyRoutes::new(None),
    };
    let service = ApplicationService::new(store.clone(), secrets()?, audit, runtime);
    let input = NewApplicationInput {
        name: "web".to_owned(),
        description: None,
        source: SourceInput::Image("nginx".to_owned()),
        strategy: None,
    };
    Ok(service
        .create(actor, project.environments[0].id, input)
        .await?
        .id)
}
