use axum::body::Bytes;
use axum::extract::{Path, State};
use axum::http::{HeaderMap, StatusCode};
use axum::routing::{get, post};
use axum::{Json, Router};
use ferrodock_domain::webhooks::Delivery;
use utoipa::OpenApi;
use uuid::Uuid;

use super::{actor, limited};
use crate::dto::{CreatedWebhookDto, WebhookDeliveryDto, WebhookDto};
use crate::error::{ApiResult, ErrorEnvelope};
use crate::extract::{Auth, ClientMeta};
use crate::rate_limit::credential_limiter;
use crate::state::AppState;

const EVENT_HEADERS: [&str; 4] = [
    "x-github-event",
    "x-gitea-event",
    "x-forgejo-event",
    "x-gitlab-event",
];

fn header<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers.get(name).and_then(|value| value.to_str().ok())
}

#[utoipa::path(
    get,
    path = "/api/v1/applications/{id}/webhook",
    tag = "webhooks",
    description = "Shows whether push-to-deploy is on for a Git application and how its last delivery went",
    params(("id" = Uuid, Path, description = "Application id")),
    responses((status = 200, body = WebhookDto), (status = 404, body = ErrorEnvelope), (status = 422, body = ErrorEnvelope))
)]
pub async fn get_webhook(
    State(state): State<AppState>,
    Auth(principal): Auth,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<WebhookDto>> {
    Ok(Json(
        state.webhooks.get(&principal, id.into()).await?.into(),
    ))
}

#[utoipa::path(
    post,
    path = "/api/v1/applications/{id}/webhook",
    tag = "webhooks",
    description = "Turns on push-to-deploy, or replaces the secret if it is already on. The secret is returned only in this response",
    params(("id" = Uuid, Path, description = "Application id")),
    responses((status = 200, body = CreatedWebhookDto), (status = 403, body = ErrorEnvelope), (status = 404, body = ErrorEnvelope), (status = 422, body = ErrorEnvelope))
)]
pub async fn rotate_webhook(
    State(state): State<AppState>,
    Auth(principal): Auth,
    client: ClientMeta,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<CreatedWebhookDto>> {
    let (webhook, secret) = state
        .webhooks
        .rotate(actor(&principal, &client), id.into())
        .await?;
    Ok(Json(CreatedWebhookDto {
        webhook: Some(webhook).into(),
        secret,
    }))
}

#[utoipa::path(
    delete,
    path = "/api/v1/applications/{id}/webhook",
    tag = "webhooks",
    description = "Turns push-to-deploy off and forgets the secret",
    params(("id" = Uuid, Path, description = "Application id")),
    responses((status = 204), (status = 403, body = ErrorEnvelope), (status = 404, body = ErrorEnvelope))
)]
pub async fn delete_webhook(
    State(state): State<AppState>,
    Auth(principal): Auth,
    client: ClientMeta,
    Path(id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    state
        .webhooks
        .disable(actor(&principal, &client), id.into())
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/api/v1/webhooks/{id}",
    tag = "webhooks",
    description = "Receives push events from GitHub, GitLab, Gitea or Forgejo. GitHub, Gitea and Forgejo sign the body with the secret (X-Hub-Signature-256 or X-Gitea-Signature); GitLab sends it as X-Gitlab-Token. A push to the application's branch queues a deployment; other events are acknowledged and ignored",
    params(("id" = Uuid, Path, description = "Application id")),
    request_body(content = String, content_type = "application/json"),
    responses((status = 202, body = WebhookDeliveryDto), (status = 401, body = ErrorEnvelope), (status = 422, body = ErrorEnvelope))
)]
pub async fn receive_webhook(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
    body: Bytes,
) -> ApiResult<(StatusCode, Json<WebhookDeliveryDto>)> {
    let delivery = Delivery {
        event: EVENT_HEADERS.iter().find_map(|name| header(&headers, name)),
        hub_signature: header(&headers, "x-hub-signature-256"),
        gitea_signature: header(&headers, "x-gitea-signature"),
        gitlab_token: header(&headers, "x-gitlab-token"),
        body: &body,
    };
    let outcome = state.webhooks.receive(id.into(), delivery).await?;
    Ok((StatusCode::ACCEPTED, Json(outcome.into())))
}

#[derive(OpenApi)]
#[openapi(
    paths(get_webhook, rotate_webhook, delete_webhook, receive_webhook),
    components(schemas(WebhookDto, CreatedWebhookDto, WebhookDeliveryDto))
)]
pub(crate) struct WebhookApi;

pub(crate) fn webhook_routes(trust_forwarded: bool) -> Router<AppState> {
    let limiter = credential_limiter(trust_forwarded);
    Router::new()
        .route(
            "/applications/{id}/webhook",
            get(get_webhook).post(rotate_webhook).delete(delete_webhook),
        )
        .route(
            "/webhooks/{id}",
            limited(post(receive_webhook), limiter.as_ref()),
        )
}
