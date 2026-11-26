mod backups;
mod container_events;
mod databases;
mod healthcheck;
mod jobs;
mod serve;
mod telemetry;
mod update;
mod wiring;

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};
use ferrodock_core::AppResult;
use ferrodock_core::config::AppConfig;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[derive(Debug, Parser)]
#[command(
    name = "ferrodock",
    version,
    about = "Lightweight self-hosted PaaS control plane"
)]
struct Cli {
    #[arg(long, env = "FERRODOCK_CONFIG", global = true)]
    config: Option<PathBuf>,
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Serve,
    Migrate,
    Openapi,
    Update {
        #[command(subcommand)]
        action: update::UpdateCommand,
    },
    Healthcheck {
        #[arg(long, default_value = "127.0.0.1:3000")]
        address: String,
    },
    Backup,
    Restore {
        archive: PathBuf,
        #[arg(long)]
        master_key: PathBuf,
    },
}

#[tokio::main]
async fn main() -> ExitCode {
    let cli = Cli::parse();
    telemetry::init();
    match run(cli).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            tracing::error!(code = error.code(), error = ?error, "ferrodock exited with an error");
            ExitCode::FAILURE
        }
    }
}

async fn run(cli: Cli) -> AppResult<()> {
    match &cli.command {
        Command::Openapi => return serve::print_openapi(),
        Command::Healthcheck { address } => return healthcheck::probe(address).await,
        Command::Update { action } => return update::run(action).await,
        Command::Serve | Command::Migrate | Command::Backup | Command::Restore { .. } => {}
    }
    let config = AppConfig::load(cli.config.as_deref())?;
    match cli.command {
        Command::Serve => serve::run(config).await,
        Command::Migrate => wiring::migrate_only(&config).await,
        Command::Backup => backups::create_now(&config).await,
        Command::Restore {
            archive,
            master_key,
        } => backups::restore(&config, &archive, &master_key).await,
        Command::Openapi | Command::Healthcheck { .. } | Command::Update { .. } => Ok(()),
    }
}
