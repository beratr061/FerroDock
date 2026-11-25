use ferrodock_core::{AppError, AppResult};

const MAX_IMAGE_LEN: usize = 255;
const MAX_TAG_LEN: usize = 128;
const DIGEST_HEX_LEN: usize = 64;

fn is_registry(component: &str) -> bool {
    let looks_like_host =
        component.contains('.') || component.contains(':') || component == "localhost";
    let (host, port) = component.split_once(':').unwrap_or((component, "1"));
    looks_like_host
        && host.starts_with(|c: char| c.is_ascii_alphanumeric())
        && host.ends_with(|c: char| c.is_ascii_alphanumeric())
        && host
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-'))
        && !port.is_empty()
        && port.chars().all(|c| c.is_ascii_digit())
}

fn is_path_component(component: &str) -> bool {
    let edges_valid = component.starts_with(|c: char| c.is_ascii_alphanumeric())
        && component.ends_with(|c: char| c.is_ascii_alphanumeric());
    edges_valid
        && component
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '.' | '_' | '-'))
}

fn is_tag(tag: &str) -> bool {
    tag.len() <= MAX_TAG_LEN
        && tag.starts_with(|c: char| c.is_ascii_alphanumeric() || c == '_')
        && tag
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
}

fn is_digest(digest: &str) -> bool {
    digest.strip_prefix("sha256:").is_some_and(|hex| {
        hex.len() == DIGEST_HEX_LEN
            && hex
                .chars()
                .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
    })
}

fn is_name(name: &str) -> bool {
    let mut components = name.split('/').collect::<Vec<_>>();
    if components.len() > 1 && components.first().is_some_and(|first| is_registry(first)) {
        components.remove(0);
    }
    !components.is_empty() && components.into_iter().all(is_path_component)
}

fn split_tag(reference: &str) -> (&str, Option<&str>) {
    match reference.rsplit_once(':') {
        Some((name, tag)) if !tag.contains('/') => (name, Some(tag)),
        _ => (reference, None),
    }
}

pub fn validate_image(reference: &str) -> AppResult<()> {
    let (named, digest) = match reference.split_once('@') {
        Some((named, digest)) => (named, Some(digest)),
        None => (reference, None),
    };
    let (name, tag) = split_tag(named);
    let valid = reference.len() <= MAX_IMAGE_LEN
        && is_name(name)
        && tag.is_none_or(is_tag)
        && digest.is_none_or(is_digest);
    if valid {
        Ok(())
    } else {
        Err(AppError::validation("IMAGE_INVALID"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_common_references() {
        for image in [
            "nginx",
            "nginx:1.29-alpine",
            "ghcr.io/beratr061/ferrodock:0.1.1",
            "registry.example.com:5000/team/api:latest",
            "localhost/app",
            "postgres@sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        ] {
            assert!(validate_image(image).is_ok(), "{image}");
        }
    }

    #[test]
    fn rejects_malformed_references() {
        for image in [
            "",
            "Nginx",
            "-nginx",
            "nginx:",
            "nginx:-x",
            "a b",
            "nginx@sha256:xyz",
            "../x",
        ] {
            assert!(validate_image(image).is_err(), "{image}");
        }
    }
}
