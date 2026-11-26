mod common;

use std::sync::Arc;

use ferrodock_core::crypto::hex;
use ferrodock_core::{AppResult, Id};
use ferrodock_db::pg::PgStore;
use ferrodock_domain::applications::{
    ApplicationService, BuildKind, GitSourceInput, NewApplicationInput, Runtime, SourceInput,
};
use ferrodock_domain::audit::{Actor, AuditLog};
use ferrodock_domain::deployments::{DeploymentRepos, DeploymentService};
use ferrodock_domain::projects::{NewProjectInput, ProjectService};
use ferrodock_domain::webhooks::{Delivery, Outcome, WebhookService};
use ferrodock_infra::deploy_log::DeployLogs;
use ferrodock_infra::docker::DockerEngine;
use ferrodock_infra::proxy::ProxyRoutes;
use hmac::{Hmac, KeyInit, Mac};
use sha2::Sha256;
use sqlx::PgPool;

fn service(store: &Arc<PgStore>) -> AppResult<WebhookService> {
    let repos = DeploymentRepos {
        deployments: store.clone(),
        applications: store.clone(),
        jobs: store.clone(),
    };
    let logs = DeployLogs::new(std::env::temp_dir().join(format!("fd-test-{}", Id::new())));
    let audit = AuditLog::new(store.clone());
    let deployments = DeploymentService::new(
        repos,
        logs,
        audit.clone(),
        ferrodock_infra::realtime::Hub::new(),
    );
    Ok(WebhookService::new(
        (store.clone(), store.clone()),
        deployments,
        common::secrets()?,
        audit,
    ))
}

async fn git_application(store: &Arc<PgStore>, actor: Actor<'_>) -> AppResult<Id> {
    let audit = AuditLog::new(store.clone());
    let project = ProjectService::new(store.clone(), audit.clone())
        .create(
            actor,
            NewProjectInput {
                name: "Shop".to_owned(),
                description: None,
            },
        )
        .await?;
    let runtime = Runtime {
        docker: DockerEngine::connect_local()?,
        proxy: ProxyRoutes::new(None),
    };
    let source = SourceInput::Git(GitSourceInput {
        url: "https://github.com/example/shop.git".to_owned(),
        branch: Some("main".to_owned()),
        build_kind: BuildKind::Dockerfile,
        build_path: None,
        build_context: None,
    });
    let input = NewApplicationInput {
        name: "shop".to_owned(),
        description: None,
        source,
        strategy: None,
    };
    let application = ApplicationService::new(store.clone(), common::secrets()?, audit, runtime)
        .create(actor, project.environments[0].id, input)
        .await?;
    Ok(application.id)
}

fn signature(secret: &str, body: &[u8]) -> String {
    let Ok(mut mac) = Hmac::<Sha256>::new_from_slice(secret.as_bytes()) else {
        return String::new();
    };
    mac.update(body);
    format!("sha256={}", hex(&mac.finalize().into_bytes()))
}

fn github<'a>(header: &'a str, body: &'a [u8]) -> Delivery<'a> {
    Delivery {
        event: Some("push"),
        hub_signature: Some(header),
        body,
        ..Delivery::default()
    }
}

#[sqlx::test(migrator = "ferrodock_db::pg::MIGRATOR")]
async fn signed_pushes_to_the_branch_deploy(pool: PgPool) -> AppResult<()> {
    let store = Arc::new(PgStore::from_pool(pool));
    let owner = common::sign_up_owner(&store).await?;
    let actor = Actor {
        principal: &owner,
        ip_address: None,
    };
    let webhooks = service(&store)?;
    let app = git_application(&store, actor).await?;
    assert!(webhooks.get(&owner, app).await?.is_none());
    let (_, secret) = webhooks.rotate(actor, app).await?;

    let main = br#"{"ref":"refs/heads/main","after":"abc123"}"#;
    let outcome = webhooks
        .receive(app, github(&signature(&secret, main), main))
        .await?;
    assert!(matches!(outcome, Outcome::Deployed(_)));

    let other = br#"{"ref":"refs/heads/feature","after":"abc123"}"#;
    let outcome = webhooks
        .receive(app, github(&signature(&secret, other), other))
        .await?;
    assert_eq!(outcome, Outcome::Ignored("ignored_branch"));

    let forged = webhooks
        .receive(app, github(&signature("guess", main), main))
        .await;
    assert_eq!(forged.err().map(|e| e.code()), Some("UNAUTHORIZED"));
    let last = webhooks
        .get(&owner, app)
        .await?
        .and_then(|hook| hook.last_outcome);
    assert_eq!(last.as_deref(), Some("rejected"));

    let (_, rotated) = webhooks.rotate(actor, app).await?;
    let stale = webhooks
        .receive(app, github(&signature(&secret, main), main))
        .await;
    assert!(stale.is_err());
    assert!(
        webhooks
            .receive(app, github(&signature(&rotated, main), main))
            .await
            .is_ok()
    );

    webhooks.disable(actor, app).await?;
    let disabled = webhooks
        .receive(app, github(&signature(&rotated, main), main))
        .await;
    assert!(disabled.is_err());

    let image = common::application_in(&store, actor, "Images").await?;
    let refused = webhooks.rotate(actor, image).await;
    assert_eq!(
        refused.err().map(|e| e.code()),
        Some("WEBHOOK_NOT_APPLICABLE")
    );
    Ok(())
}
