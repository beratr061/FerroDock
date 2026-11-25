use std::time::Duration;

use ferrodock_core::{AppError, AppResult, ErrorKind};
use reqwest::StatusCode;
use reqwest::header::{ETAG, IF_NONE_MATCH};

const TIMEOUT: Duration = Duration::from_secs(15);
const MAX_MANIFEST_BYTES: usize = 64 * 1024;

#[derive(Debug, Clone)]
pub enum Fetched {
    NotModified,
    Body { body: String, etag: Option<String> },
}

#[derive(Debug, Clone)]
pub struct ManifestClient {
    http: reqwest::Client,
    url: String,
}

fn unavailable(error: impl Into<Box<dyn std::error::Error + Send + Sync>>) -> AppError {
    AppError::new(ErrorKind::Unavailable, "UPDATE_SERVER_UNAVAILABLE").with_source(error)
}

impl ManifestClient {
    pub fn new(url: impl Into<String>, user_agent: &str) -> AppResult<Self> {
        let _ = rustls::crypto::ring::default_provider().install_default();
        let http = reqwest::Client::builder()
            .timeout(TIMEOUT)
            .user_agent(user_agent)
            .https_only(true)
            .build()
            .map_err(AppError::internal)?;
        Ok(Self {
            http,
            url: url.into(),
        })
    }

    pub async fn fetch(&self, etag: Option<&str>) -> AppResult<Fetched> {
        let mut request = self.http.get(&self.url);
        if let Some(tag) = etag {
            request = request.header(IF_NONE_MATCH, tag);
        }
        let response = request.send().await.map_err(unavailable)?;
        if response.status() == StatusCode::NOT_MODIFIED {
            return Ok(Fetched::NotModified);
        }
        let response = response.error_for_status().map_err(unavailable)?;
        let etag = response
            .headers()
            .get(ETAG)
            .and_then(|value| value.to_str().ok())
            .map(str::to_owned);
        let bytes = response.bytes().await.map_err(unavailable)?;
        if bytes.len() > MAX_MANIFEST_BYTES {
            return Err(
                AppError::validation("UPDATE_MANIFEST_INVALID").with_param("reason", "SIZE")
            );
        }
        let body = String::from_utf8(bytes.to_vec()).map_err(|_| {
            AppError::validation("UPDATE_MANIFEST_INVALID").with_param("reason", "ENCODING")
        })?;
        Ok(Fetched::Body { body, etag })
    }
}
