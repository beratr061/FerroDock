use std::sync::Arc;

use ferrodock_core::events::{DomainEvent, EventBus};
use ferrodock_core::{AppResult, ErrorKind, Id};
use ferrodock_db::pg::PgStore;
use ferrodock_domain::audit::AuditLog;
use ferrodock_domain::auth::{AuthRepos, AuthService, Principal, Role, SetupInput};
use ferrodock_domain::projects::{Actor, NewProjectInput, ProjectService, ProjectUpdate};
use sqlx::PgPool;
use time::Duration;
use zeroize::Zeroizing;

struct SilentBus;

impl EventBus for SilentBus {
    fn publish(&self, _event: DomainEvent) {}
}

async fn owner_and_service(pool: PgPool) -> AppResult<(Principal, ProjectService)> {
    let store = Arc::new(PgStore::from_pool(pool));
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
    let principal = auth.authenticate_session(&signed_in.token).await?;
    let projects = ProjectService::new(store.clone(), AuditLog::new(store));
    Ok((principal, projects))
}

fn actor(principal: &Principal) -> Actor<'_> {
    Actor {
        principal,
        ip_address: None,
    }
}

fn input(name: &str) -> NewProjectInput {
    NewProjectInput {
        name: name.to_owned(),
        description: None,
    }
}

#[sqlx::test(migrator = "ferrodock_db::pg::MIGRATOR")]
async fn projects_start_with_production_and_keep_one_environment(pool: PgPool) -> AppResult<()> {
    let (owner, projects) = owner_and_service(pool).await?;
    let created = projects.create(actor(&owner), input("Shop API")).await?;
    assert_eq!(created.slug, "shop-api");
    assert_eq!(created.environments.len(), 1);
    let duplicate = projects.create(actor(&owner), input("shop api")).await;
    assert_eq!(duplicate.err().map(|e| e.code()), Some("PROJECT_EXISTS"));

    let staging = projects
        .add_environment(actor(&owner), created.id, "Staging")
        .await?;
    let production = created.environments[0].id;
    projects
        .remove_environment(actor(&owner), created.id, production)
        .await?;
    let last = projects
        .remove_environment(actor(&owner), created.id, staging.id)
        .await;
    assert_eq!(last.err().map(|e| e.code()), Some("ENVIRONMENT_LAST"));

    let renamed = ProjectUpdate {
        name: Some("Shop".to_owned()),
        description: Some("Storefront".to_owned()),
    };
    let updated = projects.update(actor(&owner), created.id, renamed).await?;
    assert_eq!(
        (updated.name.as_str(), updated.slug.as_str()),
        ("Shop", "shop-api")
    );
    assert_eq!(projects.list(&owner).await?[0].environments.len(), 1);
    Ok(())
}

#[sqlx::test(migrator = "ferrodock_db::pg::MIGRATOR")]
async fn projects_are_isolated_and_role_guarded(pool: PgPool) -> AppResult<()> {
    let (owner, projects) = owner_and_service(pool).await?;
    let created = projects.create(actor(&owner), input("Internal")).await?;
    let stranger = Principal {
        organization_id: Id::new(),
        ..owner.clone()
    };
    let hidden = projects.get(&stranger, created.id).await;
    assert_eq!(hidden.err().map(|e| e.kind()), Some(ErrorKind::NotFound));
    assert!(projects.list(&stranger).await?.is_empty());

    let viewer = Principal {
        role: Role::Viewer,
        ..owner.clone()
    };
    let denied = projects.create(actor(&viewer), input("Nope")).await;
    assert_eq!(denied.err().map(|e| e.kind()), Some(ErrorKind::Forbidden));
    let developer = Principal {
        role: Role::Developer,
        ..owner.clone()
    };
    let blocked = projects.delete(actor(&developer), created.id).await;
    assert_eq!(blocked.err().map(|e| e.kind()), Some(ErrorKind::Forbidden));
    projects.delete(actor(&owner), created.id).await?;
    assert!(projects.list(&owner).await?.is_empty());
    Ok(())
}
