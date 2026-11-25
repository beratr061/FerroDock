use std::sync::Arc;

use async_trait::async_trait;
use ferrodock_core::AppResult;
use ferrodock_domain::auth::AuthService;

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
