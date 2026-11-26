use std::pin::Pin;

use bollard::exec::{StartExecOptions, StartExecResults};
use bollard::models::ExecConfig;
use bollard::query_parameters::ResizeExecOptionsBuilder;
use ferrodock_core::{AppError, AppResult};
use futures_util::{Stream, StreamExt};
use tokio::io::AsyncWrite;

use super::workloads::APPLICATION_LABEL;
use super::{
    COMPOSE_PROJECT_LABEL, CONTROL_PLANE_PROJECT, DockerEngine, docker_error, validate_reference,
};

const SHELL_PROBE: &str = "if command -v bash >/dev/null 2>&1; then exec bash; \
    elif command -v zsh >/dev/null 2>&1; then exec zsh; \
    elif command -v ash >/dev/null 2>&1; then exec ash; else exec sh; fi";

pub type ShellOutput = Pin<Box<dyn Stream<Item = AppResult<Vec<u8>>> + Send>>;
pub type ShellInput = Pin<Box<dyn AsyncWrite + Send>>;

pub struct ShellSession {
    pub exec_id: String,
    pub output: ShellOutput,
    pub input: ShellInput,
}

impl std::fmt::Debug for ShellSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ShellSession")
            .field("exec_id", &self.exec_id)
            .finish_non_exhaustive()
    }
}

#[derive(Debug, Clone)]
pub struct ShellTarget {
    pub container_id: String,
    pub application_id: Option<String>,
    pub running: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct ShellSize {
    pub columns: u16,
    pub rows: u16,
}

fn exec_config(size: ShellSize) -> ExecConfig {
    ExecConfig {
        attach_stdin: Some(true),
        attach_stdout: Some(true),
        attach_stderr: Some(true),
        tty: Some(true),
        console_size: Some(vec![usize::from(size.rows), usize::from(size.columns)]),
        env: Some(vec![
            "TERM=xterm-256color".to_owned(),
            "COLORTERM=truecolor".to_owned(),
        ]),
        cmd: Some(vec![
            "/bin/sh".to_owned(),
            "-c".to_owned(),
            SHELL_PROBE.to_owned(),
        ]),
        ..ExecConfig::default()
    }
}

impl DockerEngine {
    pub async fn shell_target(&self, reference: &str) -> AppResult<ShellTarget> {
        let reference = validate_reference(reference)?;
        let details = self
            .client
            .inspect_container(reference, None)
            .await
            .map_err(docker_error)?;
        let labels = details
            .config
            .and_then(|config| config.labels)
            .unwrap_or_default();
        if labels.get(COMPOSE_PROJECT_LABEL).map(String::as_str) == Some(CONTROL_PLANE_PROJECT) {
            return Err(AppError::forbidden().with_param("reason", "CONTAINER_PROTECTED"));
        }
        Ok(ShellTarget {
            container_id: details.id.unwrap_or_default(),
            application_id: labels.get(APPLICATION_LABEL).cloned(),
            running: details
                .state
                .and_then(|state| state.running)
                .unwrap_or(false),
        })
    }

    pub async fn open_shell(
        &self,
        target: &ShellTarget,
        size: ShellSize,
    ) -> AppResult<ShellSession> {
        if !target.running {
            return Err(AppError::conflict("CONTAINER_NOT_RUNNING"));
        }
        let created = self
            .client
            .create_exec(&target.container_id, exec_config(size))
            .await
            .map_err(docker_error)?;
        let options = StartExecOptions {
            detach: false,
            tty: true,
            output_capacity: None,
        };
        match self
            .client
            .start_exec(&created.id, Some(options))
            .await
            .map_err(docker_error)?
        {
            StartExecResults::Attached { output, input } => Ok(ShellSession {
                exec_id: created.id,
                output: Box::pin(output.map(|item| {
                    item.map(|chunk| chunk.as_ref().to_vec())
                        .map_err(docker_error)
                })),
                input,
            }),
            StartExecResults::Detached => Err(AppError::conflict("SHELL_DETACHED")),
        }
    }

    pub async fn resize_shell(&self, exec_id: &str, size: ShellSize) -> AppResult<()> {
        let options = ResizeExecOptionsBuilder::new()
            .h(i32::from(size.rows))
            .w(i32::from(size.columns))
            .build();
        self.client
            .resize_exec(exec_id, options)
            .await
            .map_err(docker_error)
    }

    pub async fn shell_exit_code(&self, exec_id: &str) -> Option<i64> {
        self.client
            .inspect_exec(exec_id)
            .await
            .ok()
            .and_then(|details| details.exit_code)
    }
}
