use std::collections::HashSet;

use ferrodock_core::{AppError, AppResult};

pub const MAX_VARIABLES: usize = 500;
const MAX_KEY_LEN: usize = 128;
const MAX_VALUE_LEN: usize = 32 * 1024;

pub fn validate_key(key: &str) -> AppResult<()> {
    let valid = !key.is_empty()
        && key.len() <= MAX_KEY_LEN
        && key.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_')
        && key.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
    if valid {
        Ok(())
    } else {
        Err(AppError::validation("VARIABLE_KEY_INVALID").with_param("key", key))
    }
}

pub fn validate_value(key: &str, value: &str) -> AppResult<()> {
    if value.len() > MAX_VALUE_LEN || value.contains('\0') {
        return Err(AppError::validation("VARIABLE_VALUE_INVALID")
            .with_param("key", key)
            .with_param("max", MAX_VALUE_LEN));
    }
    Ok(())
}

pub fn validate_keys<'a>(keys: impl Iterator<Item = &'a str>) -> AppResult<()> {
    let mut seen = HashSet::new();
    for key in keys {
        validate_key(key)?;
        if !seen.insert(key) {
            return Err(AppError::validation("VARIABLE_KEY_DUPLICATE").with_param("key", key));
        }
    }
    if seen.len() > MAX_VARIABLES {
        return Err(AppError::validation("VARIABLE_LIMIT").with_param("max", MAX_VARIABLES));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_posix_style_keys_only() {
        for key in ["DATABASE_URL", "_private", "a1"] {
            assert!(validate_key(key).is_ok(), "{key}");
        }
        for key in ["", "1ABC", "MY-VAR", "A B", "ŞEHİR", "X=Y"] {
            assert!(validate_key(key).is_err(), "{key}");
        }
    }

    #[test]
    fn rejects_duplicates_and_bad_values() {
        assert!(validate_keys(["A", "B", "A"].into_iter()).is_err());
        assert!(validate_value("A", "line\nbreak is fine").is_ok());
        assert!(validate_value("A", "nul\0byte").is_err());
        assert!(validate_value("A", &"x".repeat(40_000)).is_err());
    }
}
