use ferrodock_core::{AppError, AppResult};
use zeroize::Zeroizing;

const MAX_EMAIL_LEN: usize = 254;
const SUPPORTED_LOCALES: [&str; 2] = ["en", "tr"];

#[derive(Debug)]
pub struct SetupInput {
    pub email: String,
    pub name: String,
    pub password: Zeroizing<String>,
    pub organization_name: String,
    pub locale: String,
    pub setup_token: Option<Zeroizing<String>>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

#[derive(Debug)]
pub struct SignInInput {
    pub email: String,
    pub password: Zeroizing<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

pub fn normalize_email(email: &str) -> AppResult<String> {
    let normalized = email.trim().to_lowercase();
    let valid = normalized.len() <= MAX_EMAIL_LEN
        && normalized
            .split_once('@')
            .is_some_and(|(local, domain)| !local.is_empty() && domain.contains('.'))
        && !normalized.chars().any(char::is_whitespace);
    if valid {
        Ok(normalized)
    } else {
        Err(AppError::validation("EMAIL_INVALID"))
    }
}

pub fn clean_locale(locale: &str) -> String {
    let candidate = locale.trim().to_lowercase();
    if SUPPORTED_LOCALES.contains(&candidate.as_str()) {
        candidate
    } else {
        "en".to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_and_validates_email() {
        assert_eq!(
            normalize_email("  Ada@Example.COM ").ok(),
            Some("ada@example.com".to_owned())
        );
        assert!(normalize_email("no-at-sign").is_err());
        assert!(normalize_email("a b@example.com").is_err());
    }

    #[test]
    fn falls_back_to_supported_locale() {
        assert_eq!(clean_locale("TR"), "tr");
        assert_eq!(clean_locale("xx"), "en");
    }
}
