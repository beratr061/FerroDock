use ferrodock_core::crypto::SecretBox;
use ferrodock_core::sealed_stream::{CHUNK_SIZE, StreamOpener, StreamSealer};
use ferrodock_core::{AppError, AppResult};
use ferrodock_infra::docker::{ExecPipe, PipeOutput};
use ferrodock_infra::object_storage::{ObjectStorage, ObjectWriter};
use futures_util::StreamExt;
use tokio::io::AsyncWriteExt;

use super::work::Work;

async fn seal_into(
    secrets: &SecretBox,
    mut stdout: PipeOutput,
    writer: &mut ObjectWriter,
) -> AppResult<u64> {
    let mut sealer = StreamSealer::new(secrets);
    writer.write(sealer.header()).await?;
    let mut buffer: Vec<u8> = Vec::with_capacity(CHUNK_SIZE * 2);
    let mut total = 0u64;
    while let Some(piece) = stdout.next().await {
        let piece = piece?;
        total += piece.len() as u64;
        buffer.extend_from_slice(&piece);
        while buffer.len() > CHUNK_SIZE {
            let chunk = sealer.seal(&buffer[..CHUNK_SIZE], false)?;
            buffer.drain(..CHUNK_SIZE);
            writer.write(&chunk).await?;
        }
    }
    writer.write(&sealer.seal(&buffer, true)?).await?;
    Ok(total)
}

async fn verify(secrets: &SecretBox, storage: &ObjectStorage, key: &str) -> AppResult<()> {
    let mut opener = StreamOpener::new(secrets);
    let mut reader = storage.reader(key).await?;
    while let Some(piece) = reader.next().await {
        opener.push(&piece?)?;
    }
    opener.finish()
}

async fn feed(
    secrets: &SecretBox,
    storage: &ObjectStorage,
    key: &str,
    mut stdin: ferrodock_infra::docker::ShellInput,
) -> AppResult<()> {
    let mut opener = StreamOpener::new(secrets);
    let mut reader = storage.reader(key).await?;
    let broken =
        |error: std::io::Error| AppError::validation("BACKUP_RESTORE_FAILED").with_source(error);
    while let Some(piece) = reader.next().await {
        for chunk in opener.push(&piece?)? {
            stdin.write_all(&chunk).await.map_err(broken)?;
        }
    }
    opener.finish()?;
    stdin.shutdown().await.map_err(broken)
}

async fn drain(mut stdout: PipeOutput) -> AppResult<()> {
    while let Some(piece) = stdout.next().await {
        piece?;
    }
    Ok(())
}

impl Work<'_> {
    pub(super) async fn upload(&self, pipe: ExecPipe, key: &str) -> AppResult<i64> {
        let ExecPipe {
            stdout,
            stdin,
            done,
        } = pipe;
        drop(stdin);
        let mut writer = self.storage.writer(key).await?;
        let sealed = seal_into(&self.ctx.secrets, stdout, &mut writer).await;
        let exited = self
            .ctx
            .docker
            .close_pipe(&done, "BACKUP_DUMP_FAILED")
            .await;
        match (sealed, exited) {
            (Ok(plain), Ok(())) if plain > 0 => {
                let size = writer.finish().await?;
                Ok(i64::try_from(size).unwrap_or(i64::MAX))
            }
            (Ok(_), Ok(())) => {
                writer.abort().await;
                Err(AppError::validation("BACKUP_DUMP_FAILED").with_param("exit_code", 0))
            }
            (Err(error), _) | (_, Err(error)) => {
                writer.abort().await;
                Err(error)
            }
        }
    }

    pub(super) async fn verify(&self, key: &str) -> AppResult<()> {
        verify(&self.ctx.secrets, &self.storage, key).await
    }

    pub(super) async fn download_into(&self, key: &str, pipe: ExecPipe) -> AppResult<()> {
        let ExecPipe {
            stdout,
            stdin,
            done,
        } = pipe;
        let fed = feed(&self.ctx.secrets, &self.storage, key, stdin);
        let (fed, drained) = tokio::join!(fed, drain(stdout));
        let exited = self
            .ctx
            .docker
            .close_pipe(&done, "BACKUP_RESTORE_FAILED")
            .await;
        exited.and(fed).and(drained)
    }
}

#[cfg(test)]
mod tests {
    use ferrodock_core::Id;
    use ferrodock_infra::object_storage::StorageTarget;
    use tokio::io::AsyncReadExt;

    use super::*;

    fn pattern(length: usize) -> Vec<u8> {
        (0..length)
            .map(|index| u8::try_from(index % 251).unwrap_or_default())
            .collect()
    }

    #[tokio::test]
    async fn seals_streams_and_restores_large_dumps_in_chunks() -> AppResult<()> {
        let secrets = SecretBox::from_key(&[7u8; 32])?;
        let dir = std::env::temp_dir().join(format!("fd-pipeline-{}", Id::new()));
        let storage = ObjectStorage::open(StorageTarget::Local(dir.clone()), "db").await?;
        let data = pattern(3 * CHUNK_SIZE + 12_345);
        let pieces: Vec<AppResult<Vec<u8>>> = data
            .chunks(32 * 1024)
            .map(|piece| Ok(piece.to_vec()))
            .collect();
        let stdout: PipeOutput = Box::pin(futures_util::stream::iter(pieces));
        let mut writer = storage.writer("dump.fdenc").await?;
        assert_eq!(
            seal_into(&secrets, stdout, &mut writer).await?,
            data.len() as u64
        );
        writer.finish().await?;
        verify(&secrets, &storage, "dump.fdenc").await?;
        let (input, mut output) = tokio::io::duplex(64 * 1024);
        let reading = tokio::spawn(async move {
            let mut restored = Vec::new();
            output.read_to_end(&mut restored).await.map(|_| restored)
        });
        feed(&secrets, &storage, "dump.fdenc", Box::pin(input)).await?;
        let restored = reading
            .await
            .map_err(AppError::internal)?
            .map_err(AppError::internal)?;
        assert_eq!(restored, data);
        let other = SecretBox::from_key(&[8u8; 32])?;
        assert!(verify(&other, &storage, "dump.fdenc").await.is_err());
        let _ = tokio::fs::remove_dir_all(dir).await;
        Ok(())
    }
}
