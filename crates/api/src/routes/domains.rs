use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use ferrodock_domain::domains::NewDomainInput;
use uuid::Uuid;

use super::actor;
use crate::dto::{CreateDomainRequest, DnsCheckDto, DomainDto, DomainListDto};
use crate::error::{ApiResult, ErrorEnvelope};
use crate::extract::{Auth, ClientMeta};
use crate::state::AppState;

#[utoipa::path(
    get,
    path = "/api/v1/applications/{id}/domains",
    tag = "domains",
    description = "Lists the domains routed to an application and whether this server runs the proxy",
    params(("id" = Uuid, Path, description = "Application id")),
    responses((status = 200, body = DomainListDto), (status = 404, body = ErrorEnvelope))
)]
pub async fn list_domains(
    State(state): State<AppState>,
    Auth(principal): Auth,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<DomainListDto>> {
    let domains = state.domains.list(&principal, id.into()).await?;
    Ok(Json(DomainListDto {
        proxy_enabled: state.domains.proxy_enabled(),
        domains: domains.into_iter().map(Into::into).collect(),
    }))
}

#[utoipa::path(
    post,
    path = "/api/v1/applications/{id}/domains",
    tag = "domains",
    description = "Routes a host name to the application; HTTPS uses a Let's Encrypt certificate and redirects plain HTTP",
    params(("id" = Uuid, Path, description = "Application id")),
    request_body = CreateDomainRequest,
    responses(
        (status = 201, body = DomainDto),
        (status = 403, body = ErrorEnvelope),
        (status = 409, body = ErrorEnvelope),
        (status = 422, body = ErrorEnvelope)
    )
)]
pub async fn create_domain(
    State(state): State<AppState>,
    Auth(principal): Auth,
    client: ClientMeta,
    Path(id): Path<Uuid>,
    Json(request): Json<CreateDomainRequest>,
) -> ApiResult<(StatusCode, Json<DomainDto>)> {
    let input = NewDomainInput {
        host: request.host,
        path_prefix: request.path_prefix,
        container_port: request.container_port,
        https: request.https.unwrap_or(true),
        service: request.service,
    };
    let domain = state
        .domains
        .add(actor(&principal, &client), id.into(), input)
        .await?;
    Ok((StatusCode::CREATED, Json(domain.into())))
}

#[utoipa::path(
    delete,
    path = "/api/v1/applications/{id}/domains/{domain_id}",
    tag = "domains",
    description = "Stops routing a domain to the application",
    params(
        ("id" = Uuid, Path, description = "Application id"),
        ("domain_id" = Uuid, Path, description = "Domain id")
    ),
    responses((status = 204), (status = 403, body = ErrorEnvelope), (status = 404, body = ErrorEnvelope))
)]
pub async fn delete_domain(
    State(state): State<AppState>,
    Auth(principal): Auth,
    client: ClientMeta,
    Path((id, domain_id)): Path<(Uuid, Uuid)>,
) -> ApiResult<StatusCode> {
    state
        .domains
        .remove(actor(&principal, &client), id.into(), domain_id.into())
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/api/v1/applications/{id}/domains/{domain_id}/dns",
    tag = "domains",
    description = "Resolves the domain and compares it with this server's public address; Cloudflare proxied records are recognised",
    params(
        ("id" = Uuid, Path, description = "Application id"),
        ("domain_id" = Uuid, Path, description = "Domain id")
    ),
    responses((status = 200, body = DnsCheckDto), (status = 404, body = ErrorEnvelope))
)]
pub async fn check_domain_dns(
    State(state): State<AppState>,
    Auth(principal): Auth,
    Path((id, domain_id)): Path<(Uuid, Uuid)>,
) -> ApiResult<Json<DnsCheckDto>> {
    let report = state
        .domains
        .check(&principal, id.into(), domain_id.into())
        .await?;
    Ok(Json(report.into()))
}
