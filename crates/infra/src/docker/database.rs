use std::collections::HashMap;

use bollard::container::LogOutput;
use bollard::exec::{StartExecOptions, StartExecResults};
use bollard::models::ExecConfig;
use bollard::query_parameters::ListContainersOptionsBuilder;
use ferrodock_core::{AppError, AppResult};
use std::pin::Pin;

use futures_util::{Stream, StreamExt};
use tokio::io::AsyncWriteExt;

use super::exec::ShellInput;
use super::{COMPOSE_PROJECT_LABEL, DockerEngine, docker_error};

const COMPOSE_SERVICE_LABEL: &str = "com.docker.compose.service";
const MAX_ERROR_OUTPUT: usize = 2_000;

#[derive(Debug, Clone)]
pub struct DatabaseTarget {
    pub container: String,
    pub user: String,
    pub database: String,
}

struct ExecOutcome {
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    exit_code: i64,
}

fn failure(code: &'static str, outcome: &ExecOutcome) -> AppError {
    let stderr = String::from_utf8_lossy(&outcome.stderr);
    let detail: String = stderr.chars().take(MAX_ERROR_OUTPUT).collect();
    AppError::validation(code)
        .with_param("exit_code", outcome.exit_code)
        .with_param("detail", detail.trim())
}

fn exec_config(command: Vec<String>, with_input: bool) -> ExecConfig {
    ExecConfig {
        attach_stdin: Some(with_input),
        attach_stdout: Some(true),
        attach_stderr: Some(true),
        tty: Some(false),
        cmd: Some(command),
        ..ExecConfig::default()
    }
}

async fn feed(mut writer: ShellInput, input: Option<Vec<u8>>) {
    if let Some(bytes) = input {
        let _ = writer.write_all(&bytes).await;
    }
    let _ = writer.shutdown().await;
}

async fn collect(
    mut output: Pin<Box<dyn Stream<Item = Result<LogOutput, bollard::errors::Error>> + Send>>,
) -> AppResult<ExecOutcome> {
    let mut outcome = ExecOutcome {
        stdout: Vec::new(),
        stderr: Vec::new(),
        exit_code: -1,
    };
    while let Some(chunk) = output.next().await {
        match chunk.map_err(docker_error)? {
            LogOutput::StdOut { message } => outcome.stdout.extend_from_slice(&message),
            LogOutput::StdErr { message } => outcome.stderr.extend_from_slice(&message),
            _ => {}
        }
    }
    Ok(outcome)
}

impl DockerEngine {
    pub async fn compose_service_container(
        &self,
        project: &str,
        service: &str,
    ) -> AppResult<Option<String>> {
        let project_filter = format!("{COMPOSE_PROJECT_LABEL}={project}");
        let service_filter = format!("{COMPOSE_SERVICE_LABEL}={service}");
        let filters = HashMap::from([(
            "label",
            vec![project_filter.as_str(), service_filter.as_str()],
        )]);
        let options = ListContainersOptionsBuilder::new()
            .filters(&filters)
            .build();
        let containers = self
            .client
            .list_containers(Some(options))
            .await
            .map_err(docker_error)?;
        Ok(containers.into_iter().find_map(|summary| summary.id))
    }

    async fn run_exec(
        &self,
        container: &str,
        command: Vec<String>,
        input: Option<Vec<u8>>,
    ) -> AppResult<ExecOutcome> {
        let config = exec_config(command, input.is_some());
        let created = self
            .client
            .create_exec(container, config)
            .await
            .map_err(docker_error)?;
        let options = StartExecOptions {
            detach: false,
            tty: false,
            output_capacity: None,
        };
        let started = self
            .client
            .start_exec(&created.id, Some(options))
            .await
            .map_err(docker_error)?;
        let StartExecResults::Attached {
            output,
            input: writer,
        } = started
        else {
            return Err(AppError::conflict("SHELL_DETACHED"));
        };
        let feeder = tokio::spawn(feed(writer, input));
        let mut outcome = collect(output).await?;
        let _ = feeder.await;
        let details = self
            .client
            .inspect_exec(&created.id)
            .await
            .map_err(docker_error)?;
        outcome.exit_code = details.exit_code.unwrap_or(-1);
        Ok(outcome)
    }

    pub async fn dump_with(&self, container: &str, script: &str) -> AppResult<Vec<u8>> {
        let command = vec!["sh".to_owned(), "-c".to_owned(), script.to_owned()];
        let outcome = self.run_exec(container, command, None).await?;
        if outcome.exit_code == 0 && !outcome.stdout.is_empty() {
            Ok(outcome.stdout)
        } else {
            Err(failure("BACKUP_DUMP_FAILED", &outcome))
        }
    }

    pub async fn restore_with(
        &self,
        container: &str,
        script: &str,
        input: Vec<u8>,
    ) -> AppResult<()> {
        let command = vec!["sh".to_owned(), "-c".to_owned(), script.to_owned()];
        let outcome = self.run_exec(container, command, Some(input)).await?;
        if outcome.exit_code == 0 {
            Ok(())
        } else {
            Err(failure("BACKUP_RESTORE_FAILED", &outcome))
        }
    }

    pub async fn pg_dump(&self, target: &DatabaseTarget) -> AppResult<Vec<u8>> {
        let command = [
            "pg_dump",
            "-U",
            &target.user,
            "-d",
            &target.database,
            "--format=custom",
            "--no-owner",
        ]
        .map(str::to_owned)
        .to_vec();
        let outcome = self.run_exec(&target.container, command, None).await?;
        if outcome.exit_code == 0 && !outcome.stdout.is_empty() {
            Ok(outcome.stdout)
        } else {
            Err(failure("BACKUP_DUMP_FAILED", &outcome))
        }
    }

    pub async fn pg_restore(&self, target: &DatabaseTarget, dump: Vec<u8>) -> AppResult<()> {
        let command = [
            "pg_restore",
            "-U",
            &target.user,
            "-d",
            &target.database,
            "--clean",
            "--if-exists",
            "--no-owner",
            "--single-transaction",
        ]
        .map(str::to_owned)
        .to_vec();
        let outcome = self
            .run_exec(&target.container, command, Some(dump))
            .await?;
        if outcome.exit_code == 0 {
            Ok(())
        } else {
            Err(failure("BACKUP_RESTORE_FAILED", &outcome))
        }
    }
}
