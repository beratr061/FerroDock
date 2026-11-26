use ferrodock_domain::runtime_config::{
    MountInput, MountKind, PortInput, Protocol, RuntimeConfig, RuntimeConfigInput,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum MountKindDto {
    Volume,
    Bind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ProtocolDto {
    Tcp,
    Udp,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct MountDto {
    pub kind: MountKindDto,
    pub source: String,
    pub target: String,
    #[serde(default)]
    pub read_only: bool,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PortDto {
    pub published: u16,
    pub target: u16,
    pub protocol: ProtocolDto,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct RuntimeConfigDto {
    pub mounts: Vec<MountDto>,
    pub ports: Vec<PortDto>,
}

impl From<RuntimeConfig> for RuntimeConfigDto {
    fn from(config: RuntimeConfig) -> Self {
        let mounts = config
            .mounts
            .into_iter()
            .map(|mount| MountDto {
                kind: if mount.kind == "bind" {
                    MountKindDto::Bind
                } else {
                    MountKindDto::Volume
                },
                source: mount.source,
                target: mount.target,
                read_only: mount.read_only,
            })
            .collect();
        let ports = config
            .ports
            .into_iter()
            .map(|port| PortDto {
                published: port.published,
                target: port.target,
                protocol: if port.protocol == "udp" {
                    ProtocolDto::Udp
                } else {
                    ProtocolDto::Tcp
                },
            })
            .collect();
        Self { mounts, ports }
    }
}

impl From<RuntimeConfigDto> for RuntimeConfigInput {
    fn from(request: RuntimeConfigDto) -> Self {
        let mounts = request
            .mounts
            .into_iter()
            .map(|mount| MountInput {
                kind: match mount.kind {
                    MountKindDto::Volume => MountKind::Volume,
                    MountKindDto::Bind => MountKind::Bind,
                },
                source: mount.source,
                target: mount.target,
                read_only: mount.read_only,
            })
            .collect();
        let ports = request
            .ports
            .into_iter()
            .map(|port| PortInput {
                published: port.published,
                target: port.target,
                protocol: match port.protocol {
                    ProtocolDto::Tcp => Protocol::Tcp,
                    ProtocolDto::Udp => Protocol::Udp,
                },
            })
            .collect();
        Self { mounts, ports }
    }
}
