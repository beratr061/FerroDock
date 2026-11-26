use std::collections::HashSet;
use std::hash::Hash;

use ferrodock_core::{AppError, AppResult};
use ferrodock_db::runtime_config::{MountRecord, PortRecord, RuntimeConfigRecord};

use super::paths::{container_path, host_path};

pub const MAX_ENTRIES: usize = 20;
const MAX_VOLUME_NAME_CHARS: usize = 63;
const PROXY_PORTS: [u16; 2] = [80, 443];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MountKind {
    Volume,
    Bind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Protocol {
    Tcp,
    Udp,
}

#[derive(Debug, Clone)]
pub struct MountInput {
    pub kind: MountKind,
    pub source: String,
    pub target: String,
    pub read_only: bool,
}

#[derive(Debug, Clone)]
pub struct PortInput {
    pub published: u16,
    pub target: u16,
    pub protocol: Protocol,
}

#[derive(Debug, Clone, Default)]
pub struct RuntimeConfigInput {
    pub mounts: Vec<MountInput>,
    pub ports: Vec<PortInput>,
}

fn volume_name(name: &str) -> AppResult<String> {
    let name = name.trim();
    let valid = name.len() <= MAX_VOLUME_NAME_CHARS
        && name.starts_with(|c: char| c.is_ascii_lowercase() || c.is_ascii_digit())
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '-' | '_' | '.'));
    if valid {
        Ok(name.to_owned())
    } else {
        Err(AppError::validation("RUNTIME_VOLUME_NAME_INVALID").with_param("name", name))
    }
}

fn mount(input: &MountInput) -> AppResult<MountRecord> {
    let (kind, source) = match input.kind {
        MountKind::Volume => ("volume", volume_name(&input.source)?),
        MountKind::Bind => ("bind", host_path(&input.source)?),
    };
    Ok(MountRecord {
        kind: kind.to_owned(),
        source,
        target: container_path(&input.target)?,
        read_only: input.read_only,
    })
}

fn port(input: &PortInput) -> AppResult<PortRecord> {
    if input.published == 0 || input.target == 0 {
        return Err(AppError::validation("RUNTIME_PORT_INVALID"));
    }
    if PROXY_PORTS.contains(&input.published) {
        return Err(
            AppError::validation("RUNTIME_PORT_RESERVED").with_param("port", input.published)
        );
    }
    let protocol = match input.protocol {
        Protocol::Tcp => "tcp",
        Protocol::Udp => "udp",
    };
    Ok(PortRecord {
        published: input.published,
        target: input.target,
        protocol: protocol.to_owned(),
    })
}

fn unique<T: Eq + Hash>(keys: impl Iterator<Item = T>, code: &'static str) -> AppResult<()> {
    let mut seen = HashSet::new();
    for key in keys {
        if !seen.insert(key) {
            return Err(AppError::validation(code));
        }
    }
    Ok(())
}

pub fn validate(input: &RuntimeConfigInput) -> AppResult<RuntimeConfigRecord> {
    if input.mounts.len() > MAX_ENTRIES || input.ports.len() > MAX_ENTRIES {
        return Err(AppError::validation("RUNTIME_TOO_MANY").with_param("max", MAX_ENTRIES));
    }
    let mounts = input
        .mounts
        .iter()
        .map(mount)
        .collect::<AppResult<Vec<_>>>()?;
    let ports = input
        .ports
        .iter()
        .map(port)
        .collect::<AppResult<Vec<_>>>()?;
    unique(
        mounts.iter().map(|m| m.target.as_str()),
        "RUNTIME_MOUNT_DUPLICATE",
    )?;
    unique(
        ports.iter().map(|p| (p.published, p.protocol.as_str())),
        "RUNTIME_PORT_DUPLICATE",
    )?;
    Ok(RuntimeConfigRecord { mounts, ports })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn volume(source: &str, target: &str) -> MountInput {
        MountInput {
            kind: MountKind::Volume,
            source: source.to_owned(),
            target: target.to_owned(),
            read_only: false,
        }
    }

    fn tcp(published: u16, target: u16) -> PortInput {
        PortInput {
            published,
            target,
            protocol: Protocol::Tcp,
        }
    }

    fn rejection(mounts: Vec<MountInput>, ports: Vec<PortInput>) -> Option<String> {
        validate(&RuntimeConfigInput { mounts, ports })
            .err()
            .map(|error| error.code().to_owned())
    }

    #[test]
    fn accepts_volumes_and_ports() -> AppResult<()> {
        let input = RuntimeConfigInput {
            mounts: vec![volume("data", "/var/lib/data/")],
            ports: vec![tcp(5432, 5432)],
        };
        let config = validate(&input)?;
        assert_eq!(config.mounts[0].target, "/var/lib/data");
        assert_eq!(config.ports[0].protocol, "tcp");
        Ok(())
    }

    #[test]
    fn rejects_bad_names_reserved_ports_and_duplicates() {
        let bad_name = rejection(vec![volume("Data", "/d")], vec![]);
        assert_eq!(bad_name.as_deref(), Some("RUNTIME_VOLUME_NAME_INVALID"));
        let reserved = rejection(vec![], vec![tcp(443, 8443)]);
        assert_eq!(reserved.as_deref(), Some("RUNTIME_PORT_RESERVED"));
        let twice = rejection(vec![volume("a", "/d"), volume("b", "/d/")], vec![]);
        assert_eq!(twice.as_deref(), Some("RUNTIME_MOUNT_DUPLICATE"));
        let ports = rejection(vec![], vec![tcp(9000, 1), tcp(9000, 2)]);
        assert_eq!(ports.as_deref(), Some("RUNTIME_PORT_DUPLICATE"));
    }
}
