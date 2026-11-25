use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use ferrodock_core::{AppError, AppResult};
use semver::Version;
use serde::{Deserialize, Serialize};

const SCHEMA_VERSION: u32 = 1;
const DIGEST_PREFIX: &str = "sha256:";
const DIGEST_HEX_LEN: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComponentImages {
    pub traefik: String,
    pub postgres: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseManifest {
    pub schema: u32,
    pub channel: String,
    pub version: Version,
    pub image: String,
    pub digest: String,
    pub min_from_version: Version,
    pub published_at: String,
    pub notes_url: String,
    pub components: ComponentImages,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedManifest {
    pub payload: String,
    pub signature: String,
}

fn invalid(reason: &'static str) -> AppError {
    AppError::validation("UPDATE_MANIFEST_INVALID").with_param("reason", reason)
}

fn valid_digest(digest: &str) -> bool {
    digest.strip_prefix(DIGEST_PREFIX).is_some_and(|hex| {
        hex.len() == DIGEST_HEX_LEN && hex.bytes().all(|byte| byte.is_ascii_hexdigit())
    })
}

impl ReleaseManifest {
    pub fn validate(&self) -> AppResult<()> {
        if self.schema != SCHEMA_VERSION {
            return Err(invalid("UNSUPPORTED_SCHEMA"));
        }
        if !valid_digest(&self.digest) {
            return Err(invalid("DIGEST"));
        }
        if self.image.is_empty() || self.image.contains(char::is_whitespace) {
            return Err(invalid("IMAGE"));
        }
        if self.min_from_version > self.version {
            return Err(invalid("MIN_FROM_VERSION"));
        }
        Ok(())
    }

    pub fn pinned_image(&self) -> String {
        format!("{}@{}", self.image, self.digest)
    }
}

impl SignedManifest {
    pub fn sign(manifest: &ReleaseManifest, key: &SigningKey) -> AppResult<Self> {
        manifest.validate()?;
        let payload = serde_json::to_vec(manifest).map_err(AppError::internal)?;
        let signature = key.sign(&payload);
        Ok(Self {
            payload: STANDARD.encode(&payload),
            signature: STANDARD.encode(signature.to_bytes()),
        })
    }

    pub fn verify(&self, key: &VerifyingKey) -> AppResult<ReleaseManifest> {
        let payload = STANDARD
            .decode(&self.payload)
            .map_err(|_| invalid("ENCODING"))?;
        let signature_bytes: [u8; 64] = STANDARD
            .decode(&self.signature)
            .ok()
            .and_then(|bytes| bytes.try_into().ok())
            .ok_or_else(|| invalid("ENCODING"))?;
        key.verify_strict(&payload, &Signature::from_bytes(&signature_bytes))
            .map_err(|_| AppError::forbidden().with_param("reason", "UPDATE_SIGNATURE_INVALID"))?;
        let manifest: ReleaseManifest =
            serde_json::from_slice(&payload).map_err(|_| invalid("PAYLOAD"))?;
        manifest.validate()?;
        Ok(manifest)
    }
}

pub fn parse_envelope(body: &str) -> AppResult<SignedManifest> {
    serde_json::from_str(body).map_err(|_| invalid("ENVELOPE"))
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    pub fn sample(version: &str) -> ReleaseManifest {
        ReleaseManifest {
            schema: SCHEMA_VERSION,
            channel: "stable".to_owned(),
            version: Version::parse(version).unwrap_or_else(|_| Version::new(0, 0, 0)),
            image: "ghcr.io/beratr061/ferrodock".to_owned(),
            digest: format!("{DIGEST_PREFIX}{}", "a".repeat(DIGEST_HEX_LEN)),
            min_from_version: Version::new(0, 1, 0),
            published_at: "2026-11-25T00:00:00Z".to_owned(),
            notes_url: "https://github.com/beratr061/FerroDock/releases".to_owned(),
            components: ComponentImages {
                traefik: "traefik:v3.8.0".to_owned(),
                postgres: "postgres:18.7-alpine".to_owned(),
            },
        }
    }

    pub fn key(seed: u8) -> SigningKey {
        SigningKey::from_bytes(&[seed; 32])
    }

    #[test]
    fn signed_manifest_roundtrips() -> AppResult<()> {
        let signed = SignedManifest::sign(&sample("0.2.0"), &key(7))?;
        let verified = signed.verify(&key(7).verifying_key())?;
        assert_eq!(verified, sample("0.2.0"));
        Ok(())
    }

    #[test]
    fn rejects_foreign_key_and_tampering() -> AppResult<()> {
        let signed = SignedManifest::sign(&sample("0.2.0"), &key(7))?;
        assert!(signed.verify(&key(8).verifying_key()).is_err());
        let mut forged = signed.clone();
        let forged_payload = serde_json::to_vec(&sample("9.9.9")).map_err(AppError::internal)?;
        forged.payload = STANDARD.encode(forged_payload);
        assert!(forged.verify(&key(7).verifying_key()).is_err());
        Ok(())
    }

    #[test]
    fn rejects_malformed_digest() {
        let mut manifest = sample("0.2.0");
        manifest.digest = "sha256:short".to_owned();
        assert!(SignedManifest::sign(&manifest, &key(7)).is_err());
    }
}
