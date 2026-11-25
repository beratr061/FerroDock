use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use ferrodock_domain::auth::NewApiKeyInput;
use uuid::Uuid;

use crate::dto::{ApiKeyDto, CreateApiKeyRequest, CreatedApiKeyResponse};
use crate::error::{ApiResult, ErrorEnvelope};
use crate::extract::Auth;
use crate::state::AppState;

#[utoipa::path(
    get,
    path = "/api/v1/api-keys",
    tag = "api-keys",
    description = "Lists active API keys of the caller in the active organization",
    responses((status = 200, body = Vec<ApiKeyDto>), (status = 401, body = ErrorEnvelope))
)]
pub async fn list(
    State(state): State<AppState>,
    Auth(principal): Auth,
) -> ApiResult<Json<Vec<ApiKeyDto>>> {
    let keys = state.auth.list_api_keys(&principal).await?;
    Ok(Json(keys.into_iter().map(ApiKeyDto::from).collect()))
}

#[utoipa::path(
    post,
    path = "/api/v1/api-keys",
    tag = "api-keys",
    description = "Creates an API key; the secret is returned only in this response",
    request_body = CreateApiKeyRequest,
    responses(
        (status = 201, body = CreatedApiKeyResponse),
        (status = 403, body = ErrorEnvelope),
        (status = 422, body = ErrorEnvelope)
    )
)]
pub async fn create(
    State(state): State<AppState>,
    Auth(principal): Auth,
    Json(request): Json<CreateApiKeyRequest>,
) -> ApiResult<(StatusCode, Json<CreatedApiKeyResponse>)> {
    let input = NewApiKeyInput {
        name: request.name,
        role: request.role.into(),
        expires_in_days: request.expires_in_days,
    };
    let created = state.auth.create_api_key(&principal, input).await?;
    Ok((StatusCode::CREATED, Json(created.into())))
}

#[utoipa::path(
    delete,
    path = "/api/v1/api-keys/{id}",
    tag = "api-keys",
    description = "Revokes one of the caller's API keys",
    params(("id" = Uuid, Path, description = "API key id")),
    responses((status = 204), (status = 404, body = ErrorEnvelope))
)]
pub async fn revoke(
    State(state): State<AppState>,
    Auth(principal): Auth,
    Path(id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    state.auth.revoke_api_key(&principal, id.into()).await?;
    Ok(StatusCode::NO_CONTENT)
}
