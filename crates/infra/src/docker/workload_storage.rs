use std::collections::HashMap;

use bollard::models::{HostConfig, Mount, MountType, PortBinding, VolumeCreateRequest};
use ferrodock_core::AppResult;

use super::{DockerEngine, MANAGED_LABEL, docker_error};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MountSource {
    Volume,
    Bind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkloadMount {
    pub kind: MountSource,
    pub source: String,
    pub target: String,
    pub read_only: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkloadPort {
    pub published: u16,
    pub target: u16,
    pub protocol: String,
}

fn mount(entry: &WorkloadMount) -> Mount {
    let typ = match entry.kind {
        MountSource::Volume => MountType::VOLUME,
        MountSource::Bind => MountType::BIND,
    };
    Mount {
        target: Some(entry.target.clone()),
        source: Some(entry.source.clone()),
        typ: Some(typ),
        read_only: Some(entry.read_only),
        ..Mount::default()
    }
}

fn container_port(port: &WorkloadPort) -> String {
    format!("{}/{}", port.target, port.protocol)
}

pub fn exposed_ports(ports: &[WorkloadPort]) -> Option<Vec<String>> {
    (!ports.is_empty()).then(|| ports.iter().map(container_port).collect())
}

pub fn with_storage(
    host: HostConfig,
    mounts: &[WorkloadMount],
    ports: &[WorkloadPort],
) -> HostConfig {
    let mut bindings: HashMap<String, Option<Vec<PortBinding>>> = HashMap::new();
    for port in ports {
        let binding = PortBinding {
            host_ip: None,
            host_port: Some(port.published.to_string()),
        };
        bindings
            .entry(container_port(port))
            .or_insert_with(|| Some(Vec::new()))
            .get_or_insert_with(Vec::new)
            .push(binding);
    }
    HostConfig {
        mounts: (!mounts.is_empty()).then(|| mounts.iter().map(mount).collect()),
        port_bindings: (!bindings.is_empty()).then_some(bindings),
        ..host
    }
}

impl DockerEngine {
    pub async fn ensure_volume(&self, name: &str, owner: (&str, &str)) -> AppResult<()> {
        let (owner_label, owner_id) = owner;
        let labels = HashMap::from([
            (MANAGED_LABEL.to_owned(), "true".to_owned()),
            (owner_label.to_owned(), owner_id.to_owned()),
        ]);
        let request = VolumeCreateRequest {
            name: Some(name.to_owned()),
            labels: Some(labels),
            ..VolumeCreateRequest::default()
        };
        self.client
            .create_volume(request)
            .await
            .map(|_| ())
            .map_err(docker_error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_mounts_and_published_ports() {
        let mounts = [WorkloadMount {
            kind: MountSource::Volume,
            source: "fd-abc-data".to_owned(),
            target: "/data".to_owned(),
            read_only: false,
        }];
        let ports = [WorkloadPort {
            published: 5432,
            target: 5432,
            protocol: "tcp".to_owned(),
        }];
        let host = with_storage(HostConfig::default(), &mounts, &ports);
        let mapped = host.mounts.unwrap_or_default();
        assert_eq!(mapped[0].typ, Some(MountType::VOLUME));
        let bindings = host.port_bindings.unwrap_or_default();
        let published = bindings["5432/tcp"].clone().unwrap_or_default();
        assert_eq!(published[0].host_port.as_deref(), Some("5432"));
        assert_eq!(exposed_ports(&ports), Some(vec!["5432/tcp".to_owned()]));
        assert!(
            with_storage(HostConfig::default(), &[], &[])
                .mounts
                .is_none()
        );
    }
}
