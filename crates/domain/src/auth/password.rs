use argon2::Argon2;
use argon2::password_hash::phc::PasswordHash;
use argon2::password_hash::{PasswordHasher, PasswordVerifier};
use ferrodock_core::{AppError, AppResult};
use zeroize::Zeroizing;

const MIN_LENGTH: usize = 12;
const MAX_LENGTH: usize = 128;

pub fn validate_strength(password: &str) -> AppResult<()> {
    let length = password.chars().count();
    if length < MIN_LENGTH {
        return Err(AppError::validation("PASSWORD_TOO_SHORT").with_param("min", MIN_LENGTH));
    }
    if length > MAX_LENGTH {
        return Err(AppError::validation("PASSWORD_TOO_LONG").with_param("max", MAX_LENGTH));
    }
    Ok(())
}

pub async fn hash(password: Zeroizing<String>) -> AppResult<String> {
    tokio::task::spawn_blocking(move || {
        Argon2::default()
            .hash_password(password.as_bytes())
            .map(|hash| hash.to_string())
            .map_err(|error| AppError::internal(error.to_string()))
    })
    .await
    .map_err(AppError::internal)?
}

pub async fn verify(password: Zeroizing<String>, encoded_hash: String) -> AppResult<bool> {
    tokio::task::spawn_blocking(move || {
        let parsed = PasswordHash::new(&encoded_hash)
            .map_err(|error| AppError::internal(error.to_string()))?;
        Ok(Argon2::default()
            .verify_password(password.as_bytes(), &parsed)
            .is_ok())
    })
    .await
    .map_err(AppError::internal)?
}

const DUMMY_HASH: &str = "$argon2id$v=19$m=19456,t=2,p=1$c29tZXNhbHRzb21lc2FsdA$\
    Vh4D0pbdbVSTkzW1rOJqqfY6AqOxwi7HFk7m9tqkVb8";

pub async fn dummy_verify(password: Zeroizing<String>) {
    let _ = verify(password, DUMMY_HASH.to_owned()).await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn hashes_and_verifies() -> AppResult<()> {
        let encoded = hash(Zeroizing::new("correct horse battery".to_owned())).await?;
        assert!(
            verify(
                Zeroizing::new("correct horse battery".to_owned()),
                encoded.clone()
            )
            .await?
        );
        assert!(!verify(Zeroizing::new("wrong password!!".to_owned()), encoded).await?);
        Ok(())
    }

    #[tokio::test]
    async fn dummy_hash_is_parseable() -> AppResult<()> {
        assert!(!verify(Zeroizing::new("anything".to_owned()), DUMMY_HASH.to_owned()).await?);
        Ok(())
    }

    #[test]
    fn enforces_length_policy() {
        assert!(validate_strength("short").is_err());
        assert!(validate_strength("long enough password").is_ok());
        assert!(validate_strength(&"x".repeat(129)).is_err());
    }
}
