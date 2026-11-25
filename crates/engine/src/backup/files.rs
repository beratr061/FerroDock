use std::path::{Path, PathBuf};

use ferrodock_core::archive::Entry;
use ferrodock_core::{AppError, AppResult};

const PROXY_PREFIX: &str = "proxy/";
const KNOWN_HOSTS_ENTRY: &str = "ssh/known_hosts";

fn known_hosts(data_dir: &Path) -> PathBuf {
    data_dir.join("ssh").join("known_hosts")
}

fn is_route_file(name: &str) -> bool {
    name.starts_with("app-")
        && !name.contains('/')
        && Path::new(name)
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("yml"))
}

fn io(error: std::io::Error) -> AppError {
    AppError::internal(error)
}

async fn proxy_entries(dir: &Path) -> AppResult<Vec<Entry>> {
    let mut entries = Vec::new();
    let mut listing = match tokio::fs::read_dir(dir).await {
        Ok(listing) => listing,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(entries),
        Err(error) => return Err(io(error)),
    };
    while let Some(item) = listing.next_entry().await.map_err(io)? {
        let name = item.file_name().to_string_lossy().into_owned();
        if is_route_file(&name) {
            let data = tokio::fs::read(item.path()).await.map_err(io)?;
            entries.push(Entry {
                name: format!("{PROXY_PREFIX}{name}"),
                data,
            });
        }
    }
    Ok(entries)
}

pub async fn collect(proxy_dir: Option<&Path>, data_dir: &Path) -> AppResult<Vec<Entry>> {
    let mut entries = match proxy_dir {
        Some(dir) => proxy_entries(dir).await?,
        None => Vec::new(),
    };
    if let Ok(data) = tokio::fs::read(known_hosts(data_dir)).await {
        entries.push(Entry {
            name: KNOWN_HOSTS_ENTRY.to_owned(),
            data,
        });
    }
    Ok(entries)
}

fn destination(entry: &Entry, proxy_dir: Option<&Path>, data_dir: &Path) -> Option<PathBuf> {
    if entry.name == KNOWN_HOSTS_ENTRY {
        return Some(known_hosts(data_dir));
    }
    let file = entry.name.strip_prefix(PROXY_PREFIX)?;
    is_route_file(file)
        .then(|| proxy_dir.map(|dir| dir.join(file)))
        .flatten()
}

pub async fn restore(
    entries: &[Entry],
    proxy_dir: Option<&Path>,
    data_dir: &Path,
) -> AppResult<usize> {
    let mut written = 0;
    for entry in entries {
        let Some(path) = destination(entry, proxy_dir, data_dir) else {
            continue;
        };
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await.map_err(io)?;
        }
        tokio::fs::write(&path, &entry.data).await.map_err(io)?;
        written += 1;
    }
    Ok(written)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(name: &str) -> Entry {
        Entry {
            name: name.to_owned(),
            data: Vec::new(),
        }
    }

    #[test]
    fn maps_only_known_entries() {
        let data = Path::new("/data");
        let proxy = Path::new("/proxy");
        assert_eq!(
            destination(&entry("proxy/app-abc.yml"), Some(proxy), data),
            Some(proxy.join("app-abc.yml"))
        );
        assert_eq!(
            destination(&entry("proxy/panel.yml"), Some(proxy), data),
            None
        );
        assert_eq!(destination(&entry("proxy/app-abc.yml"), None, data), None);
        assert_eq!(
            destination(&entry("ssh/known_hosts"), None, data),
            Some(known_hosts(data))
        );
        assert_eq!(destination(&entry("other/file"), Some(proxy), data), None);
    }
}
