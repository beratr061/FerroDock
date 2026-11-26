use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use uuid::Uuid;

use super::super::actor;
use crate::dto::{BackupDestinationDto, CreateBackupDestinationRequest};
use crate::error::{ApiResult, ErrorEnvelope};
use crate::extract::{Auth, ClientMeta};
use crate::state::AppState;

#[utoipa::path(
    get,
    path = "/api/v1/backup-destinations",
    tag = "backups",
    description = "Lists where database backups can be stored; secrets are never returned",
    responses((status = 200, body = Vec<BackupDestinationDto>), (status = 401, body = ErrorEnvelope))
)]
pub async fn list_backup_destinations(
    State(state): State<AppState>,
    Auth(principal): Auth,
) -> ApiResult<Json<Vec<BackupDestinationDto>>> {
    let destinations = state.scheduled_backups.destinations(&principal).await?;
    Ok(Json(destinations.into_iter().map(Into::into).collect()))
}

#[utoipa::path(
    post,
    path = "/api/v1/backup-destinations",
    tag = "backups",
    description = "Adds a local or S3-compatible destination after writing, reading and deleting a probe object; requires the admin role",
    request_body = CreateBackupDestinationRequest,
    responses(
        (status = 201, body = BackupDestinationDto),
        (status = 403, body = ErrorEnvelope),
        (status = 409, body = ErrorEnvelope),
        (status = 422, body = ErrorEnvelope)
    )
)]
pub async fn create_backup_destination(
    State(state): State<AppState>,
    Auth(principal): Auth,
    client: ClientMeta,
    Json(request): Json<CreateBackupDestinationRequest>,
) -> ApiResult<(StatusCode, Json<BackupDestinationDto>)> {
    let destination = state
        .scheduled_backups
        .create_destination(actor(&principal, &client), request.into())
        .await?;
    Ok((StatusCode::CREATED, Json(destination.into())))
}

#[utoipa::path(
    post,
    path = "/api/v1/backup-destinations/{id}/test",
    tag = "backups",
    description = "Checks that the destination still accepts writes; requires the admin role",
    params(("id" = Uuid, Path, description = "Destination id")),
    responses(
        (status = 204),
        (status = 403, body = ErrorEnvelope),
        (status = 404, body = ErrorEnvelope),
        (status = 422, body = ErrorEnvelope)
    )
)]
pub async fn test_backup_destination(
    State(state): State<AppState>,
    Auth(principal): Auth,
    Path(id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    state
        .scheduled_backups
        .test_destination(&principal, id.into())
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    delete,
    path = "/api/v1/backup-destinations/{id}",
    tag = "backups",
    description = "Removes a destination that no backup schedule uses; stored files are left in place. Requires the admin role",
    params(("id" = Uuid, Path, description = "Destination id")),
    responses(
        (status = 204),
        (status = 403, body = ErrorEnvelope),
        (status = 404, body = ErrorEnvelope),
        (status = 409, body = ErrorEnvelope)
    )
)]
pub async fn delete_backup_destination(
    State(state): State<AppState>,
    Auth(principal): Auth,
    client: ClientMeta,
    Path(id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    state
        .scheduled_backups
        .delete_destination(actor(&principal, &client), id.into())
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
