use std::path::{Path, PathBuf};

use ferrodock_core::{AppError, AppResult};
use time::OffsetDateTime;
use time::macros::format_description;
use tokio::io::AsyncWriteExt;

const PREFIX: &str = "ferrodock-";
const EXTENSION: &str = ".fdbk";

#[derive(Debug, Clone)]
pub struct BackupStore {
    dir: PathBuf,
}

#[derive(Debug, Clone)]
pub struct BackupFile {
    pub name: String,
    pub size: u64,
    pub created_at: OffsetDateTime,
}

fn valid_name(name: &str) -> bool {
    name.strip_prefix(PREFIX)
        .and_then(|rest| rest.strip_suffix(EXTENSION))
        .is_some_and(|stamp| {
            !stamp.is_empty() && stamp.chars().all(|c| c.is_ascii_digit() || c == '-')
        })
}

fn io(error: std::io::Error) -> AppError {
    AppError::internal(error)
}

async fn write_private(path: &Path, bytes: &[u8]) -> AppResult<()> {
    let mut options = tokio::fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    options.mode(0o600);
    let mut file = options.open(path).await.map_err(io)?;
    file.write_all(bytes).await.map_err(io)?;
    file.flush().await.map_err(io)
}

impl BackupStore {
    pub fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    pub fn path(&self, name: &str) -> AppResult<PathBuf> {
        if valid_name(name) {
            Ok(self.dir.join(name))
        } else {
            Err(AppError::not_found("BACKUP_NOT_FOUND"))
        }
    }

    pub async fn write(&self, now: OffsetDateTime, bytes: &[u8]) -> AppResult<BackupFile> {
        tokio::fs::create_dir_all(&self.dir).await.map_err(io)?;
        let stamp = now
            .format(format_description!(
                "[year][month][day]-[hour][minute][second]"
            ))
            .map_err(AppError::internal)?;
        let name = format!("{PREFIX}{stamp}{EXTENSION}");
        let staging = self.dir.join(format!(".{name}.tmp"));
        write_private(&staging, bytes).await?;
        tokio::fs::rename(&staging, self.dir.join(&name))
            .await
            .map_err(io)?;
        Ok(BackupFile {
            name,
            size: bytes.len() as u64,
            created_at: now,
        })
    }

    pub async fn list(&self) -> AppResult<Vec<BackupFile>> {
        let mut entries = match tokio::fs::read_dir(&self.dir).await {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(error) => return Err(io(error)),
        };
        let mut files = Vec::new();
        while let Some(entry) = entries.next_entry().await.map_err(io)? {
            let name = entry.file_name().to_string_lossy().into_owned();
            if !valid_name(&name) {
                continue;
            }
            let metadata = entry.metadata().await.map_err(io)?;
            let created_at = metadata.modified().map(OffsetDateTime::from).map_err(io)?;
            files.push(BackupFile {
                name,
                size: metadata.len(),
                created_at,
            });
        }
        files.sort_by(|a, b| {
            b.created_at
                .cmp(&a.created_at)
                .then_with(|| b.name.cmp(&a.name))
        });
        Ok(files)
    }

    pub async fn delete(&self, name: &str) -> AppResult<()> {
        match tokio::fs::remove_file(self.path(name)?).await {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                Err(AppError::not_found("BACKUP_NOT_FOUND"))
            }
            other => other.map_err(io),
        }
    }

    pub async fn prune(&self, keep: usize) -> AppResult<usize> {
        let stale: Vec<BackupFile> = self.list().await?.into_iter().skip(keep.max(1)).collect();
        for file in &stale {
            self.delete(&file.name).await?;
        }
        Ok(stale.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn writes_lists_and_prunes() -> AppResult<()> {
        let dir = std::env::temp_dir().join(format!("fd-backups-{}", ferrodock_core::Id::new()));
        let store = BackupStore::new(dir.clone());
        let base = OffsetDateTime::UNIX_EPOCH + time::Duration::days(20_000);
        for offset in 0..3 {
            store
                .write(base + time::Duration::seconds(offset), b"x")
                .await?;
        }
        assert_eq!(store.list().await?.len(), 3);
        assert_eq!(store.prune(2).await?, 1);
        assert!(store.path("../master.key").is_err());
        assert!(store.path("ferrodock-20260101-000000.fdbk").is_ok());
        let _ = tokio::fs::remove_dir_all(dir).await;
        Ok(())
    }
}
