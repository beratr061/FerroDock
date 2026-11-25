use std::path::PathBuf;
use std::sync::Arc;

use ferrodock_core::{AppError, AppResult};
use ferrodock_db::JobQueue;
use ferrodock_db::deployments::DEPLOYMENT_JOB;
use ferrodock_infra::docker::{DockerEngine, UpdaterSpec};

use super::apply::{PANEL_PROJECT, PANEL_SERVICE};
use super::status::{UpdateRecord, read_status, write_status};
use super::{ManifestClient, UpdateStatus, check_for_update, release_key, running_version};
use crate::backup::BackupContext;

#[derive(Clone)]
pub struct UpdateLauncher {
    pub docker: DockerEngine,
    pub jobs: Arc<dyn JobQueue>,
    pub backup: BackupContext,
    pub status_file: PathBuf,
    pub host_config_dir: String,
    pub manifest_url: String,
}

impl std::fmt::Debug for UpdateLauncher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UpdateLauncher")
            .field("status_file", &self.status_file)
            .finish_non_exhaustive()
    }
}

impl UpdateLauncher {
    pub async fn status(&self) -> AppResult<Option<UpdateRecord>> {
        read_status(&self.status_file).await
    }

    pub async fn check(&self) -> AppResult<UpdateStatus> {
        let user_agent = concat!("ferrodock/", env!("CARGO_PKG_VERSION"));
        let client = ManifestClient::new(&self.manifest_url, user_agent)?;
        check_for_update(&client, &release_key()?, &running_version()).await
    }

    async fn current_image(&self) -> AppResult<String> {
        let container = self
            .docker
            .compose_service_container(PANEL_PROJECT, PANEL_SERVICE)
            .await?
            .ok_or_else(|| AppError::validation("UPDATE_PANEL_NOT_FOUND"))?;
        self.docker
            .container_image(&container)
            .await?
            .ok_or_else(|| AppError::validation("UPDATE_PANEL_NOT_FOUND"))
    }

    async fn ensure_idle(&self) -> AppResult<()> {
        if self.docker.updater_running().await? {
            return Err(AppError::conflict("UPDATE_IN_PROGRESS"));
        }
        if self.jobs.active_count(DEPLOYMENT_JOB).await? > 0 {
            return Err(AppError::conflict("UPDATE_DEPLOYMENTS_ACTIVE"));
        }
        Ok(())
    }

    async fn launch(&self, target: String) -> AppResult<UpdateRecord> {
        self.ensure_idle().await?;
        let current = self.current_image().await?;
        self.docker.ensure_image(&target).await?;
        self.backup.create().await?;
        let record = UpdateRecord::pending(Some(current), target.clone());
        write_status(&self.status_file, &record).await?;
        let spec = UpdaterSpec {
            image: target.clone(),
            args: [
                "update",
                "apply",
                "--image",
                &target,
                "--config-dir",
                &self.host_config_dir,
            ]
            .map(str::to_owned)
            .to_vec(),
            config_dir: self.host_config_dir.clone(),
        };
        self.docker.start_updater(&spec).await?;
        Ok(record)
    }

    pub async fn update(&self) -> AppResult<UpdateRecord> {
        match self.check().await? {
            UpdateStatus::Available { image, .. } => self.launch(image).await,
            UpdateStatus::UpToDate { .. } => Err(AppError::conflict("UPDATE_NOT_AVAILABLE")),
            UpdateStatus::RequiresIntermediate {
                min_from_version, ..
            } => Err(AppError::conflict("UPDATE_REQUIRES_INTERMEDIATE")
                .with_param("version", min_from_version.to_string())),
        }
    }

    pub async fn roll_back(&self) -> AppResult<UpdateRecord> {
        let previous = self
            .status()
            .await?
            .and_then(|record| record.from_image)
            .ok_or_else(|| AppError::conflict("UPDATE_NOTHING_TO_ROLL_BACK"))?;
        self.launch(previous).await
    }
}
