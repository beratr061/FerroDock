use axum::Json;
use utoipa::OpenApi;

use crate::dto::{
    ApiKeyDto, CreateApiKeyRequest, CreatedApiKeyResponse, HealthResponse, MeResponse,
    MembershipDto, RoleDto, SetupRequest, SetupStatus, SignInRequest,
};
use crate::error::{ErrorBody, ErrorEnvelope};
use crate::routes::{
    __path_check, __path_complete, __path_create, __path_list, __path_me, __path_revoke,
    __path_sign_in, __path_sign_out, __path_status,
};

#[derive(OpenApi)]
#[openapi(
    info(
        title = "FerroDock API",
        description = "Self-hosted PaaS control plane"
    ),
    paths(check, status, complete, sign_in, sign_out, me, list, create, revoke),
    components(schemas(
        ApiKeyDto,
        CreateApiKeyRequest,
        CreatedApiKeyResponse,
        ErrorBody,
        ErrorEnvelope,
        HealthResponse,
        MeResponse,
        MembershipDto,
        RoleDto,
        SetupRequest,
        SetupStatus,
        SignInRequest
    ))
)]
struct ApiDoc;

pub fn openapi_json() -> Result<String, serde_json::Error> {
    ApiDoc::openapi().to_pretty_json()
}

pub async fn serve() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}
