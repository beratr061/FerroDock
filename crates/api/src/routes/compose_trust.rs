use axum::Json;
use axum::extract::{Path, State};
use uuid::Uuid;

use super::actor;
use crate::dto::{ApplicationDto, ComposeTrustRequest};
use crate::error::{ApiResult, ErrorEnvelope};
use crate::extract::{Auth, ClientMeta};
use crate::state::AppState;

#[utoipa::path(
    put,
    path = "/api/v1/applications/{id}/compose-trust",
    tag = "applications",
    description = "Marks a compose application as trusted so its compose file skips the host-safety checks (privileged mode, host paths, devices and similar). Only the organization owner can turn it on; anyone who can edit the application can turn it off. Changing the source as a non-owner turns it off again",
    params(("id" = Uuid, Path, description = "Application id")),
    request_body = ComposeTrustRequest,
    responses(
        (status = 200, body = ApplicationDto),
        (status = 403, body = ErrorEnvelope),
        (status = 404, body = ErrorEnvelope),
        (status = 422, body = ErrorEnvelope)
    )
)]
pub async fn set_compose_trust(
    State(state): State<AppState>,
    Auth(principal): Auth,
    client: ClientMeta,
    Path(id): Path<Uuid>,
    Json(request): Json<ComposeTrustRequest>,
) -> ApiResult<Json<ApplicationDto>> {
    let application = state
        .applications
        .set_compose_trusted(actor(&principal, &client), id.into(), request.trusted)
        .await?;
    Ok(Json(application.into()))
}
