use std::sync::Arc;

use ferrodock_core::config::AppConfig;
use ferrodock_db::pg::PgStore;
use ferrodock_domain::audit::AuditLog;
use ferrodock_domain::databases::{DatabaseService, DatabaseServiceParts};
use ferrodock_engine::databases::DatabaseContext;
use ferrodock_engine::deploy::DeployContext;
use ferrodock_infra::deploy_log::DeployLogs;

use crate::wiring::panel_host;

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
