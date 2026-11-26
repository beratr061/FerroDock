mod apply;
mod check;
mod client;
mod env_file;
mod launch;
mod manifest;
mod status;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use ed25519_dalek::VerifyingKey;
use ferrodock_core::{AppError, AppResult};
use semver::Version;

pub use apply::{InstallLayout, PANEL_PROJECT, PANEL_SERVICE, apply, layout};
pub use check::{UpdateStatus, evaluate};
pub use client::{Fetched, ManifestClient};
pub use env_file::{IMAGE_VARIABLE, image_of};
pub use launch::UpdateLauncher;
pub use manifest::{ComponentImages, ReleaseManifest, SignedManifest, parse_envelope};
pub use status::{UpdateRecord, UpdateState, read_status, write_status};

pub const DEFAULT_MANIFEST_URL: &str =
    "https://github.com/beratr061/FerroDock/releases/latest/download/manifest.json";

pub fn running_version() -> Version {
    Version::parse(env!("CARGO_PKG_VERSION")).unwrap_or_else(|_| Version::new(0, 0, 0))
}

pub fn release_key() -> AppResult<VerifyingKey> {
    let encoded = option_env!("FERRODOCK_RELEASE_PUBLIC_KEY")
        .filter(|value| !value.is_empty())
        .ok_or_else(|| AppError::validation("UPDATE_KEY_NOT_CONFIGURED"))?;
    decode_public_key(encoded)
}

pub fn decode_public_key(encoded: &str) -> AppResult<VerifyingKey> {
    let bytes: [u8; 32] = STANDARD
        .decode(encoded.trim())
        .ok()
        .and_then(|bytes| bytes.try_into().ok())
        .ok_or_else(|| AppError::validation("UPDATE_KEY_INVALID"))?;
    VerifyingKey::from_bytes(&bytes).map_err(|_| AppError::validation("UPDATE_KEY_INVALID"))
}

pub async fn check_for_update(
    client: &ManifestClient,
    key: &VerifyingKey,
    current: &Version,
) -> AppResult<UpdateStatus> {
    let body = client.latest().await?;
    let manifest = parse_envelope(&body)?.verify(key)?;
    Ok(evaluate(current, &manifest))
}
