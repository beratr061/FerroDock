use ferrodock_core::{AppError, AppResult};

use super::model::S3Input;

const MAX_PREFIX_LEN: usize = 100;
const MAX_FIELD_LEN: usize = 200;
pub const MAX_RETENTION: u16 = 365;

pub fn path_prefix(value: Option<&str>) -> AppResult<String> {
    let trimmed = value.unwrap_or_default().trim().trim_matches('/');
    let valid = trimmed.len() <= MAX_PREFIX_LEN
        && trimmed
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '/' | '.'))
        && trimmed.split('/').all(|part| !matches!(part, "." | ".."))
        && !trimmed.contains("//");
    if valid {
        Ok(trimmed.to_owned())
    } else {
        Err(AppError::validation("BACKUP_PREFIX_INVALID").with_param("max", MAX_PREFIX_LEN))
    }
}

fn plain(value: &str, code: &'static str) -> AppResult<String> {
    let trimmed = value.trim();
    let valid = !trimmed.is_empty()
        && trimmed.len() <= MAX_FIELD_LEN
        && !trimmed.chars().any(|c| c.is_control() || c.is_whitespace());
    if valid {
        Ok(trimmed.to_owned())
    } else {
        Err(AppError::validation(code))
    }
}

fn bucket(value: &str) -> AppResult<String> {
    let trimmed = value.trim();
    let valid = (3..=63).contains(&trimmed.len())
        && trimmed
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '.' | '-'))
        && trimmed.starts_with(|c: char| c.is_ascii_alphanumeric());
    if valid {
        Ok(trimmed.to_owned())
    } else {
        Err(AppError::validation("BACKUP_BUCKET_INVALID"))
    }
}

fn endpoint(value: Option<&str>) -> AppResult<Option<String>> {
    let Some(raw) = value.map(str::trim).filter(|v| !v.is_empty()) else {
        return Ok(None);
    };
    let url = plain(raw, "BACKUP_ENDPOINT_INVALID")?;
    let host = url
        .strip_prefix("https://")
        .or_else(|| url.strip_prefix("http://"))
        .filter(|rest| !rest.is_empty() && !rest.contains(['@', '?', '#']));
    match host {
        Some(_) => Ok(Some(url.trim_end_matches('/').to_owned())),
        None => Err(AppError::validation("BACKUP_ENDPOINT_INVALID")),
    }
}

pub fn s3(input: &S3Input) -> AppResult<S3Input> {
    if input.secret_access_key.trim().is_empty() {
        return Err(AppError::validation("BACKUP_CREDENTIALS_INVALID"));
    }
    Ok(S3Input {
        endpoint: endpoint(input.endpoint.as_deref())?,
        region: plain(&input.region, "BACKUP_REGION_INVALID")?,
        bucket: bucket(&input.bucket)?,
        access_key_id: plain(&input.access_key_id, "BACKUP_CREDENTIALS_INVALID")?,
        secret_access_key: input.secret_access_key.clone(),
        path_style: input.path_style,
    })
}

pub fn retention(value: u16) -> AppResult<u16> {
    if (1..=MAX_RETENTION).contains(&value) {
        Ok(value)
    } else {
        Err(AppError::validation("BACKUP_RETENTION_INVALID").with_param("max", MAX_RETENTION))
    }
}

#[cfg(test)]
mod tests {
    use zeroize::Zeroizing;

    use super::*;

    fn input(endpoint: Option<&str>, bucket: &str) -> S3Input {
        S3Input {
            endpoint: endpoint.map(str::to_owned),
            region: "eu-central-1".to_owned(),
            bucket: bucket.to_owned(),
            access_key_id: "AKIA".to_owned(),
            secret_access_key: Zeroizing::new("secret".to_owned()),
            path_style: false,
        }
    }

    #[test]
    fn checks_destination_fields() {
        assert_eq!(
            path_prefix(Some("/team/db/")).ok().as_deref(),
            Some("team/db")
        );
        assert!(path_prefix(Some("a/../b")).is_err());
        assert!(path_prefix(Some("a b")).is_err());
        assert!(s3(&input(Some("http://minio:9000/"), "backups")).is_ok());
        assert!(s3(&input(Some("ftp://host"), "backups")).is_err());
        assert!(s3(&input(Some("https://user@host"), "backups")).is_err());
        assert!(s3(&input(None, "Bad_Bucket")).is_err());
        assert!(retention(0).is_err());
        assert!(retention(30).is_ok());
    }
}
