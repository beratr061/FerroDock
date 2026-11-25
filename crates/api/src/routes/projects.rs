use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use ferrodock_domain::projects::{Actor, NewProjectInput, ProjectUpdate};
use uuid::Uuid;

use crate::dto::{CreateProjectRequest, ProjectDto, UpdateProjectRequest};
use crate::error::{ApiResult, ErrorEnvelope};
use crate::extract::{Auth, ClientMeta};
use crate::state::AppState;

#[utoipa::path(
    get,
    path = "/api/v1/projects",
    tag = "projects",
    description = "Lists projects of the active organization with their environments",
    responses((status = 200, body = Vec<ProjectDto>), (status = 401, body = ErrorEnvelope))
)]
pub async fn list_projects(
    State(state): State<AppState>,
    Auth(principal): Auth,
) -> ApiResult<Json<Vec<ProjectDto>>> {
    let projects = state.projects.list(&principal).await?;
    Ok(Json(projects.into_iter().map(ProjectDto::from).collect()))
}

#[utoipa::path(
    post,
    path = "/api/v1/projects",
    tag = "projects",
    description = "Creates a project with a production environment; requires the developer role",
    request_body = CreateProjectRequest,
    responses(
        (status = 201, body = ProjectDto),
        (status = 403, body = ErrorEnvelope),
        (status = 409, body = ErrorEnvelope),
        (status = 422, body = ErrorEnvelope)
    )
)]
pub async fn create_project(
    State(state): State<AppState>,
    Auth(principal): Auth,
    client: ClientMeta,
    Json(request): Json<CreateProjectRequest>,
) -> ApiResult<(StatusCode, Json<ProjectDto>)> {
    let actor = Actor {
        principal: &principal,
        ip_address: client.ip_address.as_deref(),
    };
    let input = NewProjectInput {
        name: request.name,
        description: request.description,
    };
    let project = state.projects.create(actor, input).await?;
    Ok((StatusCode::CREATED, Json(project.into())))
}

#[utoipa::path(
    get,
    path = "/api/v1/projects/{id}",
    tag = "projects",
    description = "Returns one project with its environments",
    params(("id" = Uuid, Path, description = "Project id")),
    responses((status = 200, body = ProjectDto), (status = 404, body = ErrorEnvelope))
)]
pub async fn get_project(
    State(state): State<AppState>,
    Auth(principal): Auth,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<ProjectDto>> {
    let project = state.projects.get(&principal, id.into()).await?;
    Ok(Json(project.into()))
}

#[utoipa::path(
    patch,
    path = "/api/v1/projects/{id}",
    tag = "projects",
    description = "Renames a project or changes its description; the slug stays fixed",
    params(("id" = Uuid, Path, description = "Project id")),
    request_body = UpdateProjectRequest,
    responses(
        (status = 200, body = ProjectDto),
        (status = 403, body = ErrorEnvelope),
        (status = 404, body = ErrorEnvelope),
        (status = 422, body = ErrorEnvelope)
    )
)]
pub async fn update_project(
    State(state): State<AppState>,
    Auth(principal): Auth,
    client: ClientMeta,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateProjectRequest>,
) -> ApiResult<Json<ProjectDto>> {
    let actor = Actor {
        principal: &principal,
        ip_address: client.ip_address.as_deref(),
    };
    let update = ProjectUpdate {
        name: request.name,
        description: request.description,
    };
    let project = state.projects.update(actor, id.into(), update).await?;
    Ok(Json(project.into()))
}

#[utoipa::path(
    delete,
    path = "/api/v1/projects/{id}",
    tag = "projects",
    description = "Deletes a project and its environments; requires the admin role",
    params(("id" = Uuid, Path, description = "Project id")),
    responses(
        (status = 204),
        (status = 403, body = ErrorEnvelope),
        (status = 404, body = ErrorEnvelope)
    )
)]
pub async fn delete_project(
    State(state): State<AppState>,
    Auth(principal): Auth,
    client: ClientMeta,
    Path(id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    let actor = Actor {
        principal: &principal,
        ip_address: client.ip_address.as_deref(),
    };
    state.projects.delete(actor, id.into()).await?;
    Ok(StatusCode::NO_CONTENT)
}
