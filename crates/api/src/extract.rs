use std::net::SocketAddr;

use axum::extract::{ConnectInfo, FromRequestParts};
use axum::http::header;
use axum::http::request::Parts;
use axum_extra::extract::CookieJar;
use ferrodock_core::AppError;
use ferrodock_domain::auth::Principal;

use crate::client_ip;
use crate::error::ApiError;
use crate::session_cookie;
use crate::state::AppState;

const MAX_USER_AGENT: usize = 512;

#[derive(Debug, Clone)]
pub struct Auth(pub Principal);

impl FromRequestParts<AppState> for Auth {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, ApiError> {
        let bearer = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix("Bearer "));
        if let Some(key) = bearer {
            return Ok(Self(state.auth.authenticate_api_key(key.trim()).await?));
        }
        let jar = CookieJar::from_headers(&parts.headers);
        let cookie = jar
            .get(session_cookie::name(state.settings.secure_cookies))
            .ok_or_else(AppError::unauthorized)?;
        Ok(Self(state.auth.authenticate_session(cookie.value()).await?))
    }
}

#[derive(Debug, Clone, Default)]
pub struct ClientMeta {
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

impl FromRequestParts<AppState> for ClientMeta {
    type Rejection = std::convert::Infallible;

    fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> impl Future<Output = Result<Self, Self::Rejection>> + Send {
        let peer = parts
            .extensions
            .get::<ConnectInfo<SocketAddr>>()
            .map(|info| info.0);
        let user_agent = parts
            .headers
            .get(header::USER_AGENT)
            .and_then(|value| value.to_str().ok())
            .map(|value| value.chars().take(MAX_USER_AGENT).collect());
        std::future::ready(Ok(Self {
            ip_address: client_ip::resolve(
                peer,
                &parts.headers,
                state.settings.trust_forwarded_for,
            )
            .map(|ip| ip.to_string()),
            user_agent,
        }))
    }
}
