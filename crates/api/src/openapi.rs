use axum::Json;
use utoipa::OpenApi;

use crate::dto::{
    ApiKeyDto, ContainerActionDto, ContainerActionRequest, ContainerDto, CreateApiKeyRequest,
    CreateEnvironmentRequest, CreateProjectRequest, CreatedApiKeyResponse, EngineInfoDto,
    EnvironmentDto, HealthResponse, MeResponse, MembershipDto, ProjectDto, RoleDto, SetupRequest,
    SetupStatus, SignInRequest, UpdateProjectRequest,
};
use crate::error::{ErrorBody, ErrorEnvelope};
use crate::routes::{
    __path_check, __path_complete, __path_container_action, __path_create,
    __path_create_environment, __path_create_project, __path_delete_environment,
    __path_delete_project, __path_docker_info, __path_get_project, __path_list,
    __path_list_containers, __path_list_projects, __path_me, __path_revoke, __path_sign_in,
    __path_sign_out, __path_status, __path_update_project,
};

#[derive(OpenApi)]
#[openapi(
    info(
        title = "FerroDock API",
        description = "Self-hosted PaaS control plane"
    ),
    paths(
        check,
        status,
        complete,
        sign_in,
        sign_out,
        me,
        list,
        create,
        revoke,
        list_projects,
        create_project,
        get_project,
        update_project,
        delete_project,
        create_environment,
        delete_environment,
        docker_info,
        list_containers,
        container_action
    ),
    components(schemas(
        ApiKeyDto,
        ContainerActionDto,
        ContainerActionRequest,
        ContainerDto,
        EngineInfoDto,
        CreateApiKeyRequest,
        CreatedApiKeyResponse,
        CreateEnvironmentRequest,
        CreateProjectRequest,
        EnvironmentDto,
        ProjectDto,
        UpdateProjectRequest,
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
