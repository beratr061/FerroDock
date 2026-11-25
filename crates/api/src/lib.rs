mod client_ip;
mod dto;
mod error;
mod extract;
mod openapi;
mod rate_limit;
mod routes;
mod security;
mod session_cookie;
mod spa;
mod state;

pub use openapi::openapi_json;
pub use state::{ApiSettings, AppState, HealthProbe};

use axum::Router;
use tower_http::compression::CompressionLayer;
use tower_http::request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer};
use tower_http::trace::TraceLayer;

pub fn router(state: AppState) -> Router {
    let request_id = axum::http::HeaderName::from_static("x-request-id");
    Router::new()
        .nest(
            "/api/v1",
            routes::api_routes(state.settings.trust_forwarded_for),
        )
        .fallback(spa::serve)
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            security::csrf_guard,
        ))
        .layer(security::headers())
        .layer(tower::util::option_layer(security::hsts(
            state.settings.secure_cookies,
        )))
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
        .layer(PropagateRequestIdLayer::new(request_id.clone()))
        .layer(SetRequestIdLayer::new(request_id, MakeRequestUuid))
        .with_state(state)
}
