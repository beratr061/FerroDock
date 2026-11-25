use ferrodock_core::archive::{Entry, pack, unpack};
use ferrodock_core::crypto::SecretBox;
use ferrodock_core::{AppError, AppResult};
use semver::Version;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

const MAGIC: &[u8; 8] = b"FDBACKUP";
const FORMAT: u8 = 1;
pub const MANIFEST: &str = "manifest.json";
pub const DATABASE: &str = "database.dump";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub format: u8,
    pub version: String,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

fn invalid() -> AppError {
    AppError::validation("BACKUP_ARCHIVE_INVALID")
}

pub fn seal(
    secrets: &SecretBox,
    manifest: &Manifest,
    mut entries: Vec<Entry>,
) -> AppResult<Vec<u8>> {
    let manifest_bytes = serde_json::to_vec(manifest).map_err(AppError::internal)?;
    entries.insert(
        0,
        Entry {
            name: MANIFEST.to_owned(),
            data: manifest_bytes,
        },
    );
    let sealed = secrets.seal_bytes(&pack(&entries)?)?;
    let mut file = Vec::with_capacity(MAGIC.len() + 1 + sealed.len());
    file.extend_from_slice(MAGIC);
    file.push(FORMAT);
    file.extend_from_slice(&sealed);
    Ok(file)
}

pub fn open(secrets: &SecretBox, bytes: &[u8]) -> AppResult<(Manifest, Vec<Entry>)> {
    let body = bytes.strip_prefix(MAGIC.as_slice()).ok_or_else(invalid)?;
    let (&format, sealed) = body.split_first().ok_or_else(invalid)?;
    if format != FORMAT {
        return Err(AppError::validation("BACKUP_FORMAT_UNSUPPORTED").with_param("format", format));
    }
    let plain = secrets
        .open_bytes(sealed)
        .map_err(|_| AppError::validation("BACKUP_KEY_MISMATCH"))?;
    let mut entries = unpack(&plain)?;
    let first = (!entries.is_empty())
        .then(|| entries.remove(0))
        .ok_or_else(invalid)?;
    if first.name != MANIFEST {
        return Err(invalid());
    }
    let manifest: Manifest = serde_json::from_slice(&first.data).map_err(|_| invalid())?;
    Ok((manifest, entries))
}

pub fn ensure_compatible(manifest: &Manifest, running: &str) -> AppResult<()> {
    let archive = Version::parse(&manifest.version).map_err(|_| invalid())?;
    let current = Version::parse(running).map_err(|_| invalid())?;
    if archive > current {
        return Err(AppError::validation("BACKUP_FROM_NEWER_VERSION")
            .with_param("archive", manifest.version.as_str())
            .with_param("running", running));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest(version: &str) -> Manifest {
        Manifest {
            format: FORMAT,
            version: version.to_owned(),
            created_at: OffsetDateTime::UNIX_EPOCH,
        }
    }

    #[test]
    fn seals_and_opens_with_the_same_key_only() -> AppResult<()> {
        let key = SecretBox::from_key(&[1u8; 32])?;
        let entries = vec![Entry {
            name: DATABASE.to_owned(),
            data: b"dump".to_vec(),
        }];
        let file = seal(&key, &manifest("0.1.1"), entries.clone())?;
        let (read, restored) = open(&key, &file)?;
        assert_eq!((read.version.as_str(), restored), ("0.1.1", entries));
        let other = SecretBox::from_key(&[2u8; 32])?;
        assert_eq!(
            open(&other, &file).err().map(|e| e.code()),
            Some("BACKUP_KEY_MISMATCH")
        );
        assert!(open(&key, b"not a backup").is_err());
        Ok(())
    }

    #[test]
    fn refuses_archives_from_newer_releases() {
        assert!(ensure_compatible(&manifest("0.1.1"), "0.1.1").is_ok());
        assert!(ensure_compatible(&manifest("0.1.0"), "0.2.0").is_ok());
        assert!(ensure_compatible(&manifest("0.3.0"), "0.2.0").is_err());
    }
}
