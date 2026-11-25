use std::net::{IpAddr, SocketAddr};
use std::path::{Path, PathBuf};

use figment::Figment;
use figment::providers::{Env, Format, Serialized, Toml};
use serde::{Deserialize, Serialize};

use crate::{AppError, AppResult, Secret};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProxyMode {
    Traefik,
    None,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub bind_addr: SocketAddr,
    pub database_url: Secret<String>,
    pub database_max_connections: u32,
    pub data_dir: PathBuf,
    pub master_key_path: PathBuf,
    pub public_url: Option<String>,
    pub secure_cookies: bool,
    pub log_json: bool,
    pub session_ttl_hours: u32,
    pub trust_forwarded_for: bool,
    pub job_concurrency: u32,
    pub setup_token: Option<Secret<String>>,
    pub proxy: ProxyMode,
    pub proxy_dir: PathBuf,
    pub public_ip: Option<IpAddr>,
    pub backup_interval_hours: u32,
    pub backup_retention: u32,
    pub database_container: Option<String>,
    pub host_config_dir: String,
    pub update_manifest_url: String,
}

#[derive(Serialize)]
struct Defaults {
    bind_addr: &'static str,
    database_max_connections: u32,
    data_dir: &'static str,
    master_key_path: &'static str,
    secure_cookies: bool,
    log_json: bool,
    session_ttl_hours: u32,
    trust_forwarded_for: bool,
    job_concurrency: u32,
    proxy: ProxyMode,
    proxy_dir: &'static str,
    backup_interval_hours: u32,
    backup_retention: u32,
    host_config_dir: &'static str,
    update_manifest_url: &'static str,
}

const DEFAULTS: Defaults = Defaults {
    bind_addr: "0.0.0.0:3000",
    database_max_connections: 10,
    data_dir: "/var/lib/ferrodock",
    master_key_path: "/etc/ferrodock/master.key",
    secure_cookies: true,
    log_json: true,
    session_ttl_hours: 168,
    trust_forwarded_for: false,
    job_concurrency: 2,
    proxy: ProxyMode::None,
    proxy_dir: "/etc/ferrodock/proxy",
    backup_interval_hours: 24,
    backup_retention: 7,
    host_config_dir: "/etc/ferrodock",
    update_manifest_url: "https://github.com/beratr061/FerroDock/releases/latest/download/manifest.json",
};

impl AppConfig {
    pub fn load(config_file: Option<&Path>) -> AppResult<Self> {
        let mut figment = Figment::from(Serialized::defaults(DEFAULTS));
        if let Some(path) = config_file {
            figment = figment.merge(Toml::file(path));
        }
        figment
            .merge(Env::prefixed("FERRODOCK_"))
            .extract()
            .map_err(|error| AppError::validation("CONFIG_INVALID").with_source(error))
    }
}
