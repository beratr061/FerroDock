use axum::Json;
use axum::extract::{Path, Query, State, WebSocketUpgrade};
use axum::response::Response;
use uuid::Uuid;

use super::super::actor;
use super::super::logs::relay;
use crate::dto::{
    DatabaseActionRequest, DatabaseCredentialsDto, DatabaseDto, DatabaseLimitsRequest,
    DatabaseLogDto, DatabaseLogQuery, DatabaseLogsQuery,
};
use crate::error::{ApiResult, ErrorEnvelope};
use crate::extract::{Auth, ClientMeta};
use crate::socket_auth::SocketAuth;
use crate::state::AppState;

const DEFAULT_TAIL: u32 = 300;

#[utoipa::path(
    post,
    path = "/api/v1/databases/{id}/actions",
    tag = "databases",
    description = "Redeploys, starts or stops the database container; requires the developer role",
    params(("id" = Uuid, Path, description = "Database id")),
    request_body = DatabaseActionRequest,
    responses(
        (status = 200, body = DatabaseDto),
        (status = 403, body = ErrorEnvelope),
        (status = 404, body = ErrorEnvelope)
    )
)]
pub async fn database_action(
    State(state): State<AppState>,
    Auth(principal): Auth,
    client: ClientMeta,
    Path(id): Path<Uuid>,
    Json(request): Json<DatabaseActionRequest>,
) -> ApiResult<Json<DatabaseDto>> {
    let database = state
        .databases
        .act(actor(&principal, &client), id.into(), request.action.into())
        .await?;
    Ok(Json(database.into()))
}

#[utoipa::path(
    put,
    path = "/api/v1/databases/{id}/limits",
    tag = "databases",
    description = "Sets the memory limit in MiB (swap is disabled) and the CPU limit in thousandths of a core; null removes a limit. Redeploys the container when they change",
    params(("id" = Uuid, Path, description = "Database id")),
    request_body = DatabaseLimitsRequest,
    responses(
        (status = 200, body = DatabaseDto),
        (status = 403, body = ErrorEnvelope),
        (status = 404, body = ErrorEnvelope),
        (status = 422, body = ErrorEnvelope)
    )
)]
pub async fn set_database_limits(
    State(state): State<AppState>,
    Auth(principal): Auth,
    client: ClientMeta,
    Path(id): Path<Uuid>,
    Json(request): Json<DatabaseLimitsRequest>,
) -> ApiResult<Json<DatabaseDto>> {
    let database = state
        .databases
        .set_limits(actor(&principal, &client), id.into(), request.into())
        .await?;
    Ok(Json(database.into()))
}

#[utoipa::path(
    post,
    path = "/api/v1/databases/{id}/credentials",
    tag = "databases",
    description = "Reveals the passwords and connection URLs; requires the developer role and is recorded in the audit log",
    params(("id" = Uuid, Path, description = "Database id")),
    responses(
        (status = 200, body = DatabaseCredentialsDto),
        (status = 403, body = ErrorEnvelope),
        (status = 404, body = ErrorEnvelope)
    )
)]
pub async fn reveal_database_credentials(
    State(state): State<AppState>,
    Auth(principal): Auth,
    client: ClientMeta,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<DatabaseCredentialsDto>> {
    let credentials = state
        .databases
        .credentials(actor(&principal, &client), id.into())
        .await?;
    Ok(Json(credentials.into()))
}

#[utoipa::path(
    get,
    path = "/api/v1/databases/{id}/log",
    tag = "databases",
    description = "Returns the log of the latest deployment from a byte offset",
    params(("id" = Uuid, Path, description = "Database id"), DatabaseLogQuery),
    responses((status = 200, body = DatabaseLogDto), (status = 404, body = ErrorEnvelope))
)]
pub async fn database_deploy_log(
    State(state): State<AppState>,
    Auth(principal): Auth,
    Path(id): Path<Uuid>,
    Query(query): Query<DatabaseLogQuery>,
) -> ApiResult<Json<DatabaseLogDto>> {
    let database = state.databases.get(&principal, id.into()).await?;
    let offset = query.offset.unwrap_or_default();
    let chunk = state
        .databases
        .deploy_log(&principal, id.into(), offset)
        .await?;
    Ok(Json(DatabaseLogDto {
        text: chunk.text,
        next_offset: chunk.next_offset,
        status: database.status.into(),
    }))
}

#[utoipa::path(
    get,
    path = "/api/v1/databases/{id}/logs",
    tag = "databases",
    description = "Streams the database container output over WebSocket; each text frame is a JSON object with source (stdout or stderr) and text",
    params(("id" = Uuid, Path, description = "Database id"), DatabaseLogsQuery),
    responses(
        (status = 101),
        (status = 403, body = ErrorEnvelope),
        (status = 404, body = ErrorEnvelope)
    )
)]
pub async fn database_container_logs(
    upgrade: WebSocketUpgrade,
    State(state): State<AppState>,
    SocketAuth(principal): SocketAuth,
    Path(id): Path<Uuid>,
    Query(query): Query<DatabaseLogsQuery>,
) -> ApiResult<Response> {
    let tail = query.tail.unwrap_or(DEFAULT_TAIL);
    let stream = state
        .databases
        .container_logs(&principal, id.into(), tail)
        .await?;
    Ok(upgrade.on_upgrade(move |socket| relay(socket, stream)))
}
