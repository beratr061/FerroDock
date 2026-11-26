use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use ferrodock_core::config::{AppConfig, ProxyMode};
use ferrodock_core::crypto::SecretBox;
use ferrodock_core::{AppError, AppResult};
use ferrodock_db::JobQueue;
use ferrodock_db::jobs::{LOCAL_QUEUE, NewJob, PANEL_BACKUP_JOB};
use ferrodock_db::pg::PgStore;
use ferrodock_domain::audit::AuditLog;
use ferrodock_domain::auth::AuthService;
use ferrodock_domain::backups::{BackupService, BackupServiceParts};
use ferrodock_engine::backup::{BackupContext, DatabaseLocator};
use ferrodock_infra::backup_store::BackupStore;
use ferrodock_infra::docker::DockerEngine;
use time::OffsetDateTime;
use tokio_util::sync::CancellationToken;

const CHECK_INTERVAL: Duration = Duration::from_mins(30);

fn database_locator(config: &AppConfig) -> AppResult<DatabaseLocator> {
    let invalid = || AppError::validation("DATABASE_URL_INVALID");
    let url = config.database_url.expose();
    let rest = url
        .split_once("://")
        .map(|(_, rest)| rest)
        .ok_or_else(invalid)?;
    let (authority, path) = rest.split_once('/').ok_or_else(invalid)?;
    let userinfo = authority
        .rsplit_once('@')
        .map(|(userinfo, _)| userinfo)
        .ok_or_else(invalid)?;
    let user = userinfo
        .split(':')
        .next()
        .filter(|user| !user.is_empty())
        .ok_or_else(invalid)?;
    let database = path
        .split('?')
        .next()
        .filter(|name| !name.is_empty())
        .ok_or_else(invalid)?;
    Ok(DatabaseLocator {
        container: config.database_container.clone(),
        user: user.to_owned(),
        database: database.to_owned(),
    })
}

pub fn store(config: &AppConfig) -> BackupStore {
    BackupStore::new(config.data_dir.join("backups"))
}

pub fn context(
    config: &AppConfig,
    docker: DockerEngine,
    secrets: SecretBox,
) -> AppResult<BackupContext> {
    Ok(BackupContext {
        docker,
        secrets,
        store: store(config),
        database: database_locator(config)?,
        proxy_dir: (config.proxy == ProxyMode::Traefik).then(|| config.proxy_dir.clone()),
        data_dir: config.data_dir.clone(),
        retention: usize::try_from(config.backup_retention).unwrap_or(7),
    })
}

async fn due(store: &BackupStore, interval: Duration) -> bool {
    let newest = store
        .list()
        .await
        .ok()
        .and_then(|files| files.into_iter().next());
    newest.is_none_or(|file| {
        let age = OffsetDateTime::now_utc() - file.created_at;
        age.whole_seconds() >= i64::try_from(interval.as_secs()).unwrap_or(i64::MAX)
    })
}

async fn enqueue(jobs: &dyn JobQueue) {
    let job = NewJob {
        organization_id: None,
        kind: PANEL_BACKUP_JOB.to_owned(),
        queue: LOCAL_QUEUE.to_owned(),
        group_key: Some(PANEL_BACKUP_JOB.to_owned()),
        payload: serde_json::json!({ "scheduled": true }),
    };
    if let Err(error) = jobs.enqueue(job).await {
        tracing::warn!(code = error.code(), "scheduled backup not queued");
    }
}

pub fn schedule(config: &AppConfig, jobs: Arc<dyn JobQueue>, shutdown: CancellationToken) {
    if config.backup_interval_hours == 0 {
        return;
    }
    let interval = Duration::from_hours(u64::from(config.backup_interval_hours));
    let store = store(config);
    tokio::spawn(async move {
        loop {
            if due(&store, interval).await {
                enqueue(jobs.as_ref()).await;
            }
            tokio::select! {
                () = shutdown.cancelled() => return,
                () = tokio::time::sleep(CHECK_INTERVAL) => {}
            }
        }
    });
}

fn say(line: &str) -> AppResult<()> {
    use std::io::Write;
    writeln!(std::io::stdout().lock(), "{line}").map_err(AppError::internal)
}

async fn adopt_master_key(source: &Path, target: &Path) -> AppResult<()> {
    if source == target {
        return Ok(());
    }
    if tokio::fs::try_exists(target).await.unwrap_or(false) {
        let stamp = OffsetDateTime::now_utc().unix_timestamp();
        let previous = target.with_extension(format!("key.replaced-{stamp}"));
        tokio::fs::rename(target, previous)
            .await
            .map_err(AppError::internal)?;
    }
    tokio::fs::copy(source, target)
        .await
        .map_err(AppError::internal)?;
    Ok(())
}

pub async fn create_now(config: &AppConfig) -> AppResult<()> {
    let secrets = SecretBox::load_or_create(&config.master_key_path)?;
    let file = context(config, DockerEngine::connect_local()?, secrets)?
        .create()
        .await?;
    say(&format!("{} ({} bytes)", file.name, file.size))
}

pub async fn restore(config: &AppConfig, archive: &Path, master_key: &Path) -> AppResult<()> {
    let secrets = SecretBox::load(master_key)?;
    let bytes = tokio::fs::read(archive).await.map_err(AppError::internal)?;
    let summary = context(config, DockerEngine::connect_local()?, secrets)?
        .restore(&bytes)
        .await?;
    adopt_master_key(master_key, &config.master_key_path).await?;
    say(&format!(
        "restored backup from FerroDock {} created {} ({} extra files)",
        summary.version, summary.created_at, summary.files
    ))
}

pub fn service(
    config: &AppConfig,
    pg: &PgStore,
    auth: AuthService,
    audit: AuditLog,
) -> BackupService {
    BackupService::new(BackupServiceParts {
        store: store(config),
        jobs: Arc::new(pg.clone()),
        auth,
        audit,
        master_key_path: config.master_key_path.clone(),
    })
}
