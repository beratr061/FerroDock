use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use ferrodock_domain::audit::{Actor, AuditRecord};
use ferrodock_domain::auth::Role;
use ferrodock_engine::updater::UpdateRecord;
use serde_json::json;

use super::actor;
use crate::dto::{UpdateCheckDto, UpdateOverviewDto, UpdateRecordDto};
use crate::error::{ApiResult, ErrorEnvelope};
use crate::extract::{Auth, ClientMeta};
use crate::state::AppState;

async fn record(state: &AppState, actor: Actor<'_>, action: &'static str, update: &UpdateRecord) {
    let entry = AuditRecord {
        action,
        resource_type: "panel",
        resource_id: None,
        metadata: json!({ "from": update.from_image, "to": update.to_image }),
        ip_address: actor.ip_address,
    };
    state.audit.record(actor.principal, entry).await;
}

#[utoipa::path(
    get,
    path = "/api/v1/system/update",
    tag = "system",
    description = "Running version and the outcome of the last update or rollback",
    responses((status = 200, body = UpdateOverviewDto), (status = 403, body = ErrorEnvelope))
)]
pub async fn update_overview(
    State(state): State<AppState>,
    Auth(principal): Auth,
) -> ApiResult<Json<UpdateOverviewDto>> {
    principal.require(Role::Owner)?;
    let last = state.updates.status().await?;
    Ok(Json(UpdateOverviewDto {
        current_version: state.settings.version.to_owned(),
        last: last.map(Into::into),
    }))
}

#[utoipa::path(
    get,
    path = "/api/v1/system/update/check",
    tag = "system",
    description = "Fetches and verifies the signed release manifest",
    responses((status = 200, body = UpdateCheckDto), (status = 403, body = ErrorEnvelope))
)]
pub async fn check_update(
    State(state): State<AppState>,
    Auth(principal): Auth,
) -> ApiResult<Json<UpdateCheckDto>> {
    principal.require(Role::Owner)?;
    Ok(Json(state.updates.check().await?.into()))
}

#[utoipa::path(
    post,
    path = "/api/v1/system/update",
    tag = "system",
    description = "Backs up the panel, pulls the signed release by digest and hands over to the updater; the panel restarts",
    responses((status = 202, body = UpdateRecordDto), (status = 403, body = ErrorEnvelope), (status = 409, body = ErrorEnvelope))
)]
pub async fn start_update(
    State(state): State<AppState>,
    Auth(principal): Auth,
    client: ClientMeta,
) -> ApiResult<(StatusCode, Json<UpdateRecordDto>)> {
    principal.require(Role::Owner)?;
    let update = state.updates.update().await?;
    record(
        &state,
        actor(&principal, &client),
        "panel.update_started",
        &update,
    )
    .await;
    Ok((StatusCode::ACCEPTED, Json(update.into())))
}

#[utoipa::path(
    post,
    path = "/api/v1/system/update/rollback",
    tag = "system",
    description = "Returns the panel to the image it ran before the last update",
    responses((status = 202, body = UpdateRecordDto), (status = 403, body = ErrorEnvelope), (status = 409, body = ErrorEnvelope))
)]
pub async fn roll_back_update(
    State(state): State<AppState>,
    Auth(principal): Auth,
    client: ClientMeta,
) -> ApiResult<(StatusCode, Json<UpdateRecordDto>)> {
    principal.require(Role::Owner)?;
    let update = state.updates.roll_back().await?;
    record(
        &state,
        actor(&principal, &client),
        "panel.rollback_started",
        &update,
    )
    .await;
    Ok((StatusCode::ACCEPTED, Json(update.into())))
}
