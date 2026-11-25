use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;

use crate::dto::HealthResponse;
use crate::state::AppState;

#[utoipa::path(
    get,
    path = "/api/v1/health",
    tag = "system",
    description = "Liveness and database readiness of the control plane",
    responses(
        (status = 200, body = HealthResponse),
        (status = 503, body = HealthResponse)
    )
)]
pub async fn check(State(state): State<AppState>) -> (StatusCode, Json<HealthResponse>) {
    let database_ok = state.health.database().await.is_ok();
    let status = if database_ok {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    let body = HealthResponse {
        status: if database_ok { "ok" } else { "degraded" },
        version: state.settings.version,
        database: if database_ok { "ok" } else { "unavailable" },
    };
    (status, Json(body))
}
