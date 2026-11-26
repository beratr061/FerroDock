use std::path::Path;

use ferrodock_core::{AppError, AppResult, Id};
use tokio::process::Command;
use tokio_util::sync::CancellationToken;

use super::compose_override::{OVERRIDE_FILE, render_override};
use crate::deploy_log::DeployLog;
use crate::process::{capture, isolated, run_logged};

const COMPOSE_WAIT_SECONDS: &str = "300";

#[derive(Debug)]
pub struct ImageBuild<'a> {
    pub dockerfile: &'a Path,
    pub context: &'a Path,
    pub tag: &'a str,
    pub labels: &'a [(&'a str, &'a str)],
}

#[derive(Debug)]
pub struct ComposeUp<'a> {
    pub project: &'a str,
    pub variables: &'a [(String, String)],
    pub file: &'a Path,
    pub directory: &'a Path,
    pub application_id: Id,
}

fn docker() -> Command {
    let mut command = isolated("docker");
    command
        .env("DOCKER_BUILDKIT", "1")
        .env("BUILDKIT_PROGRESS", "plain");
    command
}

pub async fn build_image(
    build: ImageBuild<'_>,
    log: &DeployLog,
    cancel: &CancellationToken,
) -> AppResult<()> {
    let mut command = docker();
    command
        .args(["build", "--progress=plain", "--pull", "-f"])
        .arg(build.dockerfile);
    command.args(["-t", build.tag]);
    for (key, value) in build.labels {
        command.arg("--label").arg(format!("{key}={value}"));
    }
    command.arg(build.context);
    log.step(&format!("Building image {}", build.tag)).await;
    run_logged(command, log, cancel, "IMAGE_BUILD_FAILED").await
}

fn compose_command(compose: &ComposeUp<'_>, files: &[&Path]) -> Command {
    let mut command = docker();
    command.args(["compose", "-p", compose.project]);
    for file in files {
        command.arg("-f").arg(file);
    }
    command
        .arg("--project-directory")
        .arg(compose.directory)
        .envs(compose.variables.iter().map(|(key, value)| (key, value)));
    command
}

async fn write_override(compose: &ComposeUp<'_>) -> AppResult<std::path::PathBuf> {
    let mut inspect = compose_command(compose, &[compose.file]);
    inspect.args(["config", "--format", "json"]);
    let rendered = capture(inspect, "COMPOSE_CONFIG_INVALID").await?;
    let document = render_override(&rendered, compose.application_id)?;
    let path = compose.directory.join(OVERRIDE_FILE);
    tokio::fs::write(&path, document)
        .await
        .map_err(AppError::internal)?;
    Ok(path)
}

pub async fn compose_up(
    compose: ComposeUp<'_>,
    log: &DeployLog,
    cancel: &CancellationToken,
) -> AppResult<()> {
    let override_file = write_override(&compose).await?;
    let mut command = compose_command(&compose, &[compose.file, &override_file]);
    command
        .args(["up", "-d", "--build", "--remove-orphans", "--wait"])
        .args(["--wait-timeout", COMPOSE_WAIT_SECONDS]);
    log.step(&format!("Starting compose project {}", compose.project))
        .await;
    run_logged(command, log, cancel, "COMPOSE_UP_FAILED").await
}

pub async fn compose_down(project: &str) -> AppResult<()> {
    let mut command = docker();
    command.args(["compose", "-p", project, "down", "--remove-orphans"]);
    capture(command, "COMPOSE_DOWN_FAILED").await.map(|_| ())
}

pub async fn compose_service_up(file: &Path, directory: &Path, service: &str) -> AppResult<()> {
    let mut command = docker();
    command
        .args(["compose", "-f"])
        .arg(file)
        .arg("--project-directory")
        .arg(directory)
        .args(["up", "-d", "--no-deps", service]);
    capture(command, "COMPOSE_UP_FAILED").await.map(|_| ())
}
