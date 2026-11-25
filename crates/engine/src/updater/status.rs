use std::path::Path;

use ferrodock_core::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UpdateState {
    Pending,
    Applying,
    Succeeded,
    RolledBack,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateRecord {
    pub state: UpdateState,
    pub from_image: Option<String>,
    pub to_image: String,
    pub error: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub started_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339::option")]
    pub finished_at: Option<OffsetDateTime>,
}

impl UpdateRecord {
    pub fn pending(from_image: Option<String>, to_image: String) -> Self {
        Self {
            state: UpdateState::Pending,
            from_image,
            to_image,
            error: None,
            started_at: OffsetDateTime::now_utc(),
            finished_at: None,
        }
    }

    #[must_use]
    pub fn finish(mut self, state: UpdateState, error: Option<&str>) -> Self {
        self.state = state;
        self.error = error.map(str::to_owned);
        self.finished_at = Some(OffsetDateTime::now_utc());
        self
    }
}

pub async fn read_status(path: &Path) -> AppResult<Option<UpdateRecord>> {
    match tokio::fs::read(path).await {
        Ok(bytes) => serde_json::from_slice(&bytes)
            .map(Some)
            .map_err(AppError::internal),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(AppError::internal(error)),
    }
}

pub async fn write_status(path: &Path, record: &UpdateRecord) -> AppResult<()> {
    let body = serde_json::to_vec_pretty(record).map_err(AppError::internal)?;
    let staging = path.with_extension("json.tmp");
    tokio::fs::write(&staging, body)
        .await
        .map_err(AppError::internal)?;
    tokio::fs::rename(&staging, path)
        .await
        .map_err(AppError::internal)
}
