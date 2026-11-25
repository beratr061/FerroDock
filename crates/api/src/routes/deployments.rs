use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use uuid::Uuid;

use super::actor;
use crate::dto::{DeploymentDto, DeploymentLogDto, DeploymentLogQuery};
use crate::error::{ApiResult, ErrorEnvelope};
use crate::extract::{Auth, ClientMeta};
use crate::state::AppState;

#[utoipa::path(
    get,
    path = "/api/v1/applications/{id}/deployments",
    tag = "deployments",
    description = "Lists the most recent deployments of an application, newest first",
    params(("id" = Uuid, Path, description = "Application id")),
    responses((status = 200, body = Vec<DeploymentDto>), (status = 404, body = ErrorEnvelope))
)]
pub async fn list_deployments(
    State(state): State<AppState>,
    Auth(principal): Auth,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<Vec<DeploymentDto>>> {
    let deployments = state.deployments.list(&principal, id.into()).await?;
    Ok(Json(deployments.into_iter().map(Into::into).collect()))
}

#[utoipa::path(
    post,
    path = "/api/v1/applications/{id}/deployments",
    tag = "deployments",
    description = "Queues a deployment of the application's current source; requires the developer role",
    params(("id" = Uuid, Path, description = "Application id")),
    responses(
        (status = 202, body = DeploymentDto),
        (status = 403, body = ErrorEnvelope),
        (status = 404, body = ErrorEnvelope)
    )
)]
pub async fn create_deployment(
    State(state): State<AppState>,
    Auth(principal): Auth,
    client: ClientMeta,
    Path(id): Path<Uuid>,
) -> ApiResult<(StatusCode, Json<DeploymentDto>)> {
    let deployment = state
        .deployments
        .deploy(actor(&principal, &client), id.into())
        .await?;
    Ok((StatusCode::ACCEPTED, Json(deployment.into())))
}

#[utoipa::path(
    get,
    path = "/api/v1/deployments/{id}",
    tag = "deployments",
    description = "Returns one deployment with its current status",
    params(("id" = Uuid, Path, description = "Deployment id")),
    responses((status = 200, body = DeploymentDto), (status = 404, body = ErrorEnvelope))
)]
pub async fn get_deployment(
    State(state): State<AppState>,
    Auth(principal): Auth,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<DeploymentDto>> {
    Ok(Json(
        state.deployments.get(&principal, id.into()).await?.into(),
    ))
}

#[utoipa::path(
    get,
    path = "/api/v1/deployments/{id}/log",
    tag = "deployments",
    description = "Returns deployment output from a byte offset; call again with next_offset to follow",
    params(("id" = Uuid, Path, description = "Deployment id"), DeploymentLogQuery),
    responses((status = 200, body = DeploymentLogDto), (status = 404, body = ErrorEnvelope))
)]
pub async fn deployment_log(
    State(state): State<AppState>,
    Auth(principal): Auth,
    Path(id): Path<Uuid>,
    Query(query): Query<DeploymentLogQuery>,
) -> ApiResult<Json<DeploymentLogDto>> {
    let offset = query.offset.unwrap_or_default();
    let (deployment, chunk) = state.deployments.log(&principal, id.into(), offset).await?;
    Ok(Json(DeploymentLogDto::new(&deployment, chunk)))
}

#[utoipa::path(
    post,
    path = "/api/v1/deployments/{id}/rollback",
    tag = "deployments",
    description = "Queues a new deployment that runs the image of this earlier successful deployment",
    params(("id" = Uuid, Path, description = "Deployment to roll back to")),
    responses(
        (status = 202, body = DeploymentDto),
        (status = 403, body = ErrorEnvelope),
        (status = 404, body = ErrorEnvelope),
        (status = 422, body = ErrorEnvelope)
    )
)]
pub async fn rollback_deployment(
    State(state): State<AppState>,
    Auth(principal): Auth,
    client: ClientMeta,
    Path(id): Path<Uuid>,
) -> ApiResult<(StatusCode, Json<DeploymentDto>)> {
    let deployment = state
        .deployments
        .rollback(actor(&principal, &client), id.into())
        .await?;
    Ok((StatusCode::ACCEPTED, Json(deployment.into())))
}

#[utoipa::path(
    post,
    path = "/api/v1/deployments/{id}/cancel",
    tag = "deployments",
    description = "Cancels a queued deployment immediately or stops a running one",
    params(("id" = Uuid, Path, description = "Deployment id")),
    responses(
        (status = 200, body = DeploymentDto),
        (status = 404, body = ErrorEnvelope),
        (status = 409, body = ErrorEnvelope)
    )
)]
pub async fn cancel_deployment(
    State(state): State<AppState>,
    Auth(principal): Auth,
    client: ClientMeta,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<DeploymentDto>> {
    let deployment = state
        .deployments
        .cancel(actor(&principal, &client), id.into())
        .await?;
    Ok(Json(deployment.into()))
}
