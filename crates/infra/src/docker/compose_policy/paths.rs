use std::path::{Component, Path, PathBuf};

use ferrodock_core::AppResult;

use super::forbidden;

fn lexical(path: &Path) -> Option<PathBuf> {
    let mut clean = PathBuf::new();
    for component in path.components() {
        match component {
            Component::ParentDir => return None,
            Component::CurDir => {}
            other => clean.push(other),
        }
    }
    Some(clean)
}

fn resolved(path: &Path) -> PathBuf {
    let mut existing = path.to_path_buf();
    let mut rest = Vec::new();
    while !existing.exists() {
        match (
            existing.file_name().map(ToOwned::to_owned),
            existing.parent(),
        ) {
            (Some(name), Some(parent)) => {
                rest.push(name);
                existing = parent.to_path_buf();
            }
            _ => break,
        }
    }
    let mut real = std::fs::canonicalize(&existing).unwrap_or(existing);
    real.extend(rest.iter().rev());
    real
}

fn inside(root: &Path, raw: &str) -> bool {
    let joined = root.join(raw);
    let Some(clean) = lexical(&joined) else {
        return false;
    };
    resolved(&clean).starts_with(root)
}

pub(super) fn check_path(
    root: &Path,
    scope: &str,
    option: &str,
    raw: Option<&str>,
) -> AppResult<()> {
    match raw {
        Some(path) if inside(root, path) => Ok(()),
        Some(_) => Err(forbidden(scope, option)),
        None => Ok(()),
    }
}
