use axum::Json;
use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use ferrodock_core::AppError;
use tokio_util::io::ReaderStream;

use super::actor;
use crate::dto::{BackupDto, BackupRequestedDto, MasterKeyDto, MasterKeyRequest};
use crate::error::{ApiResult, ErrorEnvelope};
use crate::extract::{Auth, ClientMeta};
use crate::state::AppState;

#[utoipa::path(
    get,
    path = "/api/v1/system/backups",
    tag = "system",
    description = "Lists encrypted panel backups, newest first; owner only",
    responses((status = 200, body = Vec<BackupDto>), (status = 403, body = ErrorEnvelope))
)]
pub async fn list_backups(
    State(state): State<AppState>,
    Auth(principal): Auth,
) -> ApiResult<Json<Vec<BackupDto>>> {
    let files = state.backups.list(&principal).await?;
    Ok(Json(files.into_iter().map(Into::into).collect()))
}

#[utoipa::path(
    post,
    path = "/api/v1/system/backups",
    tag = "system",
    description = "Queues a panel backup: database dump plus proxy routes, sealed with the master key",
    responses((status = 202, body = BackupRequestedDto), (status = 403, body = ErrorEnvelope))
)]
pub async fn create_backup(
    State(state): State<AppState>,
    Auth(principal): Auth,
    client: ClientMeta,
) -> ApiResult<(StatusCode, Json<BackupRequestedDto>)> {
    let job_id = state.backups.request(actor(&principal, &client)).await?;
    Ok((
        StatusCode::ACCEPTED,
        Json(BackupRequestedDto {
            job_id: job_id.as_uuid(),
        }),
    ))
}

#[utoipa::path(
    get,
    path = "/api/v1/system/backups/{name}",
    tag = "system",
    description = "Downloads one backup archive; restoring it also needs the master key",
    params(("name" = String, Path, description = "Backup file name")),
    responses((status = 200, content_type = "application/octet-stream"), (status = 404, body = ErrorEnvelope))
)]
pub async fn download_backup(
    State(state): State<AppState>,
    Auth(principal): Auth,
    client: ClientMeta,
    Path(name): Path<String>,
) -> ApiResult<Response> {
    let path = state
        .backups
        .download(actor(&principal, &client), &name)
        .await?;
    let file = tokio::fs::File::open(path)
        .await
        .map_err(AppError::internal)?;
    let disposition = format!("attachment; filename=\"{name}\"");
    Ok((
        [
            (header::CONTENT_TYPE, "application/octet-stream".to_owned()),
            (header::CONTENT_DISPOSITION, disposition),
            (header::CACHE_CONTROL, "no-store".to_owned()),
        ],
        Body::from_stream(ReaderStream::new(file)),
    )
        .into_response())
}

#[utoipa::path(
    delete,
    path = "/api/v1/system/backups/{name}",
    tag = "system",
    description = "Deletes one backup archive",
    params(("name" = String, Path, description = "Backup file name")),
    responses((status = 204), (status = 404, body = ErrorEnvelope))
)]
pub async fn delete_backup(
    State(state): State<AppState>,
    Auth(principal): Auth,
    client: ClientMeta,
    Path(name): Path<String>,
) -> ApiResult<StatusCode> {
    state
        .backups
        .delete(actor(&principal, &client), &name)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/api/v1/system/master-key",
    tag = "system",
    description = "Returns the master key after confirming the owner's password; store it apart from the backups",
    request_body = MasterKeyRequest,
    responses((status = 200, body = MasterKeyDto), (status = 403, body = ErrorEnvelope))
)]
pub async fn export_master_key(
    State(state): State<AppState>,
    Auth(principal): Auth,
    client: ClientMeta,
    Json(request): Json<MasterKeyRequest>,
) -> ApiResult<Response> {
    let key = state
        .backups
        .export_master_key(actor(&principal, &client), request.password.into_inner())
        .await?;
    let body = Json(MasterKeyDto {
        master_key: key.to_string(),
    });
    Ok(([(header::CACHE_CONTROL, "no-store")], body).into_response())
}
