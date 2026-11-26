mod api_keys;
mod applications;
mod auth;
mod backups;
mod compose_trust;
mod databases;
mod deployments;
mod docker;
mod domains;
mod environments;
mod health;
mod logs;
mod projects;
mod realtime;
mod runtime_config;
mod scheduled_backups;
mod setup;
mod terminal;
mod updates;
mod variables;

use axum::Router;
use axum::routing::{MethodRouter, delete, get, post};
use ferrodock_domain::audit::Actor;
use ferrodock_domain::auth::Principal;

use crate::extract::ClientMeta;
use crate::openapi;
use crate::rate_limit::{CredentialLimiter, credential_limiter};
use crate::state::AppState;

pub(crate) use api_keys::{__path_create, __path_list, __path_revoke};
pub(crate) use applications::{
    __path_application_containers, __path_create_application, __path_delete_application,
    __path_get_application, __path_list_applications, __path_rotate_deploy_key,
    __path_update_application,
};
pub(crate) use auth::{__path_me, __path_sign_in, __path_sign_out};
pub(crate) use backups::{
    __path_create_backup, __path_delete_backup, __path_download_backup, __path_export_master_key,
    __path_list_backups,
};
pub(crate) use compose_trust::__path_set_compose_trust;
pub(crate) use databases::DatabaseApi;
pub(crate) use deployments::{
    __path_cancel_deployment, __path_create_deployment, __path_deployment_log,
    __path_get_deployment, __path_list_deployments, __path_rollback_deployment,
};
pub(crate) use docker::{__path_container_action, __path_docker_info, __path_list_containers};
pub(crate) use domains::{
    __path_check_domain_dns, __path_create_domain, __path_delete_domain, __path_list_domains,
};
pub(crate) use environments::{__path_create_environment, __path_delete_environment};
pub(crate) use health::__path_check;
pub(crate) use logs::__path_container_logs;
pub(crate) use projects::{
    __path_create_project, __path_delete_project, __path_get_project, __path_list_projects,
    __path_update_project,
};
pub(crate) use realtime::__path_realtime;
pub(crate) use runtime_config::{__path_get_runtime_config, __path_replace_runtime_config};
pub(crate) use scheduled_backups::ScheduledBackupApi;
pub(crate) use setup::{__path_complete, __path_status};
pub(crate) use terminal::__path_terminal;
pub(crate) use updates::{
    __path_check_update, __path_roll_back_update, __path_start_update, __path_update_overview,
};
pub(crate) use variables::{
    __path_list_variables, __path_replace_variables, __path_reveal_variable,
};

pub(crate) fn actor<'a>(principal: &'a Principal, client: &'a ClientMeta) -> Actor<'a> {
    Actor {
        principal,
        ip_address: client.ip_address.as_deref(),
    }
}

fn limited(
    route: MethodRouter<AppState>,
    limiter: Option<&CredentialLimiter>,
) -> MethodRouter<AppState> {
    match limiter {
        Some(layer) => route.layer(layer.clone()),
        None => route,
    }
}

fn workload_routes() -> Router<AppState> {
    Router::new()
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
        .merge(application_routes())
}

fn application_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/projects/{id}/applications",
            get(applications::list_applications),
        )
        .route(
            "/environments/{id}/applications",
            post(applications::create_application),
        )
        .route(
            "/applications/{id}",
            get(applications::get_application)
                .patch(applications::update_application)
                .delete(applications::delete_application),
        )
        .route(
            "/applications/{id}/containers",
            get(applications::application_containers),
        )
        .route("/applications/{id}/logs", get(logs::container_logs))
        .route(
            "/applications/{id}/compose-trust",
            axum::routing::put(compose_trust::set_compose_trust),
        )
        .route(
            "/applications/{id}/deploy-key",
            post(applications::rotate_deploy_key),
        )
}

fn deployment_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/applications/{id}/deployments",
            get(deployments::list_deployments).post(deployments::create_deployment),
        )
        .route("/deployments/{id}", get(deployments::get_deployment))
        .route("/deployments/{id}/log", get(deployments::deployment_log))
        .route(
            "/deployments/{id}/rollback",
            post(deployments::rollback_deployment),
        )
        .route(
            "/deployments/{id}/cancel",
            post(deployments::cancel_deployment),
        )
}

fn domain_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/applications/{id}/domains",
            get(domains::list_domains).post(domains::create_domain),
        )
        .route(
            "/applications/{id}/domains/{domain_id}",
            delete(domains::delete_domain),
        )
        .route(
            "/applications/{id}/domains/{domain_id}/dns",
            get(domains::check_domain_dns),
        )
}

fn variable_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/applications/{id}/variables",
            get(variables::list_variables).put(variables::replace_variables),
        )
        .route(
            "/applications/{id}/variables/{key}/reveal",
            post(variables::reveal_variable),
        )
        .route(
            "/applications/{id}/runtime",
            get(runtime_config::get_runtime_config).put(runtime_config::replace_runtime_config),
        )
}

fn system_routes() -> Router<AppState> {
    Router::new()
        .route(
            "/system/backups",
            get(backups::list_backups).post(backups::create_backup),
        )
        .route(
            "/system/backups/{name}",
            get(backups::download_backup).delete(backups::delete_backup),
        )
        .route("/system/master-key", post(backups::export_master_key))
        .route(
            "/system/update",
            get(updates::update_overview).post(updates::start_update),
        )
        .route("/system/update/check", get(updates::check_update))
        .route("/system/update/rollback", post(updates::roll_back_update))
}

fn docker_routes() -> Router<AppState> {
    Router::new()
        .route("/docker/info", get(docker::docker_info))
        .route("/docker/containers", get(docker::list_containers))
        .route(
            "/docker/containers/{reference}/actions",
            post(docker::container_action),
        )
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
        .route("/ws", get(realtime::realtime))
        .route("/terminal", get(terminal::terminal))
        .route("/api-keys", get(api_keys::list).post(api_keys::create))
        .route("/api-keys/{id}", delete(api_keys::revoke))
        .merge(workload_routes())
        .merge(deployment_routes())
        .merge(domain_routes())
        .merge(variable_routes())
        .merge(system_routes())
        .merge(docker_routes())
        .merge(databases::database_routes())
        .merge(scheduled_backups::scheduled_backup_routes())
}
