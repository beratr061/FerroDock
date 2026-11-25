use std::net::IpAddr;

use ferrodock_core::{AppError, AppResult};

const MAX_HOST_LEN: usize = 253;
const MAX_LABEL_LEN: usize = 63;
const MAX_PATH_LEN: usize = 200;

fn valid_label(label: &str) -> bool {
    !label.is_empty()
        && label.len() <= MAX_LABEL_LEN
        && !label.starts_with('-')
        && !label.ends_with('-')
        && label.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
}

pub fn normalize_host(input: &str) -> AppResult<String> {
    let invalid = || AppError::validation("DOMAIN_HOST_INVALID");
    let trimmed = input.trim().trim_end_matches('.');
    if trimmed.parse::<IpAddr>().is_ok() {
        return Err(AppError::validation("DOMAIN_HOST_IP"));
    }
    let ascii = idna::domain_to_ascii_strict(trimmed).map_err(|_| invalid())?;
    let labels: Vec<&str> = ascii.split('.').collect();
    let valid = ascii.len() <= MAX_HOST_LEN
        && labels.len() >= 2
        && labels.iter().all(|label| valid_label(label))
        && labels
            .last()
            .is_some_and(|tld| tld.chars().any(|c| c.is_ascii_alphabetic()));
    if valid { Ok(ascii) } else { Err(invalid()) }
}

pub fn normalize_path_prefix(input: Option<&str>) -> AppResult<String> {
    let raw = input
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("/");
    let path = if raw.len() > 1 {
        raw.trim_end_matches('/')
    } else {
        raw
    };
    let valid = path.starts_with('/')
        && path.len() <= MAX_PATH_LEN
        && !path.contains("//")
        && !path
            .split('/')
            .any(|segment| segment == "." || segment == "..")
        && path
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '/' | '-' | '_' | '.' | '~'));
    if valid {
        Ok(path.to_owned())
    } else {
        Err(AppError::validation("DOMAIN_PATH_INVALID"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_hosts_to_ascii() {
        assert_eq!(
            normalize_host(" App.Example.COM. ").ok().as_deref(),
            Some("app.example.com")
        );
        assert_eq!(
            normalize_host("çiçek.com.tr").ok().as_deref(),
            Some("xn--iek-1lab.com.tr")
        );
        for bad in [
            "localhost",
            "a b.com",
            "-x.com",
            "x.com`) || Host(`y.com",
            "*.x.com",
            "x.123",
        ] {
            assert!(normalize_host(bad).is_err(), "{bad}");
        }
        assert_eq!(
            normalize_host("10.0.0.1").err().map(|e| e.code()),
            Some("DOMAIN_HOST_IP")
        );
    }

    #[test]
    fn normalizes_path_prefixes() {
        assert_eq!(normalize_path_prefix(None).ok().as_deref(), Some("/"));
        assert_eq!(
            normalize_path_prefix(Some("/api/")).ok().as_deref(),
            Some("/api")
        );
        for bad in ["api", "/a/../b", "/a//b", "/a`b", "/a b"] {
            assert!(normalize_path_prefix(Some(bad)).is_err(), "{bad}");
        }
    }
}
