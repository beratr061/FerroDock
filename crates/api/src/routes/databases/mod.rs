mod crud;
mod operations;

use axum::Router;
use axum::routing::{get, post};
use utoipa::OpenApi;

use self::crud::{
    __path_create_database, __path_delete_database, __path_get_database,
    __path_list_database_engines, __path_list_databases, __path_update_database,
};
use self::operations::{
    __path_database_action, __path_database_container_logs, __path_database_deploy_log,
    __path_reveal_database_credentials,
};
use crate::dto::{
    CreateDatabaseRequest, DatabaseActionDto, DatabaseActionRequest, DatabaseCredentialsDto,
    DatabaseDto, DatabaseEngineDto, DatabaseLogDto, DatabaseStatusDto, UpdateDatabaseRequest,
};
use crate::state::AppState;

#[derive(OpenApi)]
#[openapi(
    paths(
        list_database_engines,
        list_databases,
        create_database,
        get_database,
        update_database,
        delete_database,
        database_action,
        reveal_database_credentials,
        database_deploy_log,
        database_container_logs
    ),
    components(schemas(
        CreateDatabaseRequest,
        DatabaseActionDto,
        DatabaseActionRequest,
        DatabaseCredentialsDto,
        DatabaseDto,
        DatabaseEngineDto,
        DatabaseLogDto,
        DatabaseStatusDto,
        UpdateDatabaseRequest
    ))
)]
pub(crate) struct DatabaseApi;

pub(crate) fn database_routes() -> Router<AppState> {
    Router::new()
        .route("/database-engines", get(crud::list_database_engines))
        .route("/projects/{id}/databases", get(crud::list_databases))
        .route("/environments/{id}/databases", post(crud::create_database))
        .route(
            "/databases/{id}",
            get(crud::get_database)
                .patch(crud::update_database)
                .delete(crud::delete_database),
        )
        .route("/databases/{id}/actions", post(operations::database_action))
        .route(
            "/databases/{id}/credentials",
            post(operations::reveal_database_credentials),
        )
        .route("/databases/{id}/log", get(operations::database_deploy_log))
        .route(
            "/databases/{id}/logs",
            get(operations::database_container_logs),
        )
}
