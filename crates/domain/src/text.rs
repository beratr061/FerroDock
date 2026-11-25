use ferrodock_core::{AppError, AppResult};

pub const MAX_NAME_LEN: usize = 100;
pub const MAX_DESCRIPTION_LEN: usize = 500;
const MAX_SLUG_LEN: usize = 48;

pub fn clean_name(value: &str, code: &'static str) -> AppResult<String> {
    let trimmed = value.trim();
    let length = trimmed.chars().count();
    if length == 0 || length > MAX_NAME_LEN || trimmed.chars().any(char::is_control) {
        return Err(AppError::validation(code).with_param("max", MAX_NAME_LEN));
    }
    Ok(trimmed.to_owned())
}

pub fn clean_description(value: &str) -> AppResult<String> {
    let trimmed = value.trim();
    let valid = trimmed.chars().count() <= MAX_DESCRIPTION_LEN
        && !trimmed.chars().any(|c| c.is_control() && c != '\n');
    if valid {
        Ok(trimmed.to_owned())
    } else {
        Err(AppError::validation("DESCRIPTION_INVALID").with_param("max", MAX_DESCRIPTION_LEN))
    }
}

fn transliterate(character: char) -> char {
    match character {
        'ç' | 'Ç' => 'c',
        'ğ' | 'Ğ' => 'g',
        'ı' | 'İ' => 'i',
        'ö' | 'Ö' => 'o',
        'ş' | 'Ş' => 's',
        'ü' | 'Ü' => 'u',
        other => other.to_ascii_lowercase(),
    }
}

pub fn slugify(name: &str) -> String {
    let mut slug = String::with_capacity(name.len());
    for character in name.chars().map(transliterate) {
        if character.is_ascii_alphanumeric() {
            slug.push(character);
        } else if !slug.ends_with('-') && !slug.is_empty() {
            slug.push('-');
        }
    }
    let trimmed: String = slug
        .trim_end_matches('-')
        .chars()
        .take(MAX_SLUG_LEN)
        .collect();
    let trimmed = trimmed.trim_end_matches('-');
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
    fn builds_safe_slugs() {
        assert_eq!(slugify("Acme Platform Team!"), "acme-platform-team");
        assert_eq!(slugify("Çalışma Grubu"), "calisma-grubu");
        assert_eq!(slugify("İstanbul Şubesi"), "istanbul-subesi");
        assert_eq!(slugify("!!!"), "default");
        assert_eq!(slugify(&"a".repeat(80)).len(), MAX_SLUG_LEN);
    }

    #[test]
    fn validates_names_and_descriptions() {
        assert!(clean_name("   ", "NAME_INVALID").is_err());
        assert!(clean_name("ok\u{0007}", "NAME_INVALID").is_err());
        assert_eq!(
            clean_name("  Web  ", "NAME_INVALID").ok().as_deref(),
            Some("Web")
        );
        assert!(clean_description("line one\nline two").is_ok());
        assert!(clean_description(&"x".repeat(501)).is_err());
    }
}
