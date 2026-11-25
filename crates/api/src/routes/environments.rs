use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use ferrodock_domain::projects::Actor;
use uuid::Uuid;

use crate::dto::{CreateEnvironmentRequest, EnvironmentDto};
use crate::error::{ApiResult, ErrorEnvelope};
use crate::extract::{Auth, ClientMeta};
use crate::state::AppState;

#[utoipa::path(
    post,
    path = "/api/v1/projects/{id}/environments",
    tag = "projects",
    description = "Adds an environment to a project; requires the developer role",
    params(("id" = Uuid, Path, description = "Project id")),
    request_body = CreateEnvironmentRequest,
    responses(
        (status = 201, body = EnvironmentDto),
        (status = 403, body = ErrorEnvelope),
        (status = 404, body = ErrorEnvelope),
        (status = 409, body = ErrorEnvelope),
        (status = 422, body = ErrorEnvelope)
    )
)]
pub async fn create_environment(
    State(state): State<AppState>,
    Auth(principal): Auth,
    client: ClientMeta,
    Path(id): Path<Uuid>,
    Json(request): Json<CreateEnvironmentRequest>,
) -> ApiResult<(StatusCode, Json<EnvironmentDto>)> {
    let actor = Actor {
        principal: &principal,
        ip_address: client.ip_address.as_deref(),
    };
    let environment = state
        .projects
        .add_environment(actor, id.into(), &request.name)
        .await?;
    Ok((StatusCode::CREATED, Json(environment.into())))
}

#[utoipa::path(
    delete,
    path = "/api/v1/projects/{id}/environments/{environment_id}",
    tag = "projects",
    description = "Deletes an environment; the last environment of a project cannot be deleted",
    params(
        ("id" = Uuid, Path, description = "Project id"),
        ("environment_id" = Uuid, Path, description = "Environment id")
    ),
    responses(
        (status = 204),
        (status = 403, body = ErrorEnvelope),
        (status = 404, body = ErrorEnvelope),
        (status = 409, body = ErrorEnvelope)
    )
)]
pub async fn delete_environment(
    State(state): State<AppState>,
    Auth(principal): Auth,
    client: ClientMeta,
    Path((id, environment_id)): Path<(Uuid, Uuid)>,
) -> ApiResult<StatusCode> {
    let actor = Actor {
        principal: &principal,
        ip_address: client.ip_address.as_deref(),
    };
    state
        .projects
        .remove_environment(actor, id.into(), environment_id.into())
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
