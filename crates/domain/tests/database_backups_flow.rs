mod common;

use std::sync::Arc;

use ferrodock_core::AppResult;
use ferrodock_db::BackupPolicyRepo;
use ferrodock_db::backups::PolicyRecord;
use ferrodock_db::pg::PgStore;
use ferrodock_domain::audit::{Actor, AuditLog};
use ferrodock_domain::auth::{Principal, Role};
use ferrodock_domain::database_backups::{
    DatabaseBackupParts, DatabaseBackupService, NewDestinationInput, PolicyInput,
};
use ferrodock_domain::databases::{DatabaseService, DatabaseServiceParts, NewDatabaseInput};
use ferrodock_domain::projects::{NewProjectInput, ProjectService};
use ferrodock_infra::deploy_log::DeployLogs;
use ferrodock_infra::docker::DockerEngine;
use sqlx::PgPool;
use time::{Duration, OffsetDateTime};

fn backups(store: &Arc<PgStore>) -> AppResult<DatabaseBackupService> {
    let root = std::env::temp_dir().join(format!("fd-db-backups-{}", ferrodock_core::Id::new()));
    Ok(DatabaseBackupService::new(DatabaseBackupParts {
        databases: store.clone(),
        destinations: store.clone(),
        policies: store.clone(),
        runs: store.clone(),
        jobs: store.clone(),
        secrets: common::secrets()?,
        audit: AuditLog::new(store.clone()),
        local_root: root,
    }))
}

async fn database(store: &Arc<PgStore>, actor: Actor<'_>) -> AppResult<ferrodock_core::Id> {
    let project = ProjectService::new(store.clone(), AuditLog::new(store.clone()))
        .create(
            actor,
            NewProjectInput {
                name: "Shop".to_owned(),
                description: None,
            },
        )
        .await?;
    let service = DatabaseService::new(DatabaseServiceParts {
        repo: store.clone(),
        jobs: store.clone(),
        docker: DockerEngine::connect_local()?,
        secrets: common::secrets()?,
        logs: DeployLogs::new(std::env::temp_dir().join("fd-db-logs")),
        audit: AuditLog::new(store.clone()),
        public_host: None,
    });
    let input = NewDatabaseInput {
        name: "Orders".to_owned(),
        engine: "postgres".to_owned(),
        ..NewDatabaseInput::default()
    };
    Ok(service
        .create(actor, project.environments[0].id, input)
        .await?
        .id)
}

#[sqlx::test(migrator = "ferrodock_db::pg::MIGRATOR")]
async fn schedules_claim_once_and_destinations_stay_protected(pool: PgPool) -> AppResult<()> {
    let store = Arc::new(PgStore::from_pool(pool));
    let owner = common::sign_up_owner(&store).await?;
    let actor = Actor {
        principal: &owner,
        ip_address: None,
    };
    let database_id = database(&store, actor).await?;
    let service = backups(&store)?;

    let developer = Principal {
        role: Role::Developer,
        ..owner.clone()
    };
    let developer_actor = Actor {
        principal: &developer,
        ip_address: None,
    };
    let local = NewDestinationInput {
        name: "Local disk".to_owned(),
        path_prefix: Some("/prod/".to_owned()),
        s3: None,
    };
    let denied = service
        .create_destination(developer_actor, local.clone())
        .await;
    assert_eq!(denied.err().map(|e| e.code()), Some("FORBIDDEN"));
    let destination = service.create_destination(actor, local).await?;
    assert_eq!(destination.path_prefix, "prod");

    let bad = PolicyInput {
        destination_id: destination.id,
        schedule: "every day".to_owned(),
        retention: 7,
        enabled: true,
    };
    let rejected = service.set_policy(actor, database_id, bad).await;
    assert_eq!(
        rejected.err().map(|e| e.code()),
        Some("BACKUP_SCHEDULE_INVALID")
    );
    let input = PolicyInput {
        destination_id: destination.id,
        schedule: "0 3 * * *".to_owned(),
        retention: 7,
        enabled: true,
    };
    let policy = service.set_policy(actor, database_id, input).await?;
    assert!(policy.next_run_at > OffsetDateTime::now_utc());
    let now = OffsetDateTime::now_utc();
    assert_eq!(service.run_due(now).await?, 0);

    let overdue = PolicyRecord {
        database_id,
        destination_id: destination.id,
        schedule: policy.schedule.clone(),
        retention: 7,
        enabled: true,
        next_run_at: now - Duration::minutes(5),
    };
    store.save_policy(overdue).await?;
    assert_eq!(service.run_due(now).await?, 1);
    assert_eq!(service.run_due(now).await?, 0);

    let manual = service
        .back_up_now(actor, database_id, destination.id)
        .await?;
    assert_eq!(manual.status, "queued");
    let runs = service.runs(&owner, database_id).await?;
    assert_eq!(runs.len(), 2);
    let restore = service.restore(actor, manual.id).await;
    assert_eq!(restore.err().map(|e| e.code()), Some("BACKUP_NOT_FOUND"));

    let busy = service.delete_destination(actor, destination.id).await;
    assert_eq!(
        busy.err().map(|e| e.code()),
        Some("BACKUP_DESTINATION_IN_USE")
    );
    let application = common::application_in(&store, actor, "Billing").await?;
    let wrong = service.policy(&owner, application).await;
    assert_eq!(wrong.err().map(|e| e.code()), Some("DATABASE_NOT_FOUND"));
    service.remove_policy(actor, database_id).await?;
    service.delete_destination(actor, destination.id).await?;
    assert!(service.destinations(&owner).await?.is_empty());
    Ok(())
}
