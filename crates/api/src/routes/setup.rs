use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum_extra::extract::CookieJar;
use ferrodock_domain::auth::SetupInput;

use crate::dto::{MeResponse, Password, SetupRequest, SetupStatus};
use crate::error::{ApiResult, ErrorEnvelope};
use crate::extract::ClientMeta;
use crate::session_cookie;
use crate::state::AppState;

#[utoipa::path(
    get,
    path = "/api/v1/setup",
    tag = "setup",
    description = "Reports whether the first owner account still has to be created",
    responses((status = 200, body = SetupStatus))
)]
pub async fn status(State(state): State<AppState>) -> ApiResult<Json<SetupStatus>> {
    Ok(Json(SetupStatus {
        required: state.auth.setup_required().await?,
        token_required: state.auth.setup_token_required(),
    }))
}

#[utoipa::path(
    post,
    path = "/api/v1/setup",
    tag = "setup",
    description = "Creates the first owner and organization, then signs the owner in",
    request_body = SetupRequest,
    responses(
        (status = 201, body = MeResponse),
        (status = 409, body = ErrorEnvelope),
        (status = 422, body = ErrorEnvelope)
    )
)]
pub async fn complete(
    State(state): State<AppState>,
    client: ClientMeta,
    jar: CookieJar,
    Json(request): Json<SetupRequest>,
) -> ApiResult<(StatusCode, CookieJar, Json<MeResponse>)> {
    let signed_in = state
        .auth
        .complete_setup(SetupInput {
            email: request.email,
            name: request.name,
            password: request.password.into_inner(),
            organization_name: request.organization_name,
            locale: request.locale.unwrap_or_default(),
            setup_token: request.setup_token.map(Password::into_inner),
            ip_address: client.ip_address,
            user_agent: client.user_agent,
        })
        .await?;
    let cookie = session_cookie::issue(
        &signed_in.token,
        signed_in.expires_at,
        state.settings.secure_cookies,
    );
    Ok((
        StatusCode::CREATED,
        jar.add(cookie),
        Json(signed_in.user.into()),
    ))
}
