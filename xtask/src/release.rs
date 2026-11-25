use std::fs;
use std::io::Write;
use std::path::Path;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use ed25519_dalek::SigningKey;
use ferrodock_engine::updater::{ComponentImages, ReleaseManifest, SignedManifest};
use semver::Version;
use time::OffsetDateTime;
use time::format_description::well_known::Rfc3339;

const SIGNING_KEY_ENV: &str = "FERRODOCK_RELEASE_SIGNING_KEY";
const IMAGE: &str = "ghcr.io/beratr061/ferrodock";
const NOTES_BASE: &str = "https://github.com/beratr061/FerroDock/releases/tag/v";
const MIN_FROM_VERSION: &str = "0.1.0";

pub fn generate_key(output: &Path) -> Result<(), String> {
    let mut seed = [0u8; 32];
    rand::fill(&mut seed);
    let key = SigningKey::from_bytes(&seed);
    let mut file = owner_only(output)?;
    file.write_all(STANDARD.encode(seed).as_bytes())
        .map_err(|error| format!("{}: {error}", output.display()))?;
    println!("private key written to {}", output.display());
    println!("store it as the GitHub secret {SIGNING_KEY_ENV}, then delete the file");
    println!(
        "public key (GitHub variable FERRODOCK_RELEASE_PUBLIC_KEY): {}",
        STANDARD.encode(key.verifying_key().to_bytes())
    );
    Ok(())
}

#[cfg(unix)]
fn owner_only(path: &Path) -> Result<fs::File, String> {
    use std::os::unix::fs::OpenOptionsExt;
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)
        .map_err(|error| format!("{}: {error}", path.display()))
}

#[cfg(not(unix))]
fn owner_only(path: &Path) -> Result<fs::File, String> {
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| format!("{}: {error}", path.display()))
}

fn decode_seed(encoded: &str) -> Result<SigningKey, String> {
    let seed: [u8; 32] = STANDARD
        .decode(encoded.trim())
        .ok()
        .and_then(|bytes| bytes.try_into().ok())
        .ok_or_else(|| "the key is not a base64 ed25519 seed".to_owned())?;
    Ok(SigningKey::from_bytes(&seed))
}

fn signing_key() -> Result<SigningKey, String> {
    let encoded =
        std::env::var(SIGNING_KEY_ENV).map_err(|_| format!("{SIGNING_KEY_ENV} is not set"))?;
    decode_seed(&encoded)
}

pub fn print_public_key(key_file: &Path) -> Result<(), String> {
    let encoded =
        fs::read_to_string(key_file).map_err(|error| format!("{}: {error}", key_file.display()))?;
    let key = decode_seed(&encoded)?;
    println!("{}", STANDARD.encode(key.verifying_key().to_bytes()));
    Ok(())
}

fn compose_image(root: &Path, name: &str) -> Result<String, String> {
    let compose = fs::read_to_string(root.join("deploy/docker-compose.yml"))
        .map_err(|error| error.to_string())?;
    compose
        .lines()
        .filter_map(|line| line.trim().strip_prefix("image: "))
        .find(|image| image.starts_with(&format!("{name}:")))
        .map(str::to_owned)
        .ok_or_else(|| format!("no pinned {name} image in deploy/docker-compose.yml"))
}

fn build_manifest(root: &Path, version: &str, digest: &str) -> Result<ReleaseManifest, String> {
    let version =
        Version::parse(version.trim_start_matches('v')).map_err(|error| error.to_string())?;
    let published_at = OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .map_err(|error| error.to_string())?;
    Ok(ReleaseManifest {
        schema: 1,
        channel: "stable".to_owned(),
        notes_url: format!("{NOTES_BASE}{version}"),
        version,
        image: IMAGE.to_owned(),
        digest: digest.to_owned(),
        min_from_version: Version::parse(MIN_FROM_VERSION).map_err(|error| error.to_string())?,
        published_at,
        components: ComponentImages {
            traefik: compose_image(root, "traefik")?,
            postgres: compose_image(root, "postgres")?,
        },
    })
}

pub fn sign_manifest(
    root: &Path,
    version: &str,
    digest: &str,
    output: &Path,
) -> Result<(), String> {
    let key = signing_key()?;
    let manifest = build_manifest(root, version, digest)?;
    let signed = SignedManifest::sign(&manifest, &key).map_err(|error| error.code().to_owned())?;
    signed
        .verify(&key.verifying_key())
        .map_err(|error| error.code().to_owned())?;
    let body = serde_json::to_string_pretty(&signed).map_err(|error| error.to_string())?;
    fs::write(output, body + "\n").map_err(|error| format!("{}: {error}", output.display()))?;
    println!(
        "signed manifest for {} written to {}",
        manifest.version,
        output.display()
    );
    Ok(())
}
