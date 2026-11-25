use std::process::Stdio;

use ferrodock_core::{AppError, AppResult, ErrorKind};
use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};
use tokio::process::Command;
use tokio_util::sync::CancellationToken;

use crate::deploy_log::DeployLog;

pub const CANCELLED: &str = "DEPLOYMENT_CANCELLED";

const INHERITED_VARIABLES: [&str; 14] = [
    "PATH",
    "HOME",
    "DOCKER_HOST",
    "DOCKER_CONFIG",
    "DOCKER_CERT_PATH",
    "DOCKER_TLS_VERIFY",
    "SYSTEMROOT",
    "TEMP",
    "TMP",
    "USERPROFILE",
    "APPDATA",
    "LOCALAPPDATA",
    "PROGRAMDATA",
    "PROGRAMFILES",
];

pub fn isolated(program: &str) -> Command {
    let mut command = Command::new(program);
    command.env_clear();
    for name in INHERITED_VARIABLES {
        if let Some(value) = std::env::var_os(name) {
            command.env(name, value);
        }
    }
    command
}

async fn pump(stream: impl AsyncRead + Unpin, log: DeployLog) {
    let mut lines = BufReader::new(stream).lines();
    while let Ok(Some(line)) = lines.next_line().await {
        log.line(&line).await;
    }
}

pub fn cancelled() -> AppError {
    AppError::new(ErrorKind::Conflict, CANCELLED)
}

pub async fn run_logged(
    mut command: Command,
    log: &DeployLog,
    cancel: &CancellationToken,
    failure_code: &'static str,
) -> AppResult<()> {
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    let mut child = command
        .spawn()
        .map_err(|error| AppError::new(ErrorKind::Unavailable, failure_code).with_source(error))?;
    let stdout = child
        .stdout
        .take()
        .map(|stream| tokio::spawn(pump(stream, log.clone())));
    let stderr = child
        .stderr
        .take()
        .map(|stream| tokio::spawn(pump(stream, log.clone())));
    let status = tokio::select! {
        () = cancel.cancelled() => {
            let _ = child.kill().await;
            return Err(cancelled());
        }
        status = child.wait() => status.map_err(AppError::internal)?,
    };
    for task in [stdout, stderr].into_iter().flatten() {
        let _ = task.await;
    }
    if status.success() {
        Ok(())
    } else {
        Err(AppError::validation(failure_code).with_param("exit_code", status.code().unwrap_or(-1)))
    }
}

pub async fn capture(mut command: Command, failure_code: &'static str) -> AppResult<String> {
    command.stdin(Stdio::null()).kill_on_drop(true);
    let output = command
        .output()
        .await
        .map_err(|error| AppError::new(ErrorKind::Unavailable, failure_code).with_source(error))?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
    } else {
        Err(AppError::validation(failure_code))
    }
}
