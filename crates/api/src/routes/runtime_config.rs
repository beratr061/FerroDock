use axum::Json;
use axum::extract::{Path, State};
use uuid::Uuid;

use super::actor;
use crate::dto::RuntimeConfigDto;
use crate::error::{ApiResult, ErrorEnvelope};
use crate::extract::{Auth, ClientMeta};
use crate::state::AppState;

#[utoipa::path(
    get,
    path = "/api/v1/applications/{id}/runtime",
    tag = "applications",
    description = "Lists the volumes, bind mounts and published host ports of an application",
    params(("id" = Uuid, Path, description = "Application id")),
    responses((status = 200, body = RuntimeConfigDto), (status = 404, body = ErrorEnvelope))
)]
pub async fn get_runtime_config(
    State(state): State<AppState>,
    Auth(principal): Auth,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<RuntimeConfigDto>> {
    let config = state.runtime_configs.get(&principal, id.into()).await?;
    Ok(Json(config.into()))
}

#[utoipa::path(
    put,
    path = "/api/v1/applications/{id}/runtime",
    tag = "applications",
    description = "Replaces the volumes, bind mounts and published host ports; bind mounts need the admin role and published ports turn blue-green into recreate. Applies on the next deployment",
    params(("id" = Uuid, Path, description = "Application id")),
    request_body = RuntimeConfigDto,
    responses(
        (status = 200, body = RuntimeConfigDto),
        (status = 403, body = ErrorEnvelope),
        (status = 409, body = ErrorEnvelope),
        (status = 422, body = ErrorEnvelope)
    )
)]
pub async fn replace_runtime_config(
    State(state): State<AppState>,
    Auth(principal): Auth,
    client: ClientMeta,
    Path(id): Path<Uuid>,
    Json(request): Json<RuntimeConfigDto>,
) -> ApiResult<Json<RuntimeConfigDto>> {
    let config = state
        .runtime_configs
        .replace(actor(&principal, &client), id.into(), request.into())
        .await?;
    Ok(Json(config.into()))
}
