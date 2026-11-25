use std::io::Write;

use clap::Subcommand;
use ferrodock_core::{AppError, AppResult};
use ferrodock_engine::updater::{
    DEFAULT_MANIFEST_URL, ManifestClient, check_for_update, release_key, running_version,
};

#[derive(Debug, Subcommand)]
pub enum UpdateCommand {
    Check {
        #[arg(long, env = "FERRODOCK_UPDATE_MANIFEST_URL", default_value = DEFAULT_MANIFEST_URL)]
        manifest_url: String,
    },
}

pub async fn run(command: &UpdateCommand) -> AppResult<()> {
    match command {
        UpdateCommand::Check { manifest_url } => check(manifest_url).await,
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
