use std::io::Write;

use clap::Subcommand;
use ferrodock_core::{AppError, AppResult};
use std::path::{Path, PathBuf};

use ferrodock_engine::updater::{
    DEFAULT_MANIFEST_URL, ManifestClient, UpdateState, apply, check_for_update, layout,
    release_key, running_version,
};
use ferrodock_infra::docker::DockerEngine;

#[derive(Debug, Subcommand)]
pub enum UpdateCommand {
    Check {
        #[arg(long, env = "FERRODOCK_UPDATE_MANIFEST_URL", default_value = DEFAULT_MANIFEST_URL)]
        manifest_url: String,
    },
    Apply {
        #[arg(long)]
        image: String,
        #[arg(long, default_value = "/etc/ferrodock")]
        config_dir: PathBuf,
    },
}

pub async fn run(command: &UpdateCommand) -> AppResult<()> {
    match command {
        UpdateCommand::Check { manifest_url } => check(manifest_url).await,
        UpdateCommand::Apply { image, config_dir } => apply_update(image, config_dir).await,
    }
}

async fn apply_update(image: &str, config_dir: &Path) -> AppResult<()> {
    let docker = DockerEngine::connect_local()?;
    let record = apply(&docker, &layout(config_dir), image).await?;
    let rendered = serde_json::to_string_pretty(&record).map_err(AppError::internal)?;
    writeln!(std::io::stdout().lock(), "{rendered}").map_err(AppError::internal)?;
    if record.state == UpdateState::Succeeded {
        Ok(())
    } else {
        Err(AppError::validation("UPDATE_FAILED")
            .with_param("state", format!("{:?}", record.state)))
    }
}

async fn check(manifest_url: &str) -> AppResult<()> {
    let key = release_key()?;
    let user_agent = concat!("ferrodock/", env!("CARGO_PKG_VERSION"));
    let client = ManifestClient::new(manifest_url, user_agent)?;
    let status = check_for_update(&client, &key, &running_version()).await?;
    let rendered = serde_json::to_string_pretty(&status).map_err(AppError::internal)?;
    let mut stdout = std::io::stdout().lock();
    writeln!(stdout, "{rendered}").map_err(AppError::internal)
}
