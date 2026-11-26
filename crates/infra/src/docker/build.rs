use std::path::Path;

use ferrodock_core::{AppError, AppResult, Id};
use tokio::process::Command;
use tokio_util::sync::CancellationToken;

use super::DockerEngine;
use super::compose_override::{OVERRIDE_FILE, render_override};
use super::compose_policy::audit_compose;
use super::compose_runner::ComposeRun;
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
    pub trusted: bool,
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

fn compose_run<'a>(
    compose: &ComposeUp<'a>,
    files: &[&Path],
    action: &[&str],
    with_daemon: bool,
) -> ComposeRun<'a> {
    let mut arguments = vec!["-p".to_owned(), compose.project.to_owned()];
    for file in files {
        arguments.extend(["-f".to_owned(), file.to_string_lossy().into_owned()]);
    }
    arguments.extend([
        "--project-directory".to_owned(),
        compose.directory.to_string_lossy().into_owned(),
    ]);
    arguments.extend(action.iter().map(|part| (*part).to_owned()));
    ComposeRun {
        directory: compose.directory,
        arguments,
        variables: compose.variables,
        with_daemon,
    }
}

async fn checked_config(
    docker: &DockerEngine,
    compose: &ComposeUp<'_>,
    log: &DeployLog,
) -> AppResult<String> {
    let inspect = compose_run(
        compose,
        &[compose.file],
        &["config", "--format", "json"],
        false,
    );
    let rendered = docker.compose_config(&inspect).await?;
    if compose.trusted {
        log.step("Trusted stack: compose security checks skipped")
            .await;
    } else {
        audit_compose(&rendered, compose.directory)?;
        log.step("Compose file passed the security checks").await;
    }
    Ok(rendered)
}

pub async fn compose_up(
    docker: &DockerEngine,
    compose: ComposeUp<'_>,
    log: &DeployLog,
    cancel: &CancellationToken,
) -> AppResult<()> {
    let rendered = checked_config(docker, &compose, log).await?;
    let document = render_override(&rendered, compose.application_id)?;
    let override_file = compose.directory.join(OVERRIDE_FILE);
    tokio::fs::write(&override_file, document)
        .await
        .map_err(AppError::internal)?;
    let up = [
        "up",
        "-d",
        "--build",
        "--remove-orphans",
        "--wait",
        "--wait-timeout",
        COMPOSE_WAIT_SECONDS,
    ];
    let run = compose_run(&compose, &[compose.file, &override_file], &up, true);
    log.step(&format!("Starting compose project {}", compose.project))
        .await;
    docker.compose_logged(&run, log, cancel).await
}

pub async fn compose_down(project: &str, remove_volumes: bool) -> AppResult<()> {
    let mut command = docker();
    command.args(["compose", "-p", project, "down", "--remove-orphans"]);
    if remove_volumes {
        command.arg("--volumes");
    }
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
