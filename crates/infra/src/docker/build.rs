use std::path::Path;

use ferrodock_core::AppResult;
use tokio::process::Command;
use tokio_util::sync::CancellationToken;

use crate::deploy_log::DeployLog;
use crate::process::{capture, run_logged};

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
    pub file: &'a Path,
    pub directory: &'a Path,
}

fn docker() -> Command {
    let mut command = Command::new("docker");
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

pub async fn compose_up(
    compose: ComposeUp<'_>,
    log: &DeployLog,
    cancel: &CancellationToken,
) -> AppResult<()> {
    let mut command = docker();
    command
        .args(["compose", "-p", compose.project, "-f"])
        .arg(compose.file)
        .arg("--project-directory")
        .arg(compose.directory)
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
