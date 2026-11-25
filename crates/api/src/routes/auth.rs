use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum_extra::extract::CookieJar;
use ferrodock_domain::auth::SignInInput;

use crate::dto::{MeResponse, SignInRequest};
use crate::error::{ApiResult, ErrorEnvelope};
use crate::extract::{Auth, ClientMeta};
use crate::session_cookie;
use crate::state::AppState;

#[utoipa::path(
    post,
    path = "/api/v1/auth/sign-in",
    tag = "auth",
    description = "Signs in with email and password and sets an HttpOnly session cookie",
    request_body = SignInRequest,
    responses(
        (status = 200, body = MeResponse),
        (status = 401, body = ErrorEnvelope),
        (status = 429, description = "Too many attempts from this address")
    )
)]
pub async fn sign_in(
    State(state): State<AppState>,
    client: ClientMeta,
    jar: CookieJar,
    Json(request): Json<SignInRequest>,
) -> ApiResult<(CookieJar, Json<MeResponse>)> {
    let signed_in = state
        .auth
        .sign_in(SignInInput {
            email: request.email,
            password: request.password.into_inner(),
            ip_address: client.ip_address,
            user_agent: client.user_agent,
        })
        .await?;
    let cookie = session_cookie::issue(
        &signed_in.token,
        signed_in.expires_at,
        state.settings.secure_cookies,
    );
    Ok((jar.add(cookie), Json(signed_in.user.into())))
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/sign-out",
    tag = "auth",
    description = "Ends the current session and clears the session cookie",
    responses((status = 204), (status = 401, body = ErrorEnvelope))
)]
pub async fn sign_out(
    State(state): State<AppState>,
    Auth(principal): Auth,
    jar: CookieJar,
) -> ApiResult<(StatusCode, CookieJar)> {
    state.auth.sign_out(&principal).await?;
    let cleared = jar.add(session_cookie::clear(state.settings.secure_cookies));
    Ok((StatusCode::NO_CONTENT, cleared))
}

#[utoipa::path(
    get,
    path = "/api/v1/auth/me",
    tag = "auth",
    description = "Returns the signed-in user with organization memberships",
    responses((status = 200, body = MeResponse), (status = 401, body = ErrorEnvelope))
)]
pub async fn me(
    State(state): State<AppState>,
    Auth(principal): Auth,
) -> ApiResult<Json<MeResponse>> {
    Ok(Json(state.auth.current_user(&principal).await?.into()))
}
