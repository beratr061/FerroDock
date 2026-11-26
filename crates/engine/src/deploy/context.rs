use std::path::PathBuf;
use std::sync::Arc;

use ferrodock_core::AppResult;
use ferrodock_core::Id;
use ferrodock_core::crypto::SecretBox;
use ferrodock_db::{ApplicationRepo, DeploymentRepo, DomainRepo, RuntimeConfigRepo, VariableRepo};
use ferrodock_infra::deploy_log::DeployLogs;
use ferrodock_infra::docker::{
    DockerEngine, application_key, compose_project, network_alias, short_key,
};
use ferrodock_infra::proxy::ProxyRoutes;
use ferrodock_infra::realtime::{Hub, Signal};

pub const KEPT_IMAGES: usize = 5;

#[derive(Clone)]
pub struct DeployContext {
    pub docker: DockerEngine,
    pub applications: Arc<dyn ApplicationRepo>,
    pub deployments: Arc<dyn DeploymentRepo>,
    pub variables: Arc<dyn VariableRepo>,
    pub domains: Arc<dyn DomainRepo>,
    pub runtime_configs: Arc<dyn RuntimeConfigRepo>,
    pub proxy: ProxyRoutes,
    pub secrets: SecretBox,
    pub logs: DeployLogs,
    pub hub: Hub,
    pub data_dir: PathBuf,
}

impl std::fmt::Debug for DeployContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DeployContext")
            .field("data_dir", &self.data_dir)
            .finish_non_exhaustive()
    }
}

#[derive(Debug, Clone)]
pub struct Names {
    pub application_id: Id,
    pub application_key: String,
    pub deployment_key: String,
    pub slug: String,
}

impl Names {
    pub fn new(application_id: Id, deployment_id: Id, slug: &str) -> Self {
        Self {
            application_id,
            application_key: application_key(application_id),
            deployment_key: short_key(deployment_id, 8),
            slug: slug.to_owned(),
        }
    }

    pub fn image_tag(&self) -> String {
        format!("fd/{}:{}", self.application_key, self.deployment_key)
    }

    pub fn container(&self) -> String {
        format!(
            "{}-{}-{}",
            self.slug, self.application_key, self.deployment_key
        )
    }

    pub fn volume(&self, name: &str) -> String {
        format!("fd-{}-{name}", self.application_key)
    }

    pub fn alias(&self) -> String {
        network_alias(self.application_id)
    }

    pub fn compose_project(&self) -> String {
        compose_project(self.application_id)
    }
}

impl DeployContext {
    pub fn announce(&self, organization_id: Option<Id>, application_id: Id, deployment_id: Id) {
        let signal = Signal::DeploymentStatus {
            application_id,
            deployment_id,
        };
        self.hub.publish(organization_id, signal);
    }

    pub async fn variables(&self, application_id: Id) -> AppResult<Vec<(String, String)>> {
        let records = self.variables.list(application_id).await?;
        records
            .into_iter()
            .map(|record| {
                let bytes = self.secrets.decrypt(&record.value_enc)?;
                let value = String::from_utf8(bytes.to_vec())
                    .map_err(|_| ferrodock_core::AppError::validation("SECRET_CORRUPTED"))?;
                Ok((record.key, value))
            })
            .collect()
    }

    pub fn build_dir(&self, deployment_id: Id) -> PathBuf {
        self.data_dir.join("builds").join(deployment_id.to_string())
    }

    pub fn compose_dir(&self, names: &Names) -> PathBuf {
        self.data_dir
            .join("apps")
            .join(&names.application_key)
            .join("compose")
    }

    pub fn known_hosts(&self) -> PathBuf {
        self.data_dir.join("ssh").join("known_hosts")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derives_stable_docker_names() {
        let names = Names::new(Id::new(), Id::new(), "api");
        assert_eq!(names.application_key.len(), 12);
        assert!(names.image_tag().starts_with("fd/"));
        assert!(names.container().starts_with("api-"));
        assert_eq!(names.alias(), names.compose_project());
    }
}
