use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use ferrodock_domain::databases::drivers;
use uuid::Uuid;

use super::super::actor;
use crate::dto::{
    CreateDatabaseRequest, DatabaseDto, DatabaseEngineDto, RemoveDataQuery, UpdateDatabaseRequest,
};
use crate::error::{ApiResult, ErrorEnvelope};
use crate::extract::{Auth, ClientMeta};
use crate::state::AppState;

#[utoipa::path(
    get,
    path = "/api/v1/database-engines",
    tag = "databases",
    description = "Lists the database engines FerroDock can run, with their supported versions; the first version is the default",
    responses((status = 200, body = Vec<DatabaseEngineDto>), (status = 401, body = ErrorEnvelope))
)]
pub async fn list_database_engines(Auth(_principal): Auth) -> Json<Vec<DatabaseEngineDto>> {
    Json(drivers().iter().map(|driver| (*driver).into()).collect())
}

#[utoipa::path(
    get,
    path = "/api/v1/projects/{id}/databases",
    tag = "databases",
    description = "Lists managed databases in every environment of a project",
    params(("id" = Uuid, Path, description = "Project id")),
    responses((status = 200, body = Vec<DatabaseDto>), (status = 401, body = ErrorEnvelope))
)]
pub async fn list_databases(
    State(state): State<AppState>,
    Auth(principal): Auth,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<Vec<DatabaseDto>>> {
    let databases = state.databases.list(&principal, id.into()).await?;
    Ok(Json(databases.into_iter().map(Into::into).collect()))
}

#[utoipa::path(
    post,
    path = "/api/v1/environments/{id}/databases",
    tag = "databases",
    description = "Creates a managed database with generated passwords and queues its first deployment; requires the developer role",
    params(("id" = Uuid, Path, description = "Environment id")),
    request_body = CreateDatabaseRequest,
    responses(
        (status = 201, body = DatabaseDto),
        (status = 403, body = ErrorEnvelope),
        (status = 404, body = ErrorEnvelope),
        (status = 409, body = ErrorEnvelope),
        (status = 422, body = ErrorEnvelope)
    )
)]
pub async fn create_database(
    State(state): State<AppState>,
    Auth(principal): Auth,
    client: ClientMeta,
    Path(id): Path<Uuid>,
    Json(request): Json<CreateDatabaseRequest>,
) -> ApiResult<(StatusCode, Json<DatabaseDto>)> {
    let database = state
        .databases
        .create(actor(&principal, &client), id.into(), request.into())
        .await?;
    Ok((StatusCode::CREATED, Json(database.into())))
}

#[utoipa::path(
    get,
    path = "/api/v1/databases/{id}",
    tag = "databases",
    description = "Returns one managed database without its passwords",
    params(("id" = Uuid, Path, description = "Database id")),
    responses((status = 200, body = DatabaseDto), (status = 404, body = ErrorEnvelope))
)]
pub async fn get_database(
    State(state): State<AppState>,
    Auth(principal): Auth,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<DatabaseDto>> {
    let database = state.databases.get(&principal, id.into()).await?;
    Ok(Json(database.into()))
}

#[utoipa::path(
    patch,
    path = "/api/v1/databases/{id}",
    tag = "databases",
    description = "Publishes the database on a host port, or removes the published port when null; redeploys the container when it changes",
    params(("id" = Uuid, Path, description = "Database id")),
    request_body = UpdateDatabaseRequest,
    responses(
        (status = 200, body = DatabaseDto),
        (status = 403, body = ErrorEnvelope),
        (status = 409, body = ErrorEnvelope),
        (status = 422, body = ErrorEnvelope)
    )
)]
pub async fn update_database(
    State(state): State<AppState>,
    Auth(principal): Auth,
    client: ClientMeta,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateDatabaseRequest>,
) -> ApiResult<Json<DatabaseDto>> {
    let database = state
        .databases
        .set_external_port(actor(&principal, &client), id.into(), request.external_port)
        .await?;
    Ok(Json(database.into()))
}

#[utoipa::path(
    delete,
    path = "/api/v1/databases/{id}",
    tag = "databases",
    description = "Removes the database container; the data volume is kept unless remove_data is true. Requires the admin role",
    params(("id" = Uuid, Path, description = "Database id"), RemoveDataQuery),
    responses(
        (status = 204),
        (status = 403, body = ErrorEnvelope),
        (status = 404, body = ErrorEnvelope),
        (status = 409, body = ErrorEnvelope)
    )
)]
pub async fn delete_database(
    State(state): State<AppState>,
    Auth(principal): Auth,
    client: ClientMeta,
    Path(id): Path<Uuid>,
    Query(query): Query<RemoveDataQuery>,
) -> ApiResult<StatusCode> {
    let remove_data = query.remove_data.unwrap_or(false);
    state
        .databases
        .delete(actor(&principal, &client), id.into(), remove_data)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
