use axum::Json;
use axum::extract::{Path, State};
use uuid::Uuid;

use super::actor;
use crate::dto::{ReplaceVariablesRequest, RevealedVariableDto, VariableDto};
use crate::error::{ApiResult, ErrorEnvelope};
use crate::extract::{Auth, ClientMeta};
use crate::state::AppState;

#[utoipa::path(
    get,
    path = "/api/v1/applications/{id}/variables",
    tag = "variables",
    description = "Lists environment variables; secret values are omitted",
    params(("id" = Uuid, Path, description = "Application id")),
    responses((status = 200, body = Vec<VariableDto>), (status = 404, body = ErrorEnvelope))
)]
pub async fn list_variables(
    State(state): State<AppState>,
    Auth(principal): Auth,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<Vec<VariableDto>>> {
    let variables = state.variables.list(&principal, id.into()).await?;
    Ok(Json(variables.into_iter().map(Into::into).collect()))
}

#[utoipa::path(
    put,
    path = "/api/v1/applications/{id}/variables",
    tag = "variables",
    description = "Replaces all environment variables; send a null value to keep an existing secret unchanged. Applies on the next deployment",
    params(("id" = Uuid, Path, description = "Application id")),
    request_body = ReplaceVariablesRequest,
    responses(
        (status = 200, body = Vec<VariableDto>),
        (status = 403, body = ErrorEnvelope),
        (status = 422, body = ErrorEnvelope)
    )
)]
pub async fn replace_variables(
    State(state): State<AppState>,
    Auth(principal): Auth,
    client: ClientMeta,
    Path(id): Path<Uuid>,
    Json(request): Json<ReplaceVariablesRequest>,
) -> ApiResult<Json<Vec<VariableDto>>> {
    let inputs = request.variables.into_iter().map(Into::into).collect();
    let variables = state
        .variables
        .replace(actor(&principal, &client), id.into(), inputs)
        .await?;
    Ok(Json(variables.into_iter().map(Into::into).collect()))
}

#[utoipa::path(
    post,
    path = "/api/v1/applications/{id}/variables/{key}/reveal",
    tag = "variables",
    description = "Returns one secret value; every reveal is written to the audit log",
    params(
        ("id" = Uuid, Path, description = "Application id"),
        ("key" = String, Path, description = "Variable name")
    ),
    responses(
        (status = 200, body = RevealedVariableDto),
        (status = 403, body = ErrorEnvelope),
        (status = 404, body = ErrorEnvelope)
    )
)]
pub async fn reveal_variable(
    State(state): State<AppState>,
    Auth(principal): Auth,
    client: ClientMeta,
    Path((id, key)): Path<(Uuid, String)>,
) -> ApiResult<Json<RevealedVariableDto>> {
    let value = state
        .variables
        .reveal(actor(&principal, &client), id.into(), &key)
        .await?;
    Ok(Json(RevealedVariableDto { value }))
}
