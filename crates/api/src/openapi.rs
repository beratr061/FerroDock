use axum::Json;
use axum::http::StatusCode;
use utoipa::OpenApi;
use utoipa::openapi::RefOr;

use crate::dto::{
    ApiKeyDto, ApplicationDto, BackupDto, BackupRequestedDto, BasicAuthRequest, BuildKindDto,
    ComposeTrustRequest, ContainerActionDto, ContainerActionRequest, ContainerDto,
    CreateApiKeyRequest, CreateApplicationRequest, CreateDomainRequest, CreateEnvironmentRequest,
    CreateProjectRequest, CreatedApiKeyResponse, DeploymentDto, DeploymentKindDto,
    DeploymentLogDto, DeploymentStatusDto, DnsCheckDto, DnsVerdictDto, DomainDto, DomainListDto,
    EngineInfoDto, EnvironmentDto, HealthResponse, MasterKeyDto, MasterKeyRequest, MeResponse,
    MembershipDto, MountDto, MountKindDto, PortDto, ProjectDto, ProtocolDto,
    ReplaceVariablesRequest, RevealedVariableDto, RoleDto, RuntimeConfigDto, SetupRequest,
    SetupStatus, SignInRequest, SourceDto, SourceRequest, StrategyDto, UpdateApplicationRequest,
    UpdateCheckDto, UpdateOverviewDto, UpdateProjectRequest, UpdateRecordDto, UpdateStateDto,
    VariableDto, VariableInputDto, WorkloadDto,
};
use crate::error::{ErrorBody, ErrorEnvelope};
use crate::routes::{
    __path_application_containers, __path_cancel_deployment, __path_check, __path_check_domain_dns,
    __path_check_update, __path_complete, __path_container_action, __path_container_logs,
    __path_create, __path_create_application, __path_create_backup, __path_create_deployment,
    __path_create_domain, __path_create_environment, __path_create_project,
    __path_delete_application, __path_delete_backup, __path_delete_domain,
    __path_delete_environment, __path_delete_project, __path_deployment_log, __path_docker_info,
    __path_download_backup, __path_export_master_key, __path_get_application,
    __path_get_deployment, __path_get_project, __path_get_runtime_config, __path_list,
    __path_list_applications, __path_list_backups, __path_list_containers, __path_list_deployments,
    __path_list_domains, __path_list_projects, __path_list_variables, __path_me, __path_realtime,
    __path_replace_runtime_config, __path_replace_variables, __path_reveal_variable, __path_revoke,
    __path_roll_back_update, __path_rollback_deployment, __path_rotate_deploy_key,
    __path_set_compose_trust, __path_sign_in, __path_sign_out, __path_start_update, __path_status,
    __path_terminal, __path_update_application, __path_update_overview, __path_update_project,
    DatabaseApi, ScheduledBackupApi,
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
        list_applications,
        create_application,
        get_application,
        update_application,
        rotate_deploy_key,
        set_compose_trust,
        delete_application,
        list_deployments,
        create_deployment,
        get_deployment,
        deployment_log,
        rollback_deployment,
        cancel_deployment,
        list_domains,
        create_domain,
        delete_domain,
        check_domain_dns,
        list_variables,
        replace_variables,
        reveal_variable,
        realtime,
        terminal,
        application_containers,
        container_logs,
        get_runtime_config,
        replace_runtime_config,
        list_backups,
        create_backup,
        download_backup,
        delete_backup,
        export_master_key,
        update_overview,
        check_update,
        start_update,
        roll_back_update,
        delete_environment,
        docker_info,
        list_containers,
        container_action
    ),
    components(schemas(
        ApiKeyDto,
        UpdateCheckDto,
        UpdateOverviewDto,
        UpdateRecordDto,
        UpdateStateDto,
        BackupDto,
        BackupRequestedDto,
        MasterKeyDto,
        MasterKeyRequest,
        WorkloadDto,
        ReplaceVariablesRequest,
        RevealedVariableDto,
        VariableDto,
        VariableInputDto,
        CreateDomainRequest,
        BasicAuthRequest,
        RuntimeConfigDto,
        MountDto,
        MountKindDto,
        PortDto,
        ProtocolDto,
        DnsCheckDto,
        DnsVerdictDto,
        DomainDto,
        DomainListDto,
        DeploymentDto,
        DeploymentKindDto,
        DeploymentLogDto,
        DeploymentStatusDto,
        ApplicationDto,
        BuildKindDto,
        ComposeTrustRequest,
        CreateApplicationRequest,
        SourceDto,
        SourceRequest,
        StrategyDto,
        UpdateApplicationRequest,
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

fn reason(status: &str) -> &'static str {
    status
        .parse::<u16>()
        .ok()
        .and_then(|code| StatusCode::from_u16(code).ok())
        .and_then(|code| code.canonical_reason())
        .unwrap_or("Response")
}

fn describe_responses(document: &mut utoipa::openapi::OpenApi) {
    for item in document.paths.paths.values_mut() {
        let operations = [
            &mut item.get,
            &mut item.put,
            &mut item.post,
            &mut item.delete,
            &mut item.patch,
            &mut item.head,
            &mut item.options,
        ];
        for operation in operations.into_iter().flatten() {
            for (status, response) in &mut operation.responses.responses {
                if let RefOr::T(response) = response
                    && response.description.is_empty()
                {
                    reason(status).clone_into(&mut response.description);
                }
            }
        }
    }
}

fn document() -> utoipa::openapi::OpenApi {
    let mut document = ApiDoc::openapi()
        .merge_from(DatabaseApi::openapi())
        .merge_from(ScheduledBackupApi::openapi());
    describe_responses(&mut document);
    document
}

pub fn openapi_json() -> Result<String, serde_json::Error> {
    document().to_pretty_json()
}

pub async fn serve() -> Json<utoipa::openapi::OpenApi> {
    Json(document())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_response_has_a_description() {
        let document = document();
        for (path, item) in &document.paths.paths {
            let operations = [&item.get, &item.put, &item.post, &item.delete, &item.patch];
            for operation in operations.into_iter().flatten() {
                for (status, response) in &operation.responses.responses {
                    if let RefOr::T(response) = response {
                        assert!(!response.description.is_empty(), "{path} {status}");
                    }
                }
            }
        }
        assert_eq!(reason("404"), "Not Found");
        assert_eq!(reason("101"), "Switching Protocols");
    }
}
