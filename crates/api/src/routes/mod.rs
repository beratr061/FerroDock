mod api_keys;
mod auth;
mod docker;
mod health;
mod setup;

use axum::Router;
use axum::routing::{MethodRouter, delete, get, post};

use crate::openapi;
use crate::rate_limit::{CredentialLimiter, credential_limiter};
use crate::state::AppState;

pub(crate) use api_keys::{__path_create, __path_list, __path_revoke};
pub(crate) use auth::{__path_me, __path_sign_in, __path_sign_out};
pub(crate) use docker::{__path_container_action, __path_docker_info, __path_list_containers};
pub(crate) use health::__path_check;
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
        .route("/docker/info", get(docker::docker_info))
        .route("/docker/containers", get(docker::list_containers))
        .route(
            "/docker/containers/{reference}/actions",
            post(docker::container_action),
        )
}
