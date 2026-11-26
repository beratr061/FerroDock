use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use uuid::Uuid;

use super::super::actor;
use crate::dto::{BackupPolicyDto, BackupPolicyRequest, BackupRunDto, CreateBackupRequest};
use crate::error::{ApiResult, ErrorEnvelope};
use crate::extract::{Auth, ClientMeta};
use crate::state::AppState;

#[utoipa::path(
    get,
    path = "/api/v1/databases/{id}/backup-policy",
    tag = "backups",
    description = "Returns the backup schedule of a database, or null when it has none",
    params(("id" = Uuid, Path, description = "Database id")),
    responses((status = 200, body = Option<BackupPolicyDto>), (status = 404, body = ErrorEnvelope))
)]
pub async fn get_backup_policy(
    State(state): State<AppState>,
    Auth(principal): Auth,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<Option<BackupPolicyDto>>> {
    let policy = state.database_backups.policy(&principal, id.into()).await?;
    Ok(Json(policy.map(Into::into)))
}

#[utoipa::path(
    put,
    path = "/api/v1/databases/{id}/backup-policy",
    tag = "backups",
    description = "Sets the backup schedule (five-field cron in UTC), destination and how many backups to keep; requires the developer role",
    params(("id" = Uuid, Path, description = "Database id")),
    request_body = BackupPolicyRequest,
    responses(
        (status = 200, body = BackupPolicyDto),
        (status = 403, body = ErrorEnvelope),
        (status = 404, body = ErrorEnvelope),
        (status = 422, body = ErrorEnvelope)
    )
)]
pub async fn set_backup_policy(
    State(state): State<AppState>,
    Auth(principal): Auth,
    client: ClientMeta,
    Path(id): Path<Uuid>,
    Json(request): Json<BackupPolicyRequest>,
) -> ApiResult<Json<BackupPolicyDto>> {
    let policy = state
        .database_backups
        .set_policy(actor(&principal, &client), id.into(), request.into())
        .await?;
    Ok(Json(policy.into()))
}

#[utoipa::path(
    delete,
    path = "/api/v1/databases/{id}/backup-policy",
    tag = "backups",
    description = "Stops scheduled backups; existing backups are kept",
    params(("id" = Uuid, Path, description = "Database id")),
    responses((status = 204), (status = 403, body = ErrorEnvelope), (status = 404, body = ErrorEnvelope))
)]
pub async fn delete_backup_policy(
    State(state): State<AppState>,
    Auth(principal): Auth,
    client: ClientMeta,
    Path(id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    state
        .database_backups
        .remove_policy(actor(&principal, &client), id.into())
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/api/v1/databases/{id}/backups",
    tag = "backups",
    description = "Lists the latest backup and restore runs of a database",
    params(("id" = Uuid, Path, description = "Database id")),
    responses((status = 200, body = Vec<BackupRunDto>), (status = 404, body = ErrorEnvelope))
)]
pub async fn list_database_backups(
    State(state): State<AppState>,
    Auth(principal): Auth,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<Vec<BackupRunDto>>> {
    let runs = state.database_backups.runs(&principal, id.into()).await?;
    Ok(Json(runs.into_iter().map(Into::into).collect()))
}

#[utoipa::path(
    post,
    path = "/api/v1/databases/{id}/backups",
    tag = "backups",
    description = "Queues an encrypted backup of the database to a destination; requires the developer role",
    params(("id" = Uuid, Path, description = "Database id")),
    request_body = CreateBackupRequest,
    responses(
        (status = 202, body = BackupRunDto),
        (status = 403, body = ErrorEnvelope),
        (status = 404, body = ErrorEnvelope)
    )
)]
pub async fn create_database_backup(
    State(state): State<AppState>,
    Auth(principal): Auth,
    client: ClientMeta,
    Path(id): Path<Uuid>,
    Json(request): Json<CreateBackupRequest>,
) -> ApiResult<(StatusCode, Json<BackupRunDto>)> {
    let run = state
        .database_backups
        .back_up_now(
            actor(&principal, &client),
            id.into(),
            request.destination_id.into(),
        )
        .await?;
    Ok((StatusCode::ACCEPTED, Json(run.into())))
}

#[utoipa::path(
    post,
    path = "/api/v1/database-backups/{id}/restore",
    tag = "backups",
    description = "Queues a restore that replaces the database contents with this backup; requires the admin role",
    params(("id" = Uuid, Path, description = "Backup run id")),
    responses(
        (status = 202, body = BackupRunDto),
        (status = 403, body = ErrorEnvelope),
        (status = 404, body = ErrorEnvelope)
    )
)]
pub async fn restore_database_backup(
    State(state): State<AppState>,
    Auth(principal): Auth,
    client: ClientMeta,
    Path(id): Path<Uuid>,
) -> ApiResult<(StatusCode, Json<BackupRunDto>)> {
    let run = state
        .database_backups
        .restore(actor(&principal, &client), id.into())
        .await?;
    Ok((StatusCode::ACCEPTED, Json(run.into())))
}

#[utoipa::path(
    delete,
    path = "/api/v1/database-backups/{id}",
    tag = "backups",
    description = "Deletes a stored backup file and its record; requires the admin role",
    params(("id" = Uuid, Path, description = "Backup run id")),
    responses((status = 204), (status = 403, body = ErrorEnvelope), (status = 404, body = ErrorEnvelope))
)]
pub async fn delete_database_backup(
    State(state): State<AppState>,
    Auth(principal): Auth,
    client: ClientMeta,
    Path(id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    state
        .database_backups
        .delete_backup(actor(&principal, &client), id.into())
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
