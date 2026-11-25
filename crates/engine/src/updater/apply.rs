use std::path::{Path, PathBuf};
use std::time::Duration;

use ferrodock_core::{AppError, AppResult};
use ferrodock_infra::docker::{DockerEngine, compose_service_up};
use tokio_util::sync::CancellationToken;

use super::env_file::set_image;
use super::status::{UpdateRecord, UpdateState, write_status};

pub const PANEL_PROJECT: &str = "ferrodock";
pub const PANEL_SERVICE: &str = "ferrodock";
const HEALTH_TIMEOUT: Duration = Duration::from_secs(180);

#[derive(Debug, Clone)]
pub struct InstallLayout {
    pub config_dir: PathBuf,
}

impl InstallLayout {
    pub fn env_file(&self) -> PathBuf {
        self.config_dir.join(".env")
    }

    pub fn compose_file(&self) -> PathBuf {
        self.config_dir.join("docker-compose.yml")
    }

    pub fn status_file(&self) -> PathBuf {
        self.config_dir.join("panel").join("update-status.json")
    }
}

async fn switch(docker: &DockerEngine, layout: &InstallLayout) -> AppResult<()> {
    compose_service_up(&layout.compose_file(), &layout.config_dir, PANEL_SERVICE).await?;
    let container = docker
        .compose_service_container(PANEL_PROJECT, PANEL_SERVICE)
        .await?
        .ok_or_else(|| AppError::validation("UPDATE_PANEL_NOT_FOUND"))?;
    docker
        .wait_ready(&container, HEALTH_TIMEOUT, &CancellationToken::new())
        .await
}

async fn roll_back(
    docker: &DockerEngine,
    layout: &InstallLayout,
    previous: Option<&str>,
) -> UpdateState {
    let Some(previous) = previous else {
        return UpdateState::Failed;
    };
    let restored = match set_image(&layout.env_file(), previous).await {
        Ok(_) => switch(docker, layout).await,
        Err(error) => Err(error),
    };
    if restored.is_ok() {
        UpdateState::RolledBack
    } else {
        UpdateState::Failed
    }
}

pub async fn apply(
    docker: &DockerEngine,
    layout: &InstallLayout,
    image: &str,
) -> AppResult<UpdateRecord> {
    let previous = set_image(&layout.env_file(), image).await?;
    let mut record = UpdateRecord::pending(previous.clone(), image.to_owned());
    record.state = UpdateState::Applying;
    write_status(&layout.status_file(), &record).await?;
    let record = match switch(docker, layout).await {
        Ok(()) => record.finish(UpdateState::Succeeded, None),
        Err(error) => {
            tracing::error!(
                code = error.code(),
                "new panel did not become healthy, rolling back"
            );
            let state = roll_back(docker, layout, previous.as_deref()).await;
            record.finish(state, Some(error.code()))
        }
    };
    write_status(&layout.status_file(), &record).await?;
    Ok(record)
}

pub fn layout(config_dir: &Path) -> InstallLayout {
    InstallLayout {
        config_dir: config_dir.to_path_buf(),
    }
}
