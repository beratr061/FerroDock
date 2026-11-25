use ferrodock_infra::docker::{ContainerAction, ContainerInfo, EngineInfo};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Serialize, ToSchema)]
pub struct EngineInfoDto {
    pub version: String,
    pub api_version: String,
    pub os: String,
    pub arch: String,
}

impl From<EngineInfo> for EngineInfoDto {
    fn from(info: EngineInfo) -> Self {
        Self {
            version: info.version,
            api_version: info.api_version,
            os: info.os,
            arch: info.arch,
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ContainerDto {
    pub id: String,
    pub short_id: String,
    pub name: String,
    pub image: String,
    pub state: String,
    pub status: String,
    pub created: i64,
    pub managed: bool,
    pub protected: bool,
    pub compose_project: Option<String>,
}

impl From<ContainerInfo> for ContainerDto {
    fn from(info: ContainerInfo) -> Self {
        Self {
            id: info.id,
            short_id: info.short_id,
            name: info.name,
            image: info.image,
            state: info.state,
            status: info.status,
            created: info.created,
            managed: info.managed,
            protected: info.protected,
            compose_project: info.compose_project,
        }
    }
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ContainerListQuery {
    pub all: Option<bool>,
}

#[derive(Debug, Clone, Copy, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ContainerActionDto {
    Start,
    Stop,
    Restart,
}

impl From<ContainerActionDto> for ContainerAction {
    fn from(action: ContainerActionDto) -> Self {
        match action {
            ContainerActionDto::Start => Self::Start,
            ContainerActionDto::Stop => Self::Stop,
            ContainerActionDto::Restart => Self::Restart,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ContainerActionRequest {
    pub action: ContainerActionDto,
}
