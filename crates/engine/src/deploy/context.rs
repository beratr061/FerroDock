use std::path::PathBuf;
use std::sync::Arc;

use ferrodock_core::Id;
use ferrodock_core::crypto::SecretBox;
use ferrodock_db::{ApplicationRepo, DeploymentRepo};
use ferrodock_infra::deploy_log::DeployLogs;
use ferrodock_infra::docker::DockerEngine;

pub const KEPT_IMAGES: usize = 5;

#[derive(Clone)]
pub struct DeployContext {
    pub docker: DockerEngine,
    pub applications: Arc<dyn ApplicationRepo>,
    pub deployments: Arc<dyn DeploymentRepo>,
    pub secrets: SecretBox,
    pub logs: DeployLogs,
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
    pub application_key: String,
    pub deployment_key: String,
    pub slug: String,
}

fn short_key(id: Id, length: usize) -> String {
    let simple = id.as_uuid().simple().to_string();
    simple[simple.len() - length..].to_owned()
}

impl Names {
    pub fn new(application_id: Id, deployment_id: Id, slug: &str) -> Self {
        Self {
            application_key: short_key(application_id, 12),
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

    pub fn alias(&self) -> String {
        format!("fd-{}", self.application_key)
    }

    pub fn compose_project(&self) -> String {
        format!("fd-{}", self.application_key)
    }
}

impl DeployContext {
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
