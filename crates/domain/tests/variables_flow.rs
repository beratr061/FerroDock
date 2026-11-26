mod common;

use std::sync::Arc;

use ferrodock_core::{AppResult, Id};
use ferrodock_db::pg::PgStore;
use ferrodock_domain::audit::{Actor, AuditLog};
use ferrodock_domain::auth::{Principal, Role};
use ferrodock_domain::variables::{VariableInput, VariableService};
use sqlx::PgPool;

fn input(key: &str, value: Option<&str>, secret: bool) -> VariableInput {
    VariableInput {
        key: key.to_owned(),
        value: value.map(str::to_owned),
        secret,
    }
}

#[sqlx::test(migrator = "ferrodock_db::pg::MIGRATOR")]
async fn secrets_stay_hidden_and_survive_updates(pool: PgPool) -> AppResult<()> {
    let store = Arc::new(PgStore::from_pool(pool));
    let owner = common::sign_up_owner(&store).await?;
    let actor = Actor {
        principal: &owner,
        ip_address: None,
    };
    let app = common::application_in(&store, actor, "Shop").await?;
    let variables = VariableService::new(
        (store.clone(), store.clone()),
        common::secrets()?,
        AuditLog::new(store.clone()),
    );

    let saved = variables
        .replace(
            actor,
            app,
            vec![
                input("MODE", Some("prod"), false),
                input("TOKEN", Some("t0ken"), true),
            ],
        )
        .await?;
    let token = saved.iter().find(|variable| variable.key == "TOKEN");
    assert_eq!(token.map(|variable| variable.value.is_none()), Some(true));

    variables
        .replace(actor, app, vec![input("TOKEN", None, true)])
        .await?;
    assert_eq!(variables.reveal(actor, app, "TOKEN").await?, "t0ken");
    assert_eq!(variables.list(&owner, app).await?.len(), 1);

    let missing = variables
        .replace(actor, app, vec![input("NEW", None, true)])
        .await;
    assert_eq!(
        missing.err().map(|e| e.code()),
        Some("VARIABLE_VALUE_REQUIRED")
    );
    let duplicate = variables
        .replace(
            actor,
            app,
            vec![input("A", Some("1"), false), input("A", Some("2"), false)],
        )
        .await;
    assert_eq!(
        duplicate.err().map(|e| e.code()),
        Some("VARIABLE_KEY_DUPLICATE")
    );

    let viewer = Principal {
        role: Role::Viewer,
        ..owner.clone()
    };
    let viewer_actor = Actor {
        principal: &viewer,
        ip_address: None,
    };
    assert!(variables.reveal(viewer_actor, app, "TOKEN").await.is_err());
    let stranger = Principal {
        organization_id: Id::new(),
        ..owner.clone()
    };
    assert!(variables.list(&stranger, app).await.is_err());
    Ok(())
}
