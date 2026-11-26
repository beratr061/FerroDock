use std::sync::Arc;

use async_trait::async_trait;
use ferrodock_core::AppResult;
use ferrodock_domain::applications::ApplicationService;
use ferrodock_domain::audit::AuditLog;
use ferrodock_domain::auth::AuthService;
use ferrodock_domain::backups::BackupService;
use ferrodock_domain::database_backups::DatabaseBackupService;
use ferrodock_domain::databases::DatabaseService;
use ferrodock_domain::deployments::DeploymentService;
use ferrodock_domain::domains::DomainService;
use ferrodock_domain::projects::ProjectService;
use ferrodock_domain::runtime_config::RuntimeConfigService;
use ferrodock_domain::terminal::TerminalService;
use ferrodock_domain::variables::VariableService;
use ferrodock_engine::updater::UpdateLauncher;
use ferrodock_infra::docker::DockerEngine;
use ferrodock_infra::realtime::Hub;

#[async_trait]
pub trait HealthProbe: Send + Sync + 'static {
    async fn database(&self) -> AppResult<()>;
}

#[derive(Debug, Clone)]
pub struct ApiSettings {
    pub secure_cookies: bool,
    pub public_origin: Option<String>,
    pub trust_forwarded_for: bool,
    pub version: &'static str,
}

#[derive(Clone)]
pub struct AppState {
    pub auth: AuthService,
    pub docker: DockerEngine,
    pub hub: Hub,
    pub audit: AuditLog,
    pub projects: ProjectService,
    pub applications: ApplicationService,
    pub deployments: DeploymentService,
    pub domains: DomainService,
    pub variables: VariableService,
    pub runtime_configs: RuntimeConfigService,
    pub databases: DatabaseService,
    pub database_backups: DatabaseBackupService,
    pub terminal: TerminalService,
    pub backups: BackupService,
    pub updates: UpdateLauncher,
    pub health: Arc<dyn HealthProbe>,
    pub settings: Arc<ApiSettings>,
}

impl std::fmt::Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppState")
            .field("settings", &self.settings)
            .finish_non_exhaustive()
    }
}
