use ferrodock_core::{AppError, AppResult};

const MAX_URL_LEN: usize = 500;
const MAX_BRANCH_LEN: usize = 200;
const MAX_PATH_LEN: usize = 255;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GitTransport {
    Https,
    Ssh,
}

fn is_host(value: &str) -> bool {
    let (host, port) = value.split_once(':').unwrap_or((value, "1"));
    !host.is_empty()
        && host.starts_with(|c: char| c.is_ascii_alphanumeric())
        && host
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-'))
        && !port.is_empty()
        && port.len() <= 5
        && port.chars().all(|c| c.is_ascii_digit())
}

fn is_repo_path(path: &str) -> bool {
    !path.is_empty()
        && !path.starts_with('-')
        && !path.split('/').any(|segment| segment == "..")
        && path
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-' | '/' | '~'))
}

fn is_user(user: &str) -> bool {
    !user.is_empty()
        && user
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
}

fn classify(url: &str) -> Option<GitTransport> {
    if let Some(rest) = url.strip_prefix("https://") {
        let (host, path) = rest.split_once('/')?;
        return (is_host(host) && is_repo_path(path)).then_some(GitTransport::Https);
    }
    if let Some(rest) = url.strip_prefix("ssh://") {
        let (authority, path) = rest.split_once('/')?;
        let (user, host) = authority.split_once('@')?;
        return (is_user(user) && is_host(host) && is_repo_path(path)).then_some(GitTransport::Ssh);
    }
    let (user, rest) = url.split_once('@')?;
    let (host, path) = rest.split_once(':')?;
    let valid = is_user(user) && is_host(host) && is_repo_path(path);
    valid.then_some(GitTransport::Ssh)
}

pub fn parse_repository_url(url: &str) -> AppResult<GitTransport> {
    if url.len() > MAX_URL_LEN {
        return Err(AppError::validation("GIT_URL_INVALID"));
    }
    classify(url).ok_or_else(|| AppError::validation("GIT_URL_INVALID"))
}

pub fn validate_branch(branch: &str) -> AppResult<()> {
    let valid = !branch.is_empty()
        && branch.len() <= MAX_BRANCH_LEN
        && !branch.starts_with(['-', '/', '.'])
        && !branch.ends_with(['/', '.'])
        && !branch
            .rsplit_once('.')
            .is_some_and(|(_, extension)| extension.eq_ignore_ascii_case("lock"))
        && !branch.contains("..")
        && !branch.contains("//")
        && branch
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-' | '/'));
    if valid {
        Ok(())
    } else {
        Err(AppError::validation("GIT_BRANCH_INVALID"))
    }
}

pub fn validate_relative_path(path: &str) -> AppResult<()> {
    let valid = !path.is_empty()
        && path.len() <= MAX_PATH_LEN
        && !path.starts_with(['/', '-'])
        && !path
            .split('/')
            .any(|segment| segment == ".." || segment.is_empty())
        && path
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-' | '/'));
    if valid {
        Ok(())
    } else {
        Err(AppError::validation("BUILD_PATH_INVALID").with_param("path", path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_supported_urls() {
        let https = parse_repository_url("https://github.com/acme/shop.git");
        assert_eq!(https.ok(), Some(GitTransport::Https));
        let scp = parse_repository_url("git@github.com:acme/shop.git");
        assert_eq!(scp.ok(), Some(GitTransport::Ssh));
        let ssh = parse_repository_url("ssh://git@git.example.com:2222/acme/shop.git");
        assert_eq!(ssh.ok(), Some(GitTransport::Ssh));
    }

    #[test]
    fn rejects_dangerous_urls() {
        for url in [
            "http://github.com/acme/shop.git",
            "https://user:token@github.com/acme/shop.git",
            "file:///etc/passwd",
            "ext::sh -c touch% /tmp/pwned",
            "-uhttps://github.com/acme/shop",
            "git@github.com:--upload-pack=x",
            "https://github.com/acme/../shop",
            "https://github.com/acme/shop extra",
        ] {
            assert!(parse_repository_url(url).is_err(), "{url}");
        }
    }

    #[test]
    fn validates_branches_and_paths() {
        assert!(validate_branch("release/2026.11").is_ok());
        assert!(validate_branch("--orphan").is_err());
        assert!(validate_branch("a..b").is_err());
        assert!(validate_relative_path("docker/Dockerfile.prod").is_ok());
        assert!(validate_relative_path(".").is_ok());
        assert!(validate_relative_path("../secrets").is_err());
        assert!(validate_relative_path("/etc/passwd").is_err());
    }
}
