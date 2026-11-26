use std::sync::Arc;

use ferrodock_core::crypto::SecretBox;
use ferrodock_core::events::{DomainEvent, EventBus};
use ferrodock_core::{AppResult, ErrorKind, Id};
use ferrodock_db::ApplicationRepo;
use ferrodock_db::pg::PgStore;
use ferrodock_domain::applications::{
    ApplicationService, ApplicationUpdate, BuildKind, GitSourceInput, NewApplicationInput, Runtime,
    Source, SourceInput,
};
use ferrodock_domain::audit::{Actor, AuditLog};
use ferrodock_domain::auth::{AuthRepos, AuthService, Principal, SetupInput};
use ferrodock_domain::projects::{NewProjectInput, ProjectService};
use sqlx::PgPool;
use time::Duration;
use zeroize::Zeroizing;

struct SilentBus;

impl EventBus for SilentBus {
    fn publish(&self, _event: DomainEvent) {}
}

struct Fixture {
    owner: Principal,
    environment_id: Id,
    store: Arc<PgStore>,
    secrets: SecretBox,
    applications: ApplicationService,
}

async fn sign_up_owner(store: &Arc<PgStore>) -> AppResult<Principal> {
    let repos = AuthRepos {
        identity: store.clone(),
        sessions: store.clone(),
        api_keys: store.clone(),
        audit: store.clone(),
        login_attempts: store.clone(),
    };
    let auth = AuthService::new(repos, Arc::new(SilentBus), Duration::hours(1));
    let signed_in = auth
        .complete_setup(SetupInput {
            email: "owner@example.com".to_owned(),
            name: "Owner".to_owned(),
            password: Zeroizing::new("a-very-long-password".to_owned()),
            organization_name: "Acme".to_owned(),
            locale: "en".to_owned(),
            setup_token: None,
            ip_address: None,
            user_agent: None,
        })
        .await?;
    auth.authenticate_session(&signed_in.token).await
}

async fn fixture(pool: PgPool) -> AppResult<Fixture> {
    let store = Arc::new(PgStore::from_pool(pool));
    let owner = sign_up_owner(&store).await?;
    let audit = AuditLog::new(store.clone());
    let projects = ProjectService::new(store.clone(), audit.clone());
    let actor = Actor {
        principal: &owner,
        ip_address: None,
    };
    let project = projects
        .create(
            actor,
            NewProjectInput {
                name: "Shop".to_owned(),
                description: None,
            },
        )
        .await?;
    let secrets = SecretBox::from_key(&[7u8; 32])?;
    let applications = ApplicationService::new(store.clone(), secrets.clone(), audit, runtime()?);
    Ok(Fixture {
        environment_id: project.environments[0].id,
        owner,
        store,
        secrets,
        applications,
    })
}

fn git(url: &str) -> SourceInput {
    SourceInput::Git(GitSourceInput {
        url: url.to_owned(),
        branch: None,
        build_kind: BuildKind::Dockerfile,
        build_path: None,
        build_context: None,
    })
}

fn input(name: &str, source: SourceInput) -> NewApplicationInput {
    NewApplicationInput {
        name: name.to_owned(),
        description: None,
        source,
        strategy: None,
    }
}

#[sqlx::test(migrator = "ferrodock_db::pg::MIGRATOR")]
async fn ssh_sources_get_an_encrypted_deploy_key(pool: PgPool) -> AppResult<()> {
    let f = fixture(pool).await?;
    let actor = Actor {
        principal: &f.owner,
        ip_address: None,
    };
    let app = f
        .applications
        .create(
            actor,
            f.environment_id,
            input("API", git("git@github.com:acme/api.git")),
        )
        .await?;
    let defaults = match &app.source {
        Source::Git(source) => Some((source.branch.as_str(), source.build_path.as_str())),
        Source::Image(_) => None,
    };
    assert_eq!(defaults, Some(("main", "Dockerfile")));
    let public = app.deploy_key.clone().unwrap_or_default();
    assert!(public.starts_with("ssh-ed25519 "));

    let stored = f.store.find(f.owner.organization_id, app.id).await?;
    let sealed = stored
        .and_then(|record| record.deploy_key)
        .map(|key| key.private_enc);
    let private = f.secrets.decrypt(&sealed.unwrap_or_default())?;
    assert!(private.starts_with(b"-----BEGIN OPENSSH PRIVATE KEY-----"));

    let rotated = f.applications.rotate_deploy_key(actor, app.id).await?;
    assert_ne!(rotated.deploy_key, Some(public));
    let https = ApplicationUpdate {
        source: Some(git("https://github.com/acme/api.git")),
        ..ApplicationUpdate::default()
    };
    let switched = f.applications.update(actor, app.id, https).await?;
    assert_eq!(switched.deploy_key, None);
    Ok(())
}

#[sqlx::test(migrator = "ferrodock_db::pg::MIGRATOR")]
async fn applications_validate_and_stay_in_their_organization(pool: PgPool) -> AppResult<()> {
    let f = fixture(pool).await?;
    let actor = Actor {
        principal: &f.owner,
        ip_address: None,
    };
    let image = SourceInput::Image("nginx:1.29-alpine".to_owned());
    let app = f
        .applications
        .create(actor, f.environment_id, input("Web", image.clone()))
        .await?;
    assert_eq!(app.deploy_key, None);
    let duplicate = f
        .applications
        .create(actor, f.environment_id, input("web", image))
        .await;
    assert_eq!(
        duplicate.err().map(|e| e.code()),
        Some("APPLICATION_EXISTS")
    );
    let unsafe_url = git("https://user:token@github.com/acme/api.git");
    let rejected = f
        .applications
        .create(actor, f.environment_id, input("Bad", unsafe_url))
        .await;
    assert_eq!(rejected.err().map(|e| e.code()), Some("GIT_URL_INVALID"));

    let stranger = Principal {
        organization_id: Id::new(),
        ..f.owner.clone()
    };
    let hidden = f.applications.get(&stranger, app.id).await;
    assert_eq!(hidden.err().map(|e| e.kind()), Some(ErrorKind::NotFound));
    let foreign = Actor {
        principal: &stranger,
        ip_address: None,
    };
    let elsewhere = f
        .applications
        .create(
            foreign,
            f.environment_id,
            input("X", SourceInput::Image("nginx".to_owned())),
        )
        .await;
    assert_eq!(
        elsewhere.err().map(|e| e.code()),
        Some("ENVIRONMENT_NOT_FOUND")
    );
    f.applications.delete(actor, app.id, false).await?;
    assert!(f.applications.get(&f.owner, app.id).await.is_err());
    Ok(())
}

fn runtime() -> AppResult<Runtime> {
    Ok(Runtime {
        docker: ferrodock_infra::docker::DockerEngine::connect_local()?,
        proxy: ferrodock_infra::proxy::ProxyRoutes::new(None),
    })
}
