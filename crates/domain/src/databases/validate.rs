use ferrodock_core::crypto::random_bytes;
use ferrodock_core::{AppError, AppResult};

use ferrodock_db::databases::ResourceLimits;

use super::drivers::DatabaseDriver;

const MAX_IDENTIFIER_LEN: usize = 63;
const PASSWORD_LEN: usize = 32;
const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
const DEFAULT_USERNAME: &str = "app";
const MIN_EXTERNAL_PORT: u16 = 1024;
const MEMORY_MB: std::ops::RangeInclusive<u32> = 64..=1_048_576;
const CPU_MILLIS: std::ops::RangeInclusive<u32> = 50..=256_000;

fn is_identifier(value: &str) -> bool {
    let mut characters = value.chars();
    let starts_well = characters
        .next()
        .is_some_and(|first| first.is_ascii_lowercase() || first == '_');
    starts_well
        && value.len() <= MAX_IDENTIFIER_LEN
        && characters.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

fn identifier(value: &str, code: &'static str) -> AppResult<String> {
    let trimmed = value.trim();
    if is_identifier(trimmed) {
        Ok(trimmed.to_owned())
    } else {
        Err(AppError::validation(code).with_param("max", MAX_IDENTIFIER_LEN))
    }
}

pub fn database_name(input: Option<&str>, slug: &str) -> AppResult<String> {
    if let Some(value) = input.filter(|value| !value.trim().is_empty()) {
        return identifier(value, "DATABASE_IDENTIFIER_INVALID");
    }
    let underscored = slug.replace('-', "_");
    let prefixed = if underscored.starts_with(|c: char| c.is_ascii_digit()) {
        format!("db_{underscored}")
    } else {
        underscored
    };
    identifier(&prefixed, "DATABASE_IDENTIFIER_INVALID")
}

pub fn username(driver: &dyn DatabaseDriver, input: Option<&str>) -> AppResult<String> {
    if let Some(fixed) = driver.fixed_username() {
        return Ok(fixed.to_owned());
    }
    let chosen = input
        .filter(|value| !value.trim().is_empty())
        .unwrap_or(DEFAULT_USERNAME);
    let name = identifier(chosen, "DATABASE_USERNAME_INVALID")?;
    if driver.needs_root_password() && name == "root" {
        return Err(AppError::validation("DATABASE_USERNAME_INVALID"));
    }
    Ok(name)
}

pub fn version(driver: &dyn DatabaseDriver, input: Option<&str>) -> AppResult<String> {
    let versions = driver.versions();
    let chosen = input.unwrap_or(versions[0]);
    if versions.contains(&chosen) {
        Ok(chosen.to_owned())
    } else {
        Err(AppError::validation("DATABASE_VERSION_INVALID")
            .with_param("supported", versions.join(", ")))
    }
}

pub fn external_port(port: Option<u16>) -> AppResult<Option<u16>> {
    match port {
        Some(value) if value < MIN_EXTERNAL_PORT => {
            Err(AppError::validation("DATABASE_PORT_INVALID").with_param("min", MIN_EXTERNAL_PORT))
        }
        other => Ok(other),
    }
}

pub fn limits(limits: ResourceLimits) -> AppResult<ResourceLimits> {
    if limits.memory_mb.is_some_and(|mb| !MEMORY_MB.contains(&mb)) {
        return Err(AppError::validation("DATABASE_MEMORY_INVALID")
            .with_param("min", *MEMORY_MB.start())
            .with_param("max", *MEMORY_MB.end()));
    }
    if limits
        .cpu_millis
        .is_some_and(|millis| !CPU_MILLIS.contains(&millis))
    {
        return Err(AppError::validation("DATABASE_CPU_INVALID")
            .with_param("min", *CPU_MILLIS.start())
            .with_param("max", *CPU_MILLIS.end()));
    }
    Ok(limits)
}

pub fn random_password() -> String {
    let limit = u8::try_from(ALPHABET.len() * 4).unwrap_or(u8::MAX);
    let mut password = String::with_capacity(PASSWORD_LEN);
    while password.len() < PASSWORD_LEN {
        for byte in random_bytes::<64>() {
            if byte < limit && password.len() < PASSWORD_LEN {
                password.push(char::from(ALPHABET[usize::from(byte) % ALPHABET.len()]));
            }
        }
    }
    password
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::databases::drivers::driver;

    #[test]
    fn derives_and_checks_identifiers() {
        assert_eq!(
            database_name(None, "orders-db").ok().as_deref(),
            Some("orders_db")
        );
        assert_eq!(
            database_name(None, "2024-data").ok().as_deref(),
            Some("db_2024_data")
        );
        assert!(database_name(Some("Drop Table"), "x").is_err());
        assert!(database_name(Some("a;b"), "x").is_err());
    }

    #[test]
    fn checks_usernames_versions_and_ports() -> AppResult<()> {
        let mysql = driver("mysql")?;
        assert!(username(mysql, Some("root")).is_err());
        assert_eq!(username(driver("redis")?, Some("ignored"))?, "default");
        assert_eq!(username(driver("postgres")?, None)?, "app");
        assert_eq!(version(mysql, None)?, "8.4");
        assert!(version(mysql, Some("5.7")).is_err());
        assert!(external_port(Some(80)).is_err());
        assert_eq!(external_port(Some(15432))?, Some(15432));
        let too_small = ResourceLimits {
            memory_mb: Some(32),
            cpu_millis: None,
        };
        assert!(limits(too_small).is_err());
        let fine = ResourceLimits {
            memory_mb: Some(512),
            cpu_millis: Some(500),
        };
        assert_eq!(limits(fine)?, fine);
        Ok(())
    }

    #[test]
    fn passwords_are_long_and_url_safe() {
        let password = random_password();
        assert_eq!(password.len(), PASSWORD_LEN);
        assert!(password.chars().all(|c| c.is_ascii_alphanumeric()));
        assert_ne!(password, random_password());
    }
}
