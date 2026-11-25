use std::io::Write;
use std::path::{Path, PathBuf};

use ferrodock_core::{AppError, AppResult, Id};
use tokio::process::Command;
use tokio_util::sync::CancellationToken;

use super::source::{parse_repository_url, validate_branch};
use crate::deploy_log::DeployLog;
use crate::process::{capture, isolated, run_logged};

#[derive(Debug)]
pub struct CheckoutSpec<'a> {
    pub url: &'a str,
    pub branch: &'a str,
    pub destination: &'a Path,
    pub private_key: Option<&'a [u8]>,
    pub known_hosts: &'a Path,
}

struct KeyFile(PathBuf);

impl Drop for KeyFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

fn write_key(material: &[u8]) -> AppResult<KeyFile> {
    let path = std::env::temp_dir().join(format!("fd-key-{}", Id::new()));
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
    let mut file = options.open(&path).map_err(AppError::internal)?;
    let guard = KeyFile(path);
    file.write_all(material).map_err(AppError::internal)?;
    Ok(guard)
}

fn quoted(path: &Path) -> AppResult<String> {
    let text = path.to_string_lossy().replace('\\', "/");
    if text.contains(['\'', '\n']) {
        return Err(AppError::validation("CHECKOUT_PATH_INVALID"));
    }
    Ok(format!("'{text}'"))
}

fn ssh_command(key: &Path, known_hosts: &Path) -> AppResult<String> {
    Ok(format!(
        "ssh -i {} -o IdentitiesOnly=yes -o BatchMode=yes -o StrictHostKeyChecking=accept-new -o UserKnownHostsFile={}",
        quoted(key)?,
        quoted(known_hosts)?
    ))
}

fn git() -> Command {
    let mut command = isolated("git");
    command
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_ALLOW_PROTOCOL", "https:ssh")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .args([
            "-c",
            "core.symlinks=false",
            "-c",
            "protocol.file.allow=never",
        ]);
    command
}

async fn reset_destination(destination: &Path) -> AppResult<()> {
    if tokio::fs::try_exists(destination).await.unwrap_or(false) {
        tokio::fs::remove_dir_all(destination)
            .await
            .map_err(AppError::internal)?;
    }
    if let Some(parent) = destination.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(AppError::internal)?;
    }
    Ok(())
}

pub async fn checkout(
    spec: CheckoutSpec<'_>,
    log: &DeployLog,
    cancel: &CancellationToken,
) -> AppResult<String> {
    parse_repository_url(spec.url)?;
    validate_branch(spec.branch)?;
    reset_destination(spec.destination).await?;
    let key = spec.private_key.map(write_key).transpose()?;
    let mut command = git();
    if let Some(key) = &key {
        if let Some(parent) = spec.known_hosts.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(AppError::internal)?;
        }
        command.env("GIT_SSH_COMMAND", ssh_command(&key.0, spec.known_hosts)?);
    }
    command
        .args([
            "clone",
            "--depth",
            "1",
            "--single-branch",
            "--no-recurse-submodules",
        ])
        .args(["--branch", spec.branch, "--", spec.url])
        .arg(spec.destination);
    log.step(&format!("Cloning {} ({})", spec.url, spec.branch))
        .await;
    run_logged(command, log, cancel, "GIT_CLONE_FAILED").await?;
    drop(key);
    let mut head = git();
    head.arg("-C")
        .arg(spec.destination)
        .args(["rev-parse", "HEAD"]);
    capture(head, "GIT_CLONE_FAILED").await
}

pub fn contained_path(root: &Path, relative: &str) -> AppResult<PathBuf> {
    let escape = || AppError::validation("BUILD_PATH_INVALID").with_param("path", relative);
    let canonical_root = root.canonicalize().map_err(|_| escape())?;
    let joined = root.join(relative);
    let resolved = joined.canonicalize().map_err(|_| escape())?;
    if resolved.starts_with(&canonical_root) {
        Ok(joined)
    } else {
        Err(escape())
    }
}
