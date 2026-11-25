use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use ferrodock_domain::auth::{AuditRecord, Role};
use ferrodock_infra::docker::ContainerAction;
use serde_json::json;

use crate::dto::{ContainerActionRequest, ContainerDto, ContainerListQuery, EngineInfoDto};
use crate::error::{ApiResult, ErrorEnvelope};
use crate::extract::{Auth, ClientMeta};
use crate::state::AppState;

#[utoipa::path(
    get,
    path = "/api/v1/docker/info",
    tag = "docker",
    description = "Docker engine version on this server; requires the admin role",
    responses((status = 200, body = EngineInfoDto), (status = 503, body = ErrorEnvelope))
)]
pub async fn docker_info(
    State(state): State<AppState>,
    Auth(principal): Auth,
) -> ApiResult<Json<EngineInfoDto>> {
    principal.require(Role::Admin)?;
    Ok(Json(state.docker.info().await?.into()))
}

#[utoipa::path(
    get,
    path = "/api/v1/docker/containers",
    tag = "docker",
    description = "Containers on this server, running only unless all=true; requires the admin role",
    params(ContainerListQuery),
    responses((status = 200, body = Vec<ContainerDto>), (status = 403, body = ErrorEnvelope))
)]
pub async fn list_containers(
    State(state): State<AppState>,
    Auth(principal): Auth,
    Query(query): Query<ContainerListQuery>,
) -> ApiResult<Json<Vec<ContainerDto>>> {
    principal.require(Role::Admin)?;
    let containers = state
        .docker
        .list_containers(query.all.unwrap_or(false))
        .await?;
    Ok(Json(
        containers.into_iter().map(ContainerDto::from).collect(),
    ))
}

#[utoipa::path(
    post,
    path = "/api/v1/docker/containers/{reference}/actions",
    tag = "docker",
    description = "Starts, stops or restarts a container by id or name; requires the admin role",
    params(("reference" = String, Path, description = "Container id or name")),
    request_body = ContainerActionRequest,
    responses(
        (status = 204),
        (status = 403, body = ErrorEnvelope),
        (status = 404, body = ErrorEnvelope),
        (status = 422, body = ErrorEnvelope)
    )
)]
pub async fn container_action(
    State(state): State<AppState>,
    Auth(principal): Auth,
    client: ClientMeta,
    Path(reference): Path<String>,
    Json(request): Json<ContainerActionRequest>,
) -> ApiResult<StatusCode> {
    principal.require(Role::Admin)?;
    let action = ContainerAction::from(request.action);
    state.docker.apply(&reference, action).await?;
    let record = AuditRecord {
        action: "container.action",
        resource_type: "container",
        metadata: json!({ "container": reference, "action": action }),
        ip_address: client.ip_address.as_deref(),
    };
    state.auth.record_action(&principal, record).await;
    Ok(StatusCode::NO_CONTENT)
}
