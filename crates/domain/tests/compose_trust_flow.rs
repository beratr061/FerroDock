mod common;

use std::sync::Arc;

use ferrodock_core::AppResult;
use ferrodock_db::pg::PgStore;
use ferrodock_domain::applications::{
    ApplicationService, ApplicationUpdate, BuildKind, GitSourceInput, NewApplicationInput, Runtime,
    SourceInput,
};
use ferrodock_domain::audit::{Actor, AuditLog};
use ferrodock_domain::auth::{Principal, Role};
use ferrodock_domain::projects::{NewProjectInput, ProjectService};
use ferrodock_infra::docker::DockerEngine;
use ferrodock_infra::proxy::ProxyRoutes;
use sqlx::PgPool;

fn compose(branch: &str) -> SourceInput {
    SourceInput::Git(GitSourceInput {
        url: "https://github.com/example/stack.git".to_owned(),
        branch: Some(branch.to_owned()),
        build_kind: BuildKind::Compose,
        build_path: Some("compose.yaml".to_owned()),
        build_context: None,
    })
}

fn as_role(owner: &Principal, role: Role) -> Principal {
    Principal {
        role,
        ..owner.clone()
    }
}

#[sqlx::test(migrator = "ferrodock_db::pg::MIGRATOR")]
async fn only_the_owner_trusts_a_stack_and_new_sources_lose_it(pool: PgPool) -> AppResult<()> {
    let store = Arc::new(PgStore::from_pool(pool));
    let owner = common::sign_up_owner(&store).await?;
    let actor = Actor {
        principal: &owner,
        ip_address: None,
    };
    let audit = AuditLog::new(store.clone());
    let project = ProjectService::new(store.clone(), audit.clone())
        .create(
            actor,
            NewProjectInput {
                name: "Ops".to_owned(),
                description: None,
            },
        )
        .await?;
    let runtime = Runtime {
        docker: DockerEngine::connect_local()?,
        proxy: ProxyRoutes::new(None),
    };
    let applications = ApplicationService::new(store.clone(), common::secrets()?, audit, runtime);
    let input = NewApplicationInput {
        name: "stack".to_owned(),
        description: None,
        source: compose("main"),
        strategy: None,
    };
    let stack = applications
        .create(actor, project.environments[0].id, input)
        .await?;
    assert!(!stack.compose_trusted);

    let admin = as_role(&owner, Role::Admin);
    let refused = applications
        .set_compose_trusted(
            Actor {
                principal: &admin,
                ip_address: None,
            },
            stack.id,
            true,
        )
        .await;
    assert_eq!(refused.err().map(|e| e.code()), Some("FORBIDDEN"));
    assert!(
        applications
            .set_compose_trusted(actor, stack.id, true)
            .await?
            .compose_trusted
    );

    let developer = as_role(&owner, Role::Developer);
    let update = ApplicationUpdate {
        name: None,
        description: None,
        source: Some(compose("evil")),
        strategy: None,
    };
    let moved = applications
        .update(
            Actor {
                principal: &developer,
                ip_address: None,
            },
            stack.id,
            update,
        )
        .await?;
    assert!(!moved.compose_trusted);

    let image = common::application_in(&store, actor, "Web").await?;
    let not_compose = applications.set_compose_trusted(actor, image, true).await;
    assert_eq!(
        not_compose.err().map(|e| e.code()),
        Some("COMPOSE_TRUST_NOT_APPLICABLE")
    );
    Ok(())
}
