use std::path::PathBuf;
use std::sync::Arc;

use ferrodock_core::{AppError, AppResult, Id};
use ferrodock_db::JobQueue;
use ferrodock_db::jobs::{LOCAL_QUEUE, NewJob, PANEL_BACKUP_JOB};
use ferrodock_infra::backup_store::{BackupFile, BackupStore};
use serde_json::json;
use zeroize::Zeroizing;

use crate::audit::{Actor, AuditLog, AuditRecord};
use crate::auth::{AuthService, Principal, Role};

#[derive(Clone)]
pub struct BackupService {
    store: BackupStore,
    jobs: Arc<dyn JobQueue>,
    auth: AuthService,
    audit: AuditLog,
    master_key_path: PathBuf,
}

impl std::fmt::Debug for BackupService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("BackupService")
    }
}

pub struct BackupServiceParts {
    pub store: BackupStore,
    pub jobs: Arc<dyn JobQueue>,
    pub auth: AuthService,
    pub audit: AuditLog,
    pub master_key_path: PathBuf,
}

impl std::fmt::Debug for BackupServiceParts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("BackupServiceParts")
    }
}

impl BackupService {
    pub fn new(parts: BackupServiceParts) -> Self {
        Self {
            store: parts.store,
            jobs: parts.jobs,
            auth: parts.auth,
            audit: parts.audit,
            master_key_path: parts.master_key_path,
        }
    }

    async fn audit(&self, actor: Actor<'_>, action: &'static str, name: Option<&str>) {
        let record = AuditRecord {
            action,
            resource_type: "backup",
            resource_id: None,
            metadata: json!({ "name": name }),
            ip_address: actor.ip_address,
        };
        self.audit.record(actor.principal, record).await;
    }

    pub async fn list(&self, principal: &Principal) -> AppResult<Vec<BackupFile>> {
        principal.require(Role::Owner)?;
        self.store.list().await
    }

    pub async fn request(&self, actor: Actor<'_>) -> AppResult<Id> {
        actor.principal.require(Role::Owner)?;
        let job = NewJob {
            organization_id: Some(actor.principal.organization_id),
            kind: PANEL_BACKUP_JOB.to_owned(),
            queue: LOCAL_QUEUE.to_owned(),
            group_key: Some(PANEL_BACKUP_JOB.to_owned()),
            payload: json!({}),
        };
        let record = self.jobs.enqueue(job).await?;
        self.audit(actor, "backup.requested", None).await;
        Ok(record.id)
    }

    pub async fn download(&self, actor: Actor<'_>, name: &str) -> AppResult<PathBuf> {
        actor.principal.require(Role::Owner)?;
        let path = self.store.path(name)?;
        if !tokio::fs::try_exists(&path).await.unwrap_or(false) {
            return Err(AppError::not_found("BACKUP_NOT_FOUND"));
        }
        self.audit(actor, "backup.downloaded", Some(name)).await;
        Ok(path)
    }

    pub async fn delete(&self, actor: Actor<'_>, name: &str) -> AppResult<()> {
        actor.principal.require(Role::Owner)?;
        self.store.delete(name).await?;
        self.audit(actor, "backup.deleted", Some(name)).await;
        Ok(())
    }

    pub async fn export_master_key(
        &self,
        actor: Actor<'_>,
        password: Zeroizing<String>,
    ) -> AppResult<Zeroizing<String>> {
        actor.principal.require(Role::Owner)?;
        self.auth
            .confirm_password(actor.principal, password, actor.ip_address)
            .await?;
        let key = tokio::fs::read_to_string(&self.master_key_path)
            .await
            .map_err(AppError::internal)?;
        self.audit(actor, "master_key.exported", None).await;
        Ok(Zeroizing::new(key.trim().to_owned()))
    }
}
