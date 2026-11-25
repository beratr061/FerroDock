use axum::extract::FromRequestParts;
use axum::http::header;
use axum::http::request::Parts;
use ferrodock_core::AppError;
use ferrodock_domain::auth::Principal;

use crate::error::ApiError;
use crate::extract::Auth;
use crate::security::origin_allowed;
use crate::state::AppState;

#[derive(Debug, Clone)]
pub struct SocketAuth(pub Principal);

impl FromRequestParts<AppState> for SocketAuth {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, ApiError> {
        let uses_key = parts.headers.contains_key(header::AUTHORIZATION);
        let public_origin = state.settings.public_origin.as_deref();
        if !uses_key && !origin_allowed(&parts.headers, public_origin) {
            return Err(AppError::forbidden()
                .with_param("reason", "CSRF_ORIGIN_MISMATCH")
                .into());
        }
        let Auth(principal) = Auth::from_request_parts(parts, state).await?;
        Ok(Self(principal))
    }
}
