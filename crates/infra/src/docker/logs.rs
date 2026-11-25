use std::pin::Pin;

use bollard::container::LogOutput;
use bollard::query_parameters::LogsOptionsBuilder;
use ferrodock_core::AppResult;
use futures_util::{Stream, StreamExt};
use serde::Serialize;

use super::{DockerEngine, docker_error, validate_reference};

const MAX_TAIL_LINES: u32 = 5000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LogSource {
    Stdout,
    Stderr,
}

#[derive(Debug, Clone, Serialize)]
pub struct LogLine {
    pub source: LogSource,
    pub text: String,
}

#[derive(Debug, Clone, Copy)]
pub struct LogQuery {
    pub tail: u32,
    pub follow: bool,
}

pub type LogStream = Pin<Box<dyn Stream<Item = AppResult<LogLine>> + Send>>;

fn to_line(output: LogOutput) -> Option<LogLine> {
    let (source, message) = match output {
        LogOutput::StdOut { message } | LogOutput::Console { message } => {
            (LogSource::Stdout, message)
        }
        LogOutput::StdErr { message } => (LogSource::Stderr, message),
        LogOutput::StdIn { .. } => return None,
    };
    Some(LogLine {
        source,
        text: String::from_utf8_lossy(&message).into_owned(),
    })
}

impl DockerEngine {
    pub fn container_logs(&self, reference: &str, query: LogQuery) -> AppResult<LogStream> {
        let reference = validate_reference(reference)?.to_owned();
        let tail = query.tail.min(MAX_TAIL_LINES).to_string();
        let options = LogsOptionsBuilder::new()
            .stdout(true)
            .stderr(true)
            .follow(query.follow)
            .tail(&tail)
            .build();
        let stream = self
            .client
            .logs(&reference, Some(options))
            .filter_map(|item| async move {
                match item {
                    Ok(output) => to_line(output).map(Ok),
                    Err(error) => Some(Err(docker_error(error))),
                }
            });
        Ok(Box::pin(stream))
    }
}
