use std::path::Path;

use ferrodock_core::crypto::SecretBox;
use ferrodock_core::{AppError, AppResult, Id};
use ferrodock_db::backups::DestinationRecord;
use ferrodock_infra::object_storage::{ObjectStorage, S3Target, StorageTarget};
use zeroize::Zeroizing;

fn target(
    record: &DestinationRecord,
    secrets: &SecretBox,
    local_root: &Path,
) -> AppResult<StorageTarget> {
    let Some(s3) = &record.s3 else {
        return Ok(StorageTarget::Local(local_root.join(record.id.to_string())));
    };
    let secret = secrets.decrypt(&s3.secret_access_key_enc)?;
    let secret =
        String::from_utf8(secret.to_vec()).map_err(|_| AppError::validation("SECRET_CORRUPTED"))?;
    Ok(StorageTarget::S3(S3Target {
        endpoint: s3.endpoint.clone(),
        region: s3.region.clone(),
        bucket: s3.bucket.clone(),
        access_key_id: s3.access_key_id.clone(),
        secret_access_key: Zeroizing::new(secret),
        path_style: s3.path_style,
    }))
}

pub async fn open_destination(
    record: &DestinationRecord,
    secrets: &SecretBox,
    local_root: &Path,
    database_id: Option<Id>,
) -> AppResult<ObjectStorage> {
    let prefix = match database_id {
        Some(id) => format!("{}/{id}", record.path_prefix),
        None => record.path_prefix.clone(),
    };
    ObjectStorage::open(target(record, secrets, local_root)?, &prefix).await
}
