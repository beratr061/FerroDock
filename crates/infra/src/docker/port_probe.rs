use std::net::{IpAddr, SocketAddr};
use std::time::Duration;

use bollard::models::ContainerInspectResponse;
use tokio::net::TcpStream;

use super::images::WORKLOAD_NETWORK;

const CONNECT_TIMEOUT: Duration = Duration::from_secs(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortState {
    Open,
    Closed,
    Unreachable,
}

fn exposed_port(details: &ContainerInspectResponse) -> Option<u16> {
    let exposed = details.config.as_ref()?.exposed_ports.as_ref()?;
    let mut ports: Vec<u16> = exposed
        .iter()
        .filter_map(|entry| entry.strip_suffix("/tcp")?.parse().ok())
        .collect();
    ports.sort_unstable();
    ports.first().copied()
}

pub fn probe_target(
    details: &ContainerInspectResponse,
    preferred: Option<u16>,
) -> Option<SocketAddr> {
    let networks = details.network_settings.as_ref()?.networks.as_ref()?;
    let address = networks.get(WORKLOAD_NETWORK)?.ip_address.as_deref()?;
    let ip: IpAddr = address.parse().ok()?;
    let port = preferred.or_else(|| exposed_port(details))?;
    Some(SocketAddr::new(ip, port))
}

pub async fn probe(target: SocketAddr) -> PortState {
    match tokio::time::timeout(CONNECT_TIMEOUT, TcpStream::connect(target)).await {
        Ok(Ok(_)) => PortState::Open,
        Ok(Err(error)) if error.kind() == std::io::ErrorKind::ConnectionRefused => {
            PortState::Closed
        }
        _ => PortState::Unreachable,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use bollard::models::{ContainerConfig, EndpointSettings, NetworkSettings};

    use super::*;

    fn details(exposed: Vec<&str>) -> ContainerInspectResponse {
        let endpoint = EndpointSettings {
            ip_address: Some("172.20.0.5".to_owned()),
            ..EndpointSettings::default()
        };
        ContainerInspectResponse {
            config: Some(ContainerConfig {
                exposed_ports: Some(exposed.into_iter().map(str::to_owned).collect()),
                ..ContainerConfig::default()
            }),
            network_settings: Some(NetworkSettings {
                networks: Some(HashMap::from([(WORKLOAD_NETWORK.to_owned(), endpoint)])),
                ..NetworkSettings::default()
            }),
            ..ContainerInspectResponse::default()
        }
    }

    #[test]
    fn prefers_domain_port_then_lowest_exposed_tcp_port() {
        let inspected = details(vec!["9000/tcp", "53/udp", "8080/tcp"]);
        assert_eq!(probe_target(&inspected, None).map(|t| t.port()), Some(8080));
        assert_eq!(
            probe_target(&inspected, Some(3000)).map(|t| t.port()),
            Some(3000)
        );
        assert_eq!(probe_target(&details(vec![]), None), None);
    }

    #[tokio::test]
    async fn tells_open_from_closed_ports() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.ok();
        let Some(listener) = listener else { return };
        let Ok(open) = listener.local_addr() else {
            return;
        };
        assert_eq!(probe(open).await, PortState::Open);
        drop(listener);
        let closed = probe(open).await;
        if cfg!(windows) {
            assert_ne!(closed, PortState::Open);
        } else {
            assert_eq!(closed, PortState::Closed);
        }
    }
}
