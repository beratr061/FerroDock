mod common;

use std::sync::Arc;

use ferrodock_core::AppResult;
use ferrodock_db::pg::PgStore;
use ferrodock_domain::audit::{Actor, AuditLog};
use ferrodock_domain::auth::{Principal, Role};
use ferrodock_domain::databases::{
    DatabaseService, DatabaseServiceParts, DatabaseStatus, NewDatabaseInput,
};
use ferrodock_domain::projects::{NewProjectInput, ProjectService};
use ferrodock_domain::runtime_config::{
    PortInput, Protocol, RuntimeConfigInput, RuntimeConfigService,
};
use ferrodock_infra::deploy_log::DeployLogs;
use ferrodock_infra::docker::DockerEngine;
use sqlx::PgPool;

fn service(store: &Arc<PgStore>) -> AppResult<DatabaseService> {
    Ok(DatabaseService::new(DatabaseServiceParts {
        repo: store.clone(),
        jobs: store.clone(),
        docker: DockerEngine::connect_local()?,
        secrets: common::secrets()?,
        logs: DeployLogs::new(std::env::temp_dir().join("ferrodock-database-logs")),
        audit: AuditLog::new(store.clone()),
        public_host: Some("203.0.113.7".to_owned()),
    }))
}

fn input(name: &str, engine: &str) -> NewDatabaseInput {
    NewDatabaseInput {
        name: name.to_owned(),
        engine: engine.to_owned(),
        ..NewDatabaseInput::default()
    }
}

#[sqlx::test(migrator = "ferrodock_db::pg::MIGRATOR")]
async fn databases_get_secret_credentials_and_unique_host_ports(pool: PgPool) -> AppResult<()> {
    let store = Arc::new(PgStore::from_pool(pool));
    let owner = common::sign_up_owner(&store).await?;
    let actor = Actor {
        principal: &owner,
        ip_address: None,
    };
    let project_input = NewProjectInput {
        name: "Shop".to_owned(),
        description: None,
    };
    let project = ProjectService::new(store.clone(), AuditLog::new(store.clone()))
        .create(actor, project_input)
        .await?;
    let environment = project.environments[0].id;
    let databases = service(&store)?;

    let orders = databases
        .create(actor, environment, input("Orders DB", "postgres"))
        .await?;
    assert_eq!(orders.database_name, "orders_db");
    assert_eq!(orders.version, "18");
    assert_eq!(orders.status, DatabaseStatus::Deploying);
    let cache = databases
        .create(actor, environment, input("Cache", "redis"))
        .await?;
    assert_eq!(cache.username, "default");
    let duplicate = databases
        .create(actor, environment, input("Orders DB", "mysql"))
        .await;
    assert_eq!(duplicate.err().map(|e| e.code()), Some("DATABASE_EXISTS"));

    let published = databases
        .set_external_port(actor, orders.id, Some(15432))
        .await?;
    assert_eq!(published.external_port, Some(15432));
    let secrets = databases.credentials(actor, orders.id).await?;
    assert_eq!(secrets.password.len(), 32);
    let expected = format!(
        "postgresql://app:{}@{}:5432/orders_db",
        secrets.password, orders.internal_host
    );
    assert_eq!(secrets.internal_url, expected);
    assert!(
        secrets
            .external_url
            .is_some_and(|url| url.contains("@203.0.113.7:15432/"))
    );

    let clash = databases
        .set_external_port(actor, cache.id, Some(15432))
        .await;
    assert_eq!(clash.err().map(|e| e.code()), Some("DATABASE_PORT_TAKEN"));
    let application = common::application_in(&store, actor, "Billing").await?;
    let runtime =
        RuntimeConfigService::new((store.clone(), store.clone()), AuditLog::new(store.clone()));
    let ports = RuntimeConfigInput {
        mounts: vec![],
        ports: vec![PortInput {
            published: 15432,
            target: 80,
            protocol: Protocol::Tcp,
        }],
    };
    let taken = runtime.replace(actor, application, ports).await;
    assert_eq!(taken.err().map(|e| e.code()), Some("RUNTIME_PORT_TAKEN"));

    let developer = Principal {
        role: Role::Developer,
        ..owner.clone()
    };
    let developer_actor = Actor {
        principal: &developer,
        ip_address: None,
    };
    let denied = databases.delete(developer_actor, orders.id, true).await;
    assert_eq!(denied.err().map(|e| e.code()), Some("FORBIDDEN"));
    let listed = databases.list(&owner, project.id).await?;
    assert_eq!(listed.len(), 2);
    Ok(())
}
