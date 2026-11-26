use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use uuid::Uuid;

use super::actor;
use crate::dto::{
    ApplicationDto, CreateApplicationRequest, RemoveDataQuery, UpdateApplicationRequest,
    WorkloadDto,
};
use crate::error::{ApiResult, ErrorEnvelope};
use crate::extract::{Auth, ClientMeta};
use crate::state::AppState;

#[utoipa::path(
    get,
    path = "/api/v1/projects/{id}/applications",
    tag = "applications",
    description = "Lists applications in every environment of a project",
    params(("id" = Uuid, Path, description = "Project id")),
    responses((status = 200, body = Vec<ApplicationDto>), (status = 401, body = ErrorEnvelope))
)]
pub async fn list_applications(
    State(state): State<AppState>,
    Auth(principal): Auth,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<Vec<ApplicationDto>>> {
    let applications = state.applications.list(&principal, id.into()).await?;
    Ok(Json(applications.into_iter().map(Into::into).collect()))
}

#[utoipa::path(
    post,
    path = "/api/v1/environments/{id}/applications",
    tag = "applications",
    description = "Creates an application from a Git repository or an image; requires the developer role. SSH repositories get a generated deploy key",
    params(("id" = Uuid, Path, description = "Environment id")),
    request_body = CreateApplicationRequest,
    responses(
        (status = 201, body = ApplicationDto),
        (status = 403, body = ErrorEnvelope),
        (status = 404, body = ErrorEnvelope),
        (status = 409, body = ErrorEnvelope),
        (status = 422, body = ErrorEnvelope)
    )
)]
pub async fn create_application(
    State(state): State<AppState>,
    Auth(principal): Auth,
    client: ClientMeta,
    Path(id): Path<Uuid>,
    Json(request): Json<CreateApplicationRequest>,
) -> ApiResult<(StatusCode, Json<ApplicationDto>)> {
    let application = state
        .applications
        .create(actor(&principal, &client), id.into(), request.into())
        .await?;
    Ok((StatusCode::CREATED, Json(application.into())))
}

#[utoipa::path(
    get,
    path = "/api/v1/applications/{id}",
    tag = "applications",
    description = "Returns one application; the deploy key is shown as its public half only",
    params(("id" = Uuid, Path, description = "Application id")),
    responses((status = 200, body = ApplicationDto), (status = 404, body = ErrorEnvelope))
)]
pub async fn get_application(
    State(state): State<AppState>,
    Auth(principal): Auth,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<ApplicationDto>> {
    let application = state.applications.get(&principal, id.into()).await?;
    Ok(Json(application.into()))
}

#[utoipa::path(
    patch,
    path = "/api/v1/applications/{id}",
    tag = "applications",
    description = "Changes name, description, source or deployment strategy; requires the developer role",
    params(("id" = Uuid, Path, description = "Application id")),
    request_body = UpdateApplicationRequest,
    responses(
        (status = 200, body = ApplicationDto),
        (status = 403, body = ErrorEnvelope),
        (status = 404, body = ErrorEnvelope),
        (status = 422, body = ErrorEnvelope)
    )
)]
pub async fn update_application(
    State(state): State<AppState>,
    Auth(principal): Auth,
    client: ClientMeta,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateApplicationRequest>,
) -> ApiResult<Json<ApplicationDto>> {
    let application = state
        .applications
        .update(actor(&principal, &client), id.into(), request.into())
        .await?;
    Ok(Json(application.into()))
}

#[utoipa::path(
    post,
    path = "/api/v1/applications/{id}/deploy-key",
    tag = "applications",
    description = "Replaces the deploy key of an SSH Git source; the old key stops working once removed from the repository",
    params(("id" = Uuid, Path, description = "Application id")),
    responses(
        (status = 200, body = ApplicationDto),
        (status = 403, body = ErrorEnvelope),
        (status = 404, body = ErrorEnvelope),
        (status = 422, body = ErrorEnvelope)
    )
)]
pub async fn rotate_deploy_key(
    State(state): State<AppState>,
    Auth(principal): Auth,
    client: ClientMeta,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<ApplicationDto>> {
    let application = state
        .applications
        .rotate_deploy_key(actor(&principal, &client), id.into())
        .await?;
    Ok(Json(application.into()))
}

#[utoipa::path(
    delete,
    path = "/api/v1/applications/{id}",
    tag = "applications",
    description = "Deletes an application and its containers; its volumes are kept unless remove_data is true. Requires the admin role",
    params(("id" = Uuid, Path, description = "Application id"), RemoveDataQuery),
    responses(
        (status = 204),
        (status = 403, body = ErrorEnvelope),
        (status = 404, body = ErrorEnvelope)
    )
)]
pub async fn delete_application(
    State(state): State<AppState>,
    Auth(principal): Auth,
    client: ClientMeta,
    Path(id): Path<Uuid>,
    Query(query): Query<RemoveDataQuery>,
) -> ApiResult<StatusCode> {
    let remove_data = query.remove_data.unwrap_or(false);
    state
        .applications
        .delete(actor(&principal, &client), id.into(), remove_data)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/api/v1/applications/{id}/containers",
    tag = "applications",
    description = "Lists the containers currently running for an application",
    params(("id" = Uuid, Path, description = "Application id")),
    responses((status = 200, body = Vec<WorkloadDto>), (status = 404, body = ErrorEnvelope))
)]
pub async fn application_containers(
    State(state): State<AppState>,
    Auth(principal): Auth,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<Vec<WorkloadDto>>> {
    let containers = state.applications.containers(&principal, id.into()).await?;
    Ok(Json(containers.into_iter().map(Into::into).collect()))
}
