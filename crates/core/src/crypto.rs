use std::fs;
use std::io::Write;
use std::path::Path;

use aes_gcm::aead::{Aead, KeyInit, Payload};
use aes_gcm::{Aes256Gcm, Nonce};
use base64::Engine;
use base64::engine::general_purpose::STANDARD_NO_PAD;
use zeroize::Zeroizing;

use crate::{AppError, AppResult};

const KEY_LEN: usize = 32;
const NONCE_LEN: usize = 12;
const VERSION_PREFIX: &str = "v1";

pub fn random_bytes<const N: usize>() -> [u8; N] {
    let mut buffer = [0u8; N];
    rand::fill(&mut buffer);
    buffer
}

#[derive(Clone)]
pub struct SecretBox {
    cipher: Aes256Gcm,
}

impl std::fmt::Debug for SecretBox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SecretBox(***)")
    }
}

impl SecretBox {
    pub fn from_key(key: &[u8]) -> AppResult<Self> {
        let cipher = Aes256Gcm::new_from_slice(key)
            .map_err(|_| AppError::validation("MASTER_KEY_INVALID"))?;
        Ok(Self { cipher })
    }

    pub fn load(path: &Path) -> AppResult<Self> {
        Self::from_key(&read_key(path)?)
    }

    pub fn load_or_create(path: &Path) -> AppResult<Self> {
        let key = if path.exists() {
            read_key(path)?
        } else {
            create_key(path)?
        };
        Self::from_key(&key)
    }

    pub fn encrypt(&self, plaintext: &[u8]) -> AppResult<String> {
        let nonce_bytes = random_bytes::<NONCE_LEN>();
        let ciphertext = self
            .cipher
            .encrypt(&Nonce::from(nonce_bytes), plaintext)
            .map_err(|_| AppError::validation("ENCRYPTION_FAILED"))?;
        Ok(format!(
            "{VERSION_PREFIX}:{}:{}",
            STANDARD_NO_PAD.encode(nonce_bytes),
            STANDARD_NO_PAD.encode(ciphertext)
        ))
    }

    pub fn seal_bytes(&self, plaintext: &[u8]) -> AppResult<Vec<u8>> {
        let nonce_bytes = random_bytes::<NONCE_LEN>();
        let ciphertext = self
            .cipher
            .encrypt(&Nonce::from(nonce_bytes), plaintext)
            .map_err(|_| AppError::validation("ENCRYPTION_FAILED"))?;
        let mut sealed = Vec::with_capacity(NONCE_LEN + ciphertext.len());
        sealed.extend_from_slice(&nonce_bytes);
        sealed.extend_from_slice(&ciphertext);
        Ok(sealed)
    }

    pub(crate) fn seal_chunk(
        &self,
        nonce: [u8; NONCE_LEN],
        aad: &[u8],
        plaintext: &[u8],
    ) -> AppResult<Vec<u8>> {
        let payload = Payload {
            msg: plaintext,
            aad,
        };
        self.cipher
            .encrypt(&Nonce::from(nonce), payload)
            .map_err(|_| AppError::validation("ENCRYPTION_FAILED"))
    }

    pub(crate) fn open_chunk(
        &self,
        nonce: [u8; NONCE_LEN],
        aad: &[u8],
        ciphertext: &[u8],
    ) -> Option<Zeroizing<Vec<u8>>> {
        let payload = Payload {
            msg: ciphertext,
            aad,
        };
        self.cipher
            .decrypt(&Nonce::from(nonce), payload)
            .ok()
            .map(Zeroizing::new)
    }

    pub fn open_bytes(&self, sealed: &[u8]) -> AppResult<Zeroizing<Vec<u8>>> {
        let corrupted = || AppError::validation("SECRET_CORRUPTED");
        let (nonce, body) = sealed.split_at_checked(NONCE_LEN).ok_or_else(corrupted)?;
        let nonce: [u8; NONCE_LEN] = nonce.try_into().map_err(|_| corrupted())?;
        self.cipher
            .decrypt(&Nonce::from(nonce), body)
            .map(Zeroizing::new)
            .map_err(|_| corrupted())
    }

    pub fn decrypt(&self, sealed: &str) -> AppResult<Zeroizing<Vec<u8>>> {
        let corrupted = || AppError::validation("SECRET_CORRUPTED");
        let mut parts = sealed.splitn(3, ':');
        let (Some(VERSION_PREFIX), Some(nonce), Some(body)) =
            (parts.next(), parts.next(), parts.next())
        else {
            return Err(corrupted());
        };
        let nonce: [u8; NONCE_LEN] = STANDARD_NO_PAD
            .decode(nonce)
            .ok()
            .and_then(|bytes| bytes.try_into().ok())
            .ok_or_else(corrupted)?;
        let body = STANDARD_NO_PAD.decode(body).map_err(|_| corrupted())?;
        self.cipher
            .decrypt(&Nonce::from(nonce), body.as_slice())
            .map(Zeroizing::new)
            .map_err(|_| corrupted())
    }
}

fn read_key(path: &Path) -> AppResult<Zeroizing<Vec<u8>>> {
    let encoded = Zeroizing::new(fs::read_to_string(path).map_err(AppError::internal)?);
    let key = STANDARD_NO_PAD
        .decode(encoded.trim())
        .map_err(|_| AppError::validation("MASTER_KEY_INVALID"))?;
    if key.len() != KEY_LEN {
        return Err(AppError::validation("MASTER_KEY_INVALID"));
    }
    Ok(Zeroizing::new(key))
}

fn create_key(path: &Path) -> AppResult<Zeroizing<Vec<u8>>> {
    let key = Zeroizing::new(random_bytes::<KEY_LEN>().to_vec());
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(AppError::internal)?;
    }
    let mut file = owner_only_file(path)?;
    file.write_all(STANDARD_NO_PAD.encode(key.as_slice()).as_bytes())
        .map_err(AppError::internal)?;
    Ok(key)
}

#[cfg(unix)]
fn owner_only_file(path: &Path) -> AppResult<fs::File> {
    use std::os::unix::fs::OpenOptionsExt;
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)
        .map_err(AppError::internal)
}

#[cfg(not(unix))]
fn owner_only_file(path: &Path) -> AppResult<fs::File> {
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(AppError::internal)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_and_tamper_detection() -> AppResult<()> {
        let secrets = SecretBox::from_key(&random_bytes::<KEY_LEN>())?;
        let sealed = secrets.encrypt(b"database-password")?;
        assert_eq!(secrets.decrypt(&sealed)?.as_slice(), b"database-password");
        let tampered = format!("{}A", &sealed[..sealed.len() - 1]);
        assert!(secrets.decrypt(&tampered).is_err());
        assert!(secrets.decrypt("v2:abc:def").is_err());
        let blob = secrets.seal_bytes(b"archive")?;
        assert_eq!(secrets.open_bytes(&blob)?.as_slice(), b"archive");
        let mut flipped = blob.clone();
        if let Some(last) = flipped.last_mut() {
            *last ^= 1;
        }
        assert!(secrets.open_bytes(&flipped).is_err());
        assert!(secrets.open_bytes(&blob[..4]).is_err());
        Ok(())
    }

    #[test]
    fn creates_and_reloads_master_key() -> AppResult<()> {
        let dir = tempfile::tempdir().map_err(AppError::internal)?;
        let path = dir.path().join("keys").join("master.key");
        let first = SecretBox::load_or_create(&path)?;
        let sealed = first.encrypt(b"value")?;
        let second = SecretBox::load_or_create(&path)?;
        assert_eq!(second.decrypt(&sealed)?.as_slice(), b"value");
        Ok(())
    }
}
