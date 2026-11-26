use std::collections::HashMap;
use std::path::Path;

use bollard::container::LogOutput;
use bollard::models::{ContainerCreateBody, HostConfig, Mount, MountType};
use bollard::query_parameters::{
    CreateContainerOptionsBuilder, LogsOptionsBuilder, RemoveContainerOptionsBuilder,
    WaitContainerOptions,
};
use ferrodock_core::{AppError, AppResult};
use futures_util::StreamExt;
use tokio_util::sync::CancellationToken;

use super::{DockerEngine, MANAGED_LABEL, docker_error};
use crate::deploy_log::DeployLog;
use crate::process::cancelled;

pub const COMPOSE_IMAGE: &str = "docker:29.8-cli";
const DOCKER_SOCKET: &str = "/var/run/docker.sock";
const MAX_CAPTURE: usize = 8 * 1024 * 1024;
const MAX_ERROR_OUTPUT: usize = 2_000;

#[derive(Debug, Clone)]
pub struct ComposeRun<'a> {
    pub directory: &'a Path,
    pub arguments: Vec<String>,
    pub variables: &'a [(String, String)],
    pub with_daemon: bool,
}

#[derive(Default)]
struct Captured {
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

fn bind(source: &str, read_only: bool) -> Mount {
    Mount {
        target: Some(source.to_owned()),
        source: Some(source.to_owned()),
        typ: Some(MountType::BIND),
        read_only: Some(read_only),
        ..Mount::default()
    }
}

fn body(run: &ComposeRun<'_>) -> ContainerCreateBody {
    let directory = run.directory.to_string_lossy().into_owned();
    let mut mounts = vec![bind(&directory, true)];
    if run.with_daemon {
        mounts.push(bind(DOCKER_SOCKET, false));
    }
    let mut env: Vec<String> = run
        .variables
        .iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect();
    env.extend([
        "DOCKER_BUILDKIT=1".to_owned(),
        "BUILDKIT_PROGRESS=plain".to_owned(),
    ]);
    let mut command = vec!["compose".to_owned()];
    command.extend(run.arguments.iter().cloned());
    ContainerCreateBody {
        image: Some(COMPOSE_IMAGE.to_owned()),
        entrypoint: Some(vec!["docker".to_owned()]),
        cmd: Some(command),
        env: Some(env),
        working_dir: Some(directory),
        labels: Some(HashMap::from([(
            MANAGED_LABEL.to_owned(),
            "true".to_owned(),
        )])),
        host_config: Some(HostConfig {
            mounts: Some(mounts),
            network_mode: Some("none".to_owned()),
            ..HostConfig::default()
        }),
        ..ContainerCreateBody::default()
    }
}

fn keep(buffer: &mut Vec<u8>, bytes: &[u8]) {
    let room = MAX_CAPTURE.saturating_sub(buffer.len());
    buffer.extend_from_slice(&bytes[..bytes.len().min(room)]);
}

fn failure(code: &'static str, exit_code: i64, stderr: &[u8]) -> AppError {
    let text = String::from_utf8_lossy(stderr);
    let start = text.len().saturating_sub(MAX_ERROR_OUTPUT);
    let detail = text.get(start..).unwrap_or_default().trim().to_owned();
    AppError::validation(code)
        .with_param("exit_code", exit_code)
        .with_param("detail", detail)
}

impl DockerEngine {
    async fn start_compose(&self, run: &ComposeRun<'_>) -> AppResult<String> {
        self.ensure_image(COMPOSE_IMAGE).await?;
        let options = CreateContainerOptionsBuilder::new().build();
        let created = self
            .client
            .create_container(Some(options), body(run))
            .await
            .map_err(docker_error)?;
        if let Err(error) = self.start_workload(&created.id).await {
            self.remove_volume_helper(&created.id).await;
            return Err(error);
        }
        Ok(created.id)
    }

    async fn follow_compose(
        &self,
        id: &str,
        log: Option<&DeployLog>,
        cancel: &CancellationToken,
    ) -> AppResult<Captured> {
        let options = LogsOptionsBuilder::new()
            .stdout(true)
            .stderr(true)
            .follow(true)
            .build();
        let mut stream = self.client.logs(id, Some(options));
        let mut captured = Captured::default();
        loop {
            let next = tokio::select! {
                () = cancel.cancelled() => return Err(cancelled()),
                next = stream.next() => next,
            };
            let Some(chunk) = next else {
                return Ok(captured);
            };
            let message = match chunk.map_err(docker_error)? {
                LogOutput::StdOut { message } => {
                    keep(&mut captured.stdout, &message);
                    message
                }
                LogOutput::StdErr { message } => {
                    keep(&mut captured.stderr, &message);
                    message
                }
                _ => continue,
            };
            if let Some(log) = log {
                for line in String::from_utf8_lossy(&message).lines() {
                    log.line(line).await;
                }
            }
        }
    }

    async fn exit_code(&self, id: &str) -> i64 {
        let mut waits = self.client.wait_container(id, None::<WaitContainerOptions>);
        match waits.next().await {
            Some(Ok(response)) => response.status_code,
            Some(Err(bollard::errors::Error::DockerContainerWaitError { code, .. })) => code,
            _ => -1,
        }
    }

    async fn run_compose(
        &self,
        run: &ComposeRun<'_>,
        log: Option<&DeployLog>,
        cancel: &CancellationToken,
        code: &'static str,
    ) -> AppResult<Vec<u8>> {
        let id = self.start_compose(run).await?;
        let followed = self.follow_compose(&id, log, cancel).await;
        let exit_code = self.exit_code(&id).await;
        let remove = RemoveContainerOptionsBuilder::new().force(true).build();
        let _ = self.client.remove_container(&id, Some(remove)).await;
        let captured = followed?;
        if exit_code == 0 {
            Ok(captured.stdout)
        } else {
            Err(failure(code, exit_code, &captured.stderr))
        }
    }

    pub async fn compose_config(&self, run: &ComposeRun<'_>) -> AppResult<String> {
        let stdout = self
            .run_compose(
                run,
                None,
                &CancellationToken::new(),
                "COMPOSE_CONFIG_INVALID",
            )
            .await?;
        String::from_utf8(stdout).map_err(|_| AppError::validation("COMPOSE_CONFIG_INVALID"))
    }

    pub async fn compose_logged(
        &self,
        run: &ComposeRun<'_>,
        log: &DeployLog,
        cancel: &CancellationToken,
    ) -> AppResult<()> {
        self.run_compose(run, Some(log), cancel, "COMPOSE_UP_FAILED")
            .await
            .map(|_| ())
    }
}
