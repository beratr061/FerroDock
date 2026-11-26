use std::pin::Pin;
use std::sync::{Arc, Mutex, PoisonError};

use bollard::container::LogOutput;
use bollard::exec::{StartExecOptions, StartExecResults};
use ferrodock_core::{AppError, AppResult};
use futures_util::{Stream, StreamExt};

use super::database::{exec_config, exec_failure};
use super::exec::ShellInput;
use super::{DockerEngine, docker_error};

const STDERR_LIMIT: usize = 4096;

pub type PipeOutput = Pin<Box<dyn Stream<Item = AppResult<Vec<u8>>> + Send>>;

#[derive(Debug, Clone)]
pub struct PipeDone {
    exec_id: String,
    stderr: Arc<Mutex<Vec<u8>>>,
}

pub struct ExecPipe {
    pub stdout: PipeOutput,
    pub stdin: ShellInput,
    pub done: PipeDone,
}

impl std::fmt::Debug for ExecPipe {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExecPipe")
            .field("done", &self.done)
            .finish_non_exhaustive()
    }
}

fn keep_stderr(buffer: &Mutex<Vec<u8>>, message: &[u8]) {
    let mut kept = buffer.lock().unwrap_or_else(PoisonError::into_inner);
    let room = STDERR_LIMIT.saturating_sub(kept.len());
    kept.extend_from_slice(&message[..message.len().min(room)]);
}

type RawOutput = Pin<Box<dyn Stream<Item = Result<LogOutput, bollard::errors::Error>> + Send>>;

fn stdout_only(output: RawOutput, stderr: Arc<Mutex<Vec<u8>>>) -> PipeOutput {
    Box::pin(output.filter_map(move |chunk| {
        let item = match chunk {
            Ok(LogOutput::StdOut { message }) => Some(Ok(message.to_vec())),
            Ok(LogOutput::StdErr { message }) => {
                keep_stderr(&stderr, &message);
                None
            }
            Ok(_) => None,
            Err(error) => Some(Err(docker_error(error))),
        };
        std::future::ready(item)
    }))
}

impl DockerEngine {
    pub async fn open_pipe(
        &self,
        container: &str,
        script: &str,
        with_input: bool,
    ) -> AppResult<ExecPipe> {
        let command = vec!["sh".to_owned(), "-c".to_owned(), script.to_owned()];
        let created = self
            .client
            .create_exec(container, exec_config(command, with_input))
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
        let StartExecResults::Attached { output, input } = started else {
            return Err(AppError::conflict("SHELL_DETACHED"));
        };
        let stderr = Arc::new(Mutex::new(Vec::new()));
        let stdout = stdout_only(output, stderr.clone());
        Ok(ExecPipe {
            stdout,
            stdin: input,
            done: PipeDone {
                exec_id: created.id,
                stderr,
            },
        })
    }

    pub async fn close_pipe(&self, done: &PipeDone, failure_code: &'static str) -> AppResult<()> {
        let details = self
            .client
            .inspect_exec(&done.exec_id)
            .await
            .map_err(docker_error)?;
        let exit_code = details.exit_code.unwrap_or(-1);
        if exit_code == 0 {
            return Ok(());
        }
        let stderr = done.stderr.lock().unwrap_or_else(PoisonError::into_inner);
        Err(exec_failure(failure_code, exit_code, &stderr))
    }
}
