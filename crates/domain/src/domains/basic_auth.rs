use ferrodock_core::{AppError, AppResult};
use zeroize::Zeroizing;

const MAX_USERNAME_CHARS: usize = 64;
const MIN_PASSWORD_CHARS: usize = 8;
const MAX_PASSWORD_BYTES: usize = 72;
const HASH_COST: u32 = 11;

#[derive(Debug, Clone)]
pub struct BasicAuthInput {
    pub username: String,
    pub password: Zeroizing<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BasicAuthCredential {
    pub username: String,
    pub hash: String,
}

fn valid_username(username: &str) -> bool {
    !username.is_empty()
        && username.chars().count() <= MAX_USERNAME_CHARS
        && !username
            .chars()
            .any(|c| c == ':' || c.is_control() || c.is_whitespace())
}

fn valid_password(password: &str) -> bool {
    password.chars().count() >= MIN_PASSWORD_CHARS && password.len() <= MAX_PASSWORD_BYTES
}

pub async fn hash_credential(
    input: Option<BasicAuthInput>,
) -> AppResult<Option<BasicAuthCredential>> {
    let Some(input) = input else { return Ok(None) };
    let username = input.username.trim().to_owned();
    if !valid_username(&username) {
        return Err(AppError::validation("DOMAIN_AUTH_USER_INVALID"));
    }
    if !valid_password(&input.password) {
        return Err(AppError::validation("DOMAIN_AUTH_PASSWORD_INVALID"));
    }
    let password = input.password;
    let hash = tokio::task::spawn_blocking(move || bcrypt::hash(password.as_bytes(), HASH_COST))
        .await
        .map_err(AppError::internal)?
        .map_err(AppError::internal)?;
    Ok(Some(BasicAuthCredential { username, hash }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(username: &str, password: &str) -> BasicAuthInput {
        BasicAuthInput {
            username: username.to_owned(),
            password: Zeroizing::new(password.to_owned()),
        }
    }

    #[tokio::test]
    async fn hashes_valid_credentials_with_bcrypt() -> AppResult<()> {
        let credential = hash_credential(Some(input(" ops ", "long-enough-secret"))).await?;
        let credential = credential.ok_or_else(|| AppError::validation("MISSING"))?;
        assert_eq!(credential.username, "ops");
        assert!(credential.hash.starts_with("$2"));
        assert!(bcrypt::verify("long-enough-secret", &credential.hash).unwrap_or(false));
        assert_eq!(hash_credential(None).await?, None);
        Ok(())
    }

    #[tokio::test]
    async fn rejects_unsafe_usernames_and_short_passwords() {
        for (username, password) in [
            ("a:b", "long-enough"),
            ("", "long-enough"),
            ("ops", "short"),
        ] {
            assert!(
                hash_credential(Some(input(username, password)))
                    .await
                    .is_err()
            );
        }
        assert!(
            hash_credential(Some(input("ops", &"x".repeat(73))))
                .await
                .is_err()
        );
    }
}
