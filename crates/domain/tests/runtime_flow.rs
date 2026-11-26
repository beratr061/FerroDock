mod common;

use std::sync::Arc;

use ferrodock_core::AppResult;
use ferrodock_db::pg::PgStore;
use ferrodock_domain::audit::{Actor, AuditLog};
use ferrodock_domain::auth::{Principal, Role};
use ferrodock_domain::runtime_config::{
    MountInput, MountKind, PortInput, Protocol, RuntimeConfig, RuntimeConfigInput,
    RuntimeConfigService,
};
use sqlx::PgPool;

fn mount(kind: MountKind, source: &str, target: &str) -> MountInput {
    MountInput {
        kind,
        source: source.to_owned(),
        target: target.to_owned(),
        read_only: false,
    }
}

fn tcp(published: u16) -> PortInput {
    PortInput {
        published,
        target: 8080,
        protocol: Protocol::Tcp,
    }
}

#[sqlx::test(migrator = "ferrodock_db::pg::MIGRATOR")]
async fn runtime_config_is_saved_and_host_ports_stay_unique(pool: PgPool) -> AppResult<()> {
    let store = Arc::new(PgStore::from_pool(pool));
    let owner = common::sign_up_owner(&store).await?;
    let actor = Actor {
        principal: &owner,
        ip_address: None,
    };
    let web = common::application_in(&store, actor, "Shop").await?;
    let api = common::application_in(&store, actor, "Billing").await?;
    let configs =
        RuntimeConfigService::new((store.clone(), store.clone()), AuditLog::new(store.clone()));

    let input = RuntimeConfigInput {
        mounts: vec![mount(MountKind::Volume, "data", "/var/lib/data/")],
        ports: vec![tcp(9000)],
    };
    configs.replace(actor, web, input).await?;
    let saved = configs.get(&owner, web).await?;
    assert_eq!(saved.mounts[0].target, "/var/lib/data");
    assert_eq!(saved.ports[0].published, 9000);

    let clash = RuntimeConfigInput {
        mounts: vec![],
        ports: vec![tcp(9000)],
    };
    let taken = configs.replace(actor, api, clash).await;
    assert_eq!(taken.err().map(|e| e.code()), Some("RUNTIME_PORT_TAKEN"));

    let developer = Principal {
        role: Role::Developer,
        ..owner.clone()
    };
    let developer_actor = Actor {
        principal: &developer,
        ip_address: None,
    };
    let bind = RuntimeConfigInput {
        mounts: vec![mount(MountKind::Bind, "/srv/uploads", "/uploads")],
        ports: vec![],
    };
    let denied = configs.replace(developer_actor, api, bind).await;
    assert_eq!(denied.err().map(|e| e.code()), Some("FORBIDDEN"));

    configs
        .replace(actor, web, RuntimeConfigInput::default())
        .await?;
    assert_eq!(configs.get(&owner, web).await?, RuntimeConfig::default());
    Ok(())
}
