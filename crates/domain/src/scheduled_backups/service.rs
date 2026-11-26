use std::path::PathBuf;
use std::sync::Arc;

use ferrodock_core::crypto::SecretBox;
use ferrodock_core::{AppError, AppResult, Id};
use ferrodock_db::backups::{DestinationRecord, S3Settings};
use ferrodock_db::{
    ApplicationRepo, BackupDestinationRepo, BackupPolicyRepo, BackupRunRepo, DatabaseRepo,
    JobQueue, RuntimeConfigRepo,
};
use serde_json::json;

use super::model::{Destination, NewDestinationInput};
use super::storage::open_destination;
use super::validate;
use crate::audit::{Actor, AuditLog};
use crate::auth::{Principal, Role};
use crate::text::clean_name;

pub struct ScheduledBackupParts {
    pub databases: Arc<dyn DatabaseRepo>,
    pub applications: Arc<dyn ApplicationRepo>,
    pub runtime_configs: Arc<dyn RuntimeConfigRepo>,
    pub destinations: Arc<dyn BackupDestinationRepo>,
    pub policies: Arc<dyn BackupPolicyRepo>,
    pub runs: Arc<dyn BackupRunRepo>,
    pub jobs: Arc<dyn JobQueue>,
    pub secrets: SecretBox,
    pub audit: AuditLog,
    pub local_root: PathBuf,
}

impl std::fmt::Debug for ScheduledBackupParts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ScheduledBackupParts")
    }
}

#[derive(Clone, Debug)]
pub struct ScheduledBackupService {
    pub(super) parts: Arc<ScheduledBackupParts>,
}

fn name_taken(error: AppError) -> AppError {
    if error.code() == "ALREADY_EXISTS" {
        AppError::conflict("BACKUP_DESTINATION_EXISTS")
    } else {
        error
    }
}

impl ScheduledBackupService {
    pub fn new(parts: ScheduledBackupParts) -> Self {
        Self {
            parts: Arc::new(parts),
        }
    }

    pub async fn destinations(&self, principal: &Principal) -> AppResult<Vec<Destination>> {
        let records = self
            .parts
            .destinations
            .destinations(principal.organization_id)
            .await?;
        Ok(records.into_iter().map(Into::into).collect())
    }

    pub async fn create_destination(
        &self,
        actor: Actor<'_>,
        input: NewDestinationInput,
    ) -> AppResult<Destination> {
        actor.principal.require(Role::Admin)?;
        let s3 = input.s3.as_ref().map(validate::s3).transpose()?;
        let s3 = match s3 {
            Some(s3) => Some(S3Settings {
                secret_access_key_enc: self
                    .parts
                    .secrets
                    .encrypt(s3.secret_access_key.as_bytes())?,
                endpoint: s3.endpoint,
                region: s3.region,
                bucket: s3.bucket,
                access_key_id: s3.access_key_id,
                path_style: s3.path_style,
            }),
            None => None,
        };
        let record = DestinationRecord {
            id: Id::new(),
            organization_id: actor.principal.organization_id,
            name: clean_name(&input.name, "BACKUP_DESTINATION_NAME_INVALID")?,
            path_prefix: validate::path_prefix(input.path_prefix.as_deref())?,
            s3,
            created_at: time::OffsetDateTime::now_utc(),
        };
        self.open(&record, None).await?.probe().await?;
        self.parts
            .destinations
            .create_destination(record.clone())
            .await
            .map_err(name_taken)?;
        let metadata = json!({ "name": record.name, "kind": if record.s3.is_some() { "s3" } else { "local" } });
        self.parts
            .audit
            .change(actor, "backup_destination.created", record.id, metadata)
            .await;
        Ok(record.into())
    }

    pub async fn test_destination(&self, principal: &Principal, id: Id) -> AppResult<()> {
        principal.require(Role::Admin)?;
        let record = self.destination(principal, id).await?;
        self.open(&record, None).await?.probe().await
    }

    pub async fn delete_destination(&self, actor: Actor<'_>, id: Id) -> AppResult<()> {
        actor.principal.require(Role::Admin)?;
        let record = self.destination(actor.principal, id).await?;
        if self.parts.destinations.destination_in_use(id).await? {
            return Err(AppError::conflict("BACKUP_DESTINATION_IN_USE"));
        }
        self.parts
            .destinations
            .delete_destination(actor.principal.organization_id, id)
            .await?;
        self.parts
            .audit
            .change(
                actor,
                "backup_destination.deleted",
                id,
                json!({ "name": record.name }),
            )
            .await;
        Ok(())
    }

    pub(super) async fn destination(
        &self,
        principal: &Principal,
        id: Id,
    ) -> AppResult<DestinationRecord> {
        self.parts
            .destinations
            .destination(principal.organization_id, id)
            .await?
            .ok_or_else(|| AppError::not_found("BACKUP_DESTINATION_NOT_FOUND"))
    }

    pub(super) async fn open(
        &self,
        record: &DestinationRecord,
        scope: Option<&str>,
    ) -> AppResult<ferrodock_infra::object_storage::ObjectStorage> {
        open_destination(record, &self.parts.secrets, &self.parts.local_root, scope).await
    }
}
