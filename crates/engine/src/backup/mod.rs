mod archive;
mod files;

use std::path::PathBuf;

use async_trait::async_trait;
use ferrodock_core::archive::Entry;
use ferrodock_core::crypto::SecretBox;
use ferrodock_core::{AppError, AppResult};
use ferrodock_db::jobs::{JobRecord, PANEL_BACKUP_JOB};
use ferrodock_infra::backup_store::{BackupFile, BackupStore};
use ferrodock_infra::docker::{DatabaseTarget, DockerEngine};
use time::OffsetDateTime;
use tokio_util::sync::CancellationToken;

use self::archive::{DATABASE, Manifest};
use crate::jobs::JobHandler;

const CONTROL_PLANE_PROJECT: &str = "ferrodock";
const DATABASE_SERVICE: &str = "postgres";

#[derive(Debug, Clone)]
pub struct DatabaseLocator {
    pub container: Option<String>,
    pub user: String,
    pub database: String,
}

#[derive(Debug, Clone)]
pub struct BackupContext {
    pub docker: DockerEngine,
    pub secrets: SecretBox,
    pub store: BackupStore,
    pub database: DatabaseLocator,
    pub proxy_dir: Option<PathBuf>,
    pub data_dir: PathBuf,
    pub retention: usize,
}

#[derive(Debug, Clone)]
pub struct RestoreSummary {
    pub version: String,
    pub created_at: OffsetDateTime,
    pub files: usize,
}

impl BackupContext {
    async fn target(&self) -> AppResult<DatabaseTarget> {
        let container = match &self.database.container {
            Some(container) => Some(container.clone()),
            None => {
                self.docker
                    .compose_service_container(CONTROL_PLANE_PROJECT, DATABASE_SERVICE)
                    .await?
            }
        };
        Ok(DatabaseTarget {
            container: container
                .ok_or_else(|| AppError::validation("BACKUP_DATABASE_NOT_FOUND"))?,
            user: self.database.user.clone(),
            database: self.database.database.clone(),
        })
    }

    pub async fn create(&self) -> AppResult<BackupFile> {
        let dump = self.docker.pg_dump(&self.target().await?).await?;
        let mut entries = vec![Entry {
            name: DATABASE.to_owned(),
            data: dump,
        }];
        entries.extend(files::collect(self.proxy_dir.as_deref(), &self.data_dir).await?);
        let now = OffsetDateTime::now_utc();
        let manifest = Manifest {
            format: 1,
            version: env!("CARGO_PKG_VERSION").to_owned(),
            created_at: now,
        };
        let sealed = archive::seal(&self.secrets, &manifest, entries)?;
        let file = self.store.write(now, &sealed).await?;
        let removed = self.store.prune(self.retention).await?;
        tracing::info!(name = %file.name, size = file.size, removed, "panel backup written");
        Ok(file)
    }

    pub async fn restore(&self, bytes: &[u8]) -> AppResult<RestoreSummary> {
        let (manifest, entries) = archive::open(&self.secrets, bytes)?;
        archive::ensure_compatible(&manifest, env!("CARGO_PKG_VERSION"))?;
        let (database, others): (Vec<Entry>, Vec<Entry>) = entries
            .into_iter()
            .partition(|entry| entry.name == DATABASE);
        let dump = database
            .into_iter()
            .next()
            .ok_or_else(|| AppError::validation("BACKUP_ARCHIVE_INVALID"))?;
        self.docker
            .pg_restore(&self.target().await?, dump.data)
            .await?;
        let files = files::restore(&others, self.proxy_dir.as_deref(), &self.data_dir).await?;
        Ok(RestoreSummary {
            version: manifest.version,
            created_at: manifest.created_at,
            files,
        })
    }
}

#[derive(Debug, Clone)]
pub struct BackupHandler {
    context: BackupContext,
}

impl BackupHandler {
    pub fn new(context: BackupContext) -> Self {
        Self { context }
    }
}

#[async_trait]
impl JobHandler for BackupHandler {
    fn kind(&self) -> &'static str {
        PANEL_BACKUP_JOB
    }

    async fn run(&self, _job: &JobRecord, _cancel: CancellationToken) -> AppResult<()> {
        self.context.create().await.map(|_| ())
    }
}
