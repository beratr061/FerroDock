use std::io::SeekFrom;
use std::path::PathBuf;
use std::sync::Arc;

use ferrodock_core::{AppError, AppResult, Id};
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};
use tokio::sync::Mutex;

use crate::realtime::LogSignal;

const MAX_LINE_LEN: usize = 4096;
pub const MAX_CHUNK_LEN: usize = 64 * 1024;

#[derive(Debug, Clone)]
pub struct DeployLogs {
    dir: PathBuf,
}

#[derive(Debug, Clone)]
pub struct DeployLog {
    file: Arc<Mutex<File>>,
    signal: Option<LogSignal>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogChunk {
    pub text: String,
    pub next_offset: u64,
}

fn io_error(error: std::io::Error) -> AppError {
    AppError::internal(error)
}

fn whole_lines(buffer: &[u8], full: bool) -> usize {
    match buffer.iter().rposition(|byte| *byte == b'\n') {
        Some(index) => index + 1,
        None if full => buffer.len(),
        None => 0,
    }
}

impl DeployLogs {
    pub fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    fn path(&self, id: Id) -> PathBuf {
        self.dir.join(format!("{id}.log"))
    }

    pub async fn open(&self, id: Id) -> AppResult<DeployLog> {
        tokio::fs::create_dir_all(&self.dir)
            .await
            .map_err(io_error)?;
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.path(id))
            .await
            .map_err(io_error)?;
        Ok(DeployLog {
            file: Arc::new(Mutex::new(file)),
            signal: None,
        })
    }

    pub async fn read(&self, id: Id, offset: u64) -> AppResult<LogChunk> {
        let mut file = match File::open(self.path(id)).await {
            Ok(file) => file,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(LogChunk {
                    text: String::new(),
                    next_offset: 0,
                });
            }
            Err(error) => return Err(io_error(error)),
        };
        file.seek(SeekFrom::Start(offset)).await.map_err(io_error)?;
        let mut buffer = Vec::with_capacity(MAX_CHUNK_LEN);
        (&mut file)
            .take(MAX_CHUNK_LEN as u64)
            .read_to_end(&mut buffer)
            .await
            .map_err(io_error)?;
        let usable = whole_lines(&buffer, buffer.len() == MAX_CHUNK_LEN);
        Ok(LogChunk {
            text: String::from_utf8_lossy(&buffer[..usable]).into_owned(),
            next_offset: offset + usable as u64,
        })
    }

    pub async fn remove(&self, id: Id) {
        let _ = tokio::fs::remove_file(self.path(id)).await;
    }
}

impl DeployLog {
    pub async fn line(&self, text: &str) {
        let mut cleaned: String = text
            .trim_end_matches(['\r', '\n'])
            .chars()
            .filter(|c| *c == '\t' || !c.is_control() || *c == '\u{1b}')
            .take(MAX_LINE_LEN)
            .collect();
        cleaned.push('\n');
        let written = {
            let mut file = self.file.lock().await;
            match file.write_all(cleaned.as_bytes()).await {
                Ok(()) => file.flush().await,
                Err(error) => Err(error),
            }
        };
        if let Err(error) = written {
            tracing::warn!(%error, "deploy log write failed");
        }
        if let Some(signal) = &self.signal {
            signal.appended(false).await;
        }
    }

    #[must_use]
    pub fn with_signal(mut self, signal: LogSignal) -> Self {
        self.signal = Some(signal);
        self
    }

    pub async fn step(&self, text: &str) {
        self.line(&format!("==> {text}")).await;
    }

    pub async fn flush(&self) {
        let _ = self.file.lock().await.flush().await;
        if let Some(signal) = &self.signal {
            signal.appended(true).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn reads_whole_lines_from_offsets() -> AppResult<()> {
        let dir = std::env::temp_dir().join(format!("fd-log-{}", Id::new()));
        let logs = DeployLogs::new(dir.clone());
        let id = Id::new();
        let log = logs.open(id).await?;
        log.step("Cloning").await;
        log.line("remote: counting objects\r\n").await;
        log.flush().await;
        let first = logs.read(id, 0).await?;
        assert_eq!(first.text, "==> Cloning\nremote: counting objects\n");
        let rest = logs.read(id, first.next_offset).await?;
        assert_eq!(rest.text, "");
        assert_eq!(logs.read(Id::new(), 0).await?.next_offset, 0);
        let _ = tokio::fs::remove_dir_all(dir).await;
        Ok(())
    }
}
