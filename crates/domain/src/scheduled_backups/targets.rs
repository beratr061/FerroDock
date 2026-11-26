use ferrodock_core::{AppError, AppResult, Id};
use ferrodock_db::backups::BackupTarget;
use serde_json::{Value, json};

use super::service::ScheduledBackupService;
use crate::auth::Principal;

pub fn storage_scope(target: &BackupTarget) -> String {
    match target {
        BackupTarget::Database(id) => id.to_string(),
        BackupTarget::Volume {
            application_id,
            volume,
        } => format!("{application_id}/volumes/{volume}"),
    }
}

pub(super) fn resource_id(target: &BackupTarget) -> Id {
    match target {
        BackupTarget::Database(id) => *id,
        BackupTarget::Volume { application_id, .. } => *application_id,
    }
}

pub(super) fn describe(target: &BackupTarget) -> Value {
    match target {
        BackupTarget::Database(id) => json!({ "database_id": id }),
        BackupTarget::Volume {
            application_id,
            volume,
        } => json!({ "application_id": application_id, "volume": volume }),
    }
}

impl ScheduledBackupService {
    pub(super) async fn authorize(
        &self,
        principal: &Principal,
        target: &BackupTarget,
    ) -> AppResult<()> {
        match target {
            BackupTarget::Database(id) => self
                .parts
                .databases
                .find(principal.organization_id, *id)
                .await?
                .map(|_| ())
                .ok_or_else(|| AppError::not_found("DATABASE_NOT_FOUND")),
            BackupTarget::Volume {
                application_id,
                volume,
            } => {
                self.authorize_volume(principal, *application_id, volume)
                    .await
            }
        }
    }

    async fn authorize_volume(
        &self,
        principal: &Principal,
        application_id: Id,
        volume: &str,
    ) -> AppResult<()> {
        self.parts
            .applications
            .find(principal.organization_id, application_id)
            .await?
            .ok_or_else(|| AppError::not_found("APPLICATION_NOT_FOUND"))?;
        let config = self
            .parts
            .runtime_configs
            .runtime_config(application_id)
            .await?;
        let mounted = config
            .mounts
            .iter()
            .any(|mount| mount.kind == "volume" && mount.source == volume);
        if mounted {
            Ok(())
        } else {
            Err(AppError::not_found("BACKUP_VOLUME_NOT_FOUND"))
        }
    }
}
