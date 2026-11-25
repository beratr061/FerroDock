mod api_keys;
mod auth;
mod docker;
mod environments;
mod health;
mod projects;
mod setup;

use axum::Router;
use axum::routing::{MethodRouter, delete, get, post};

use crate::openapi;
use crate::rate_limit::{CredentialLimiter, credential_limiter};
use crate::state::AppState;

pub(crate) use api_keys::{__path_create, __path_list, __path_revoke};
pub(crate) use auth::{__path_me, __path_sign_in, __path_sign_out};
pub(crate) use docker::{__path_container_action, __path_docker_info, __path_list_containers};
pub(crate) use environments::{__path_create_environment, __path_delete_environment};
pub(crate) use health::__path_check;
pub(crate) use projects::{
    __path_create_project, __path_delete_project, __path_get_project, __path_list_projects,
    __path_update_project,
};
pub(crate) use setup::{__path_complete, __path_status};

fn limited(
    route: MethodRouter<AppState>,
    limiter: Option<&CredentialLimiter>,
) -> MethodRouter<AppState> {
    match limiter {
        Some(layer) => route.layer(layer.clone()),
        None => route,
    }
}

pub fn api_routes(trust_forwarded: bool) -> Router<AppState> {
    let limiter = credential_limiter(trust_forwarded);
    Router::new()
        .route("/health", get(health::check))
        .route("/openapi.json", get(openapi::serve))
        .route(
            "/setup",
            get(setup::status).merge(limited(post(setup::complete), limiter.as_ref())),
        )
        .route(
            "/auth/sign-in",
            limited(post(auth::sign_in), limiter.as_ref()),
        )
        .route("/auth/sign-out", post(auth::sign_out))
        .route("/auth/me", get(auth::me))
        .route("/api-keys", get(api_keys::list).post(api_keys::create))
        .route("/api-keys/{id}", delete(api_keys::revoke))
        .route(
            "/projects",
            get(projects::list_projects).post(projects::create_project),
        )
        .route(
            "/projects/{id}",
            get(projects::get_project)
                .patch(projects::update_project)
                .delete(projects::delete_project),
        )
        .route(
            "/projects/{id}/environments",
            post(environments::create_environment),
        )
        .route(
            "/projects/{id}/environments/{environment_id}",
            delete(environments::delete_environment),
        )
        .route("/docker/info", get(docker::docker_info))
        .route("/docker/containers", get(docker::list_containers))
        .route(
            "/docker/containers/{reference}/actions",
            post(docker::container_action),
        )
}
