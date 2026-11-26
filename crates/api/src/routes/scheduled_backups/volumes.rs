use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use ferrodock_domain::scheduled_backups::BackupTarget;
use uuid::Uuid;

use super::super::actor;
use crate::dto::{BackupPolicyDto, BackupPolicyRequest, BackupRunDto, CreateBackupRequest};
use crate::error::{ApiResult, ErrorEnvelope};
use crate::extract::{Auth, ClientMeta};
use crate::state::AppState;

fn target((application_id, volume): (Uuid, String)) -> BackupTarget {
    BackupTarget::Volume {
        application_id: application_id.into(),
        volume,
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/applications/{id}/volumes/{volume}/backup-policy",
    tag = "backups",
    description = "Returns the backup schedule of an application volume, or null when it has none",
    params(("id" = Uuid, Path, description = "Application id"), ("volume" = String, Path, description = "Volume name as configured on the application")),
    responses((status = 200, body = Option<BackupPolicyDto>), (status = 404, body = ErrorEnvelope))
)]
pub async fn get_volume_backup_policy(
    State(state): State<AppState>,
    Auth(principal): Auth,
    Path(path): Path<(Uuid, String)>,
) -> ApiResult<Json<Option<BackupPolicyDto>>> {
    let policy = state
        .scheduled_backups
        .policy(&principal, &target(path))
        .await?;
    Ok(Json(policy.map(Into::into)))
}

#[utoipa::path(
    put,
    path = "/api/v1/applications/{id}/volumes/{volume}/backup-policy",
    tag = "backups",
    description = "Sets the backup schedule of an application volume; requires the developer role",
    params(("id" = Uuid, Path, description = "Application id"), ("volume" = String, Path, description = "Volume name")),
    request_body = BackupPolicyRequest,
    responses(
        (status = 200, body = BackupPolicyDto),
        (status = 403, body = ErrorEnvelope),
        (status = 404, body = ErrorEnvelope),
        (status = 422, body = ErrorEnvelope)
    )
)]
pub async fn set_volume_backup_policy(
    State(state): State<AppState>,
    Auth(principal): Auth,
    client: ClientMeta,
    Path(path): Path<(Uuid, String)>,
    Json(request): Json<BackupPolicyRequest>,
) -> ApiResult<Json<BackupPolicyDto>> {
    let policy = state
        .scheduled_backups
        .set_policy(actor(&principal, &client), target(path), request.into())
        .await?;
    Ok(Json(policy.into()))
}

#[utoipa::path(
    delete,
    path = "/api/v1/applications/{id}/volumes/{volume}/backup-policy",
    tag = "backups",
    description = "Stops scheduled backups of an application volume; existing backups are kept",
    params(("id" = Uuid, Path, description = "Application id"), ("volume" = String, Path, description = "Volume name")),
    responses((status = 204), (status = 403, body = ErrorEnvelope), (status = 404, body = ErrorEnvelope))
)]
pub async fn delete_volume_backup_policy(
    State(state): State<AppState>,
    Auth(principal): Auth,
    client: ClientMeta,
    Path(path): Path<(Uuid, String)>,
) -> ApiResult<StatusCode> {
    state
        .scheduled_backups
        .remove_policy(actor(&principal, &client), &target(path))
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/api/v1/applications/{id}/volumes/{volume}/backups",
    tag = "backups",
    description = "Lists the latest backup and restore runs of an application volume",
    params(("id" = Uuid, Path, description = "Application id"), ("volume" = String, Path, description = "Volume name")),
    responses((status = 200, body = Vec<BackupRunDto>), (status = 404, body = ErrorEnvelope))
)]
pub async fn list_volume_backups(
    State(state): State<AppState>,
    Auth(principal): Auth,
    Path(path): Path<(Uuid, String)>,
) -> ApiResult<Json<Vec<BackupRunDto>>> {
    let runs = state
        .scheduled_backups
        .runs(&principal, &target(path))
        .await?;
    Ok(Json(runs.into_iter().map(Into::into).collect()))
}

#[utoipa::path(
    post,
    path = "/api/v1/applications/{id}/volumes/{volume}/backups",
    tag = "backups",
    description = "Queues an encrypted archive of an application volume; requires the developer role. The volume is read while the application keeps running",
    params(("id" = Uuid, Path, description = "Application id"), ("volume" = String, Path, description = "Volume name")),
    request_body = CreateBackupRequest,
    responses(
        (status = 202, body = BackupRunDto),
        (status = 403, body = ErrorEnvelope),
        (status = 404, body = ErrorEnvelope)
    )
)]
pub async fn create_volume_backup(
    State(state): State<AppState>,
    Auth(principal): Auth,
    client: ClientMeta,
    Path(path): Path<(Uuid, String)>,
    Json(request): Json<CreateBackupRequest>,
) -> ApiResult<(StatusCode, Json<BackupRunDto>)> {
    let run = state
        .scheduled_backups
        .back_up_now(
            actor(&principal, &client),
            target(path),
            request.destination_id.into(),
        )
        .await?;
    Ok((StatusCode::ACCEPTED, Json(run.into())))
}
