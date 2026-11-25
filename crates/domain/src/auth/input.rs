use ferrodock_core::{AppError, AppResult};
use zeroize::Zeroizing;

const MAX_EMAIL_LEN: usize = 254;
const MAX_NAME_LEN: usize = 100;
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

pub fn clean_name(value: &str, code: &'static str) -> AppResult<String> {
    let trimmed = value.trim();
    let length = trimmed.chars().count();
    if length == 0 || length > MAX_NAME_LEN || trimmed.chars().any(char::is_control) {
        return Err(AppError::validation(code).with_param("max", MAX_NAME_LEN));
    }
    Ok(trimmed.to_owned())
}

pub fn clean_locale(locale: &str) -> String {
    let candidate = locale.trim().to_lowercase();
    if SUPPORTED_LOCALES.contains(&candidate.as_str()) {
        candidate
    } else {
        "en".to_owned()
    }
}

pub fn slugify(name: &str) -> String {
    let mut slug = String::with_capacity(name.len());
    for character in name.to_lowercase().chars() {
        if character.is_ascii_alphanumeric() {
            slug.push(character);
        } else if !slug.ends_with('-') && !slug.is_empty() {
            slug.push('-');
        }
    }
    let trimmed = slug.trim_end_matches('-');
    if trimmed.is_empty() {
        "default".to_owned()
    } else {
        trimmed.to_owned()
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
    fn builds_safe_slugs() {
        assert_eq!(slugify("Acme Platform Team!"), "acme-platform-team");
        assert_eq!(slugify("Çalışma Grubu"), "al-ma-grubu");
        assert_eq!(slugify("!!!"), "default");
    }

    #[test]
    fn rejects_bad_names_and_locales() {
        assert!(clean_name("   ", "NAME_INVALID").is_err());
        assert!(clean_name("ok\u{0007}", "NAME_INVALID").is_err());
        assert_eq!(clean_locale("TR"), "tr");
        assert_eq!(clean_locale("xx"), "en");
    }
}
