use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use ferrodock_core::config::AppConfig;
use ferrodock_db::pg::PgStore;
use ferrodock_domain::audit::AuditLog;
use ferrodock_domain::database_backups::{DatabaseBackupParts, DatabaseBackupService};
use ferrodock_domain::databases::{DatabaseService, DatabaseServiceParts};
use ferrodock_engine::database_backups::BackupJobContext;
use ferrodock_engine::databases::DatabaseContext;
use ferrodock_engine::deploy::DeployContext;
use ferrodock_infra::deploy_log::DeployLogs;

use time::OffsetDateTime;
use tokio_util::sync::CancellationToken;

use crate::wiring::panel_host;

const SCHEDULE_TICK: Duration = Duration::from_secs(60);

pub fn context(config: &AppConfig, store: &PgStore, deploy: &DeployContext) -> DatabaseContext {
    DatabaseContext {
        docker: deploy.docker.clone(),
        repo: Arc::new(store.clone()),
        secrets: deploy.secrets.clone(),
        logs: DeployLogs::new(config.data_dir.join("database-logs")),
    }
}

pub fn service(
    config: &AppConfig,
    store: &PgStore,
    databases: &DatabaseContext,
    audit: AuditLog,
) -> DatabaseService {
    let public_host = config
        .public_ip
        .map(|ip| ip.to_string())
        .or_else(|| panel_host(config));
    DatabaseService::new(DatabaseServiceParts {
        repo: databases.repo.clone(),
        jobs: Arc::new(store.clone()),
        docker: databases.docker.clone(),
        secrets: databases.secrets.clone(),
        logs: databases.logs.clone(),
        audit,
        public_host,
    })
}

pub fn backup_root(config: &AppConfig) -> PathBuf {
    config.data_dir.join("database-backups")
}

pub fn backup_jobs(
    config: &AppConfig,
    store: &PgStore,
    databases: &DatabaseContext,
) -> BackupJobContext {
    let shared = Arc::new(store.clone());
    BackupJobContext {
        docker: databases.docker.clone(),
        databases: shared.clone(),
        destinations: shared.clone(),
        policies: shared.clone(),
        runs: shared,
        secrets: databases.secrets.clone(),
        local_root: backup_root(config),
    }
}

pub fn backup_service(
    config: &AppConfig,
    store: &PgStore,
    databases: &DatabaseContext,
    audit: AuditLog,
) -> DatabaseBackupService {
    let shared = Arc::new(store.clone());
    DatabaseBackupService::new(DatabaseBackupParts {
        databases: shared.clone(),
        destinations: shared.clone(),
        policies: shared.clone(),
        runs: shared.clone(),
        jobs: shared,
        secrets: databases.secrets.clone(),
        audit,
        local_root: backup_root(config),
    })
}

pub fn schedule_backups(service: DatabaseBackupService, shutdown: CancellationToken) {
    tokio::spawn(async move {
        loop {
            match service.run_due(OffsetDateTime::now_utc()).await {
                Ok(0) => {}
                Ok(queued) => tracing::info!(queued, "scheduled database backups queued"),
                Err(error) => tracing::warn!(code = error.code(), "backup schedule check failed"),
            }
            tokio::select! {
                () = shutdown.cancelled() => return,
                () = tokio::time::sleep(SCHEDULE_TICK) => {}
            }
        }
    });
}
