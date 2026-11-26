use ferrodock_core::{AppError, AppResult};

const MAX_PATH_CHARS: usize = 255;
const FORBIDDEN_HOST_PATHS: [&str; 10] = [
    "/proc",
    "/sys",
    "/dev",
    "/boot",
    "/etc/ferrodock",
    "/var/lib/docker",
    "/var/lib/containerd",
    "/var/run/docker.sock",
    "/run/docker.sock",
    "/run/containerd",
];

fn normalized(path: &str) -> Option<String> {
    let trimmed = path.trim();
    let well_formed = trimmed.starts_with('/')
        && trimmed.chars().count() <= MAX_PATH_CHARS
        && !trimmed.chars().any(char::is_control)
        && !trimmed.split('/').any(|part| part == ".." || part == ".");
    if !well_formed {
        return None;
    }
    let joined = trimmed
        .split('/')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("/");
    Some(format!("/{joined}"))
}

fn within(path: &str, root: &str) -> bool {
    path == root || path.starts_with(&format!("{root}/"))
}

pub fn container_path(path: &str) -> AppResult<String> {
    normalized(path).filter(|path| path != "/").ok_or_else(|| {
        AppError::validation("RUNTIME_MOUNT_TARGET_INVALID").with_param("path", path.trim())
    })
}

pub fn host_path(path: &str) -> AppResult<String> {
    let path = normalized(path).ok_or_else(|| {
        AppError::validation("RUNTIME_BIND_PATH_INVALID").with_param("path", path.trim())
    })?;
    let forbidden = path == "/" || FORBIDDEN_HOST_PATHS.iter().any(|root| within(&path, root));
    if forbidden {
        return Err(AppError::validation("RUNTIME_BIND_PATH_FORBIDDEN").with_param("path", path));
    }
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_container_paths() {
        assert_eq!(
            container_path(" /var//data/ ").ok().as_deref(),
            Some("/var/data")
        );
        for bad in ["/", "data", "/a/../b", "/a/./b", "/a\nb"] {
            assert!(container_path(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn keeps_host_binds_away_from_the_engine_and_panel() {
        assert_eq!(host_path("/srv/app").ok().as_deref(), Some("/srv/app"));
        assert!(host_path("/var/run/docker.sock").is_err());
        assert!(host_path("/etc/ferrodock/panel").is_err());
        assert!(host_path("//").is_err());
        assert!(host_path("/var/lib/dockerish").is_ok());
    }
}
