use std::net::SocketAddr;
use std::path::{Path, PathBuf};

use figment::Figment;
use figment::providers::{Env, Format, Serialized, Toml};
use serde::{Deserialize, Serialize};

use crate::{AppError, AppResult, Secret};

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
    pub setup_token: Option<Secret<String>>,
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
