use ferrodock_engine::updater::{UpdateRecord, UpdateState, UpdateStatus};
use serde::Serialize;
use time::OffsetDateTime;
use utoipa::ToSchema;

#[derive(Debug, Clone, Copy, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum UpdateStateDto {
    Pending,
    Applying,
    Succeeded,
    RolledBack,
    Failed,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UpdateRecordDto {
    pub state: UpdateStateDto,
    pub from_image: Option<String>,
    pub to_image: String,
    pub error: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    #[schema(value_type = String, format = DateTime)]
    pub started_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339::option")]
    #[schema(value_type = Option<String>, format = DateTime)]
    pub finished_at: Option<OffsetDateTime>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UpdateOverviewDto {
    pub current_version: String,
    pub last: Option<UpdateRecordDto>,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum UpdateCheckDto {
    UpToDate {
        current: String,
    },
    Available {
        current: String,
        latest: String,
        notes_url: String,
    },
    RequiresIntermediate {
        current: String,
        latest: String,
        min_from_version: String,
    },
}

impl From<UpdateState> for UpdateStateDto {
    fn from(state: UpdateState) -> Self {
        match state {
            UpdateState::Pending => Self::Pending,
            UpdateState::Applying => Self::Applying,
            UpdateState::Succeeded => Self::Succeeded,
            UpdateState::RolledBack => Self::RolledBack,
            UpdateState::Failed => Self::Failed,
        }
    }
}

impl From<UpdateRecord> for UpdateRecordDto {
    fn from(record: UpdateRecord) -> Self {
        Self {
            state: record.state.into(),
            from_image: record.from_image,
            to_image: record.to_image,
            error: record.error,
            started_at: record.started_at,
            finished_at: record.finished_at,
        }
    }
}

impl From<UpdateStatus> for UpdateCheckDto {
    fn from(status: UpdateStatus) -> Self {
        match status {
            UpdateStatus::UpToDate { current } => Self::UpToDate {
                current: current.to_string(),
            },
            UpdateStatus::Available {
                current,
                latest,
                notes_url,
                ..
            } => Self::Available {
                current: current.to_string(),
                latest: latest.to_string(),
                notes_url,
            },
            UpdateStatus::RequiresIntermediate {
                current,
                latest,
                min_from_version,
            } => Self::RequiresIntermediate {
                current: current.to_string(),
                latest: latest.to_string(),
                min_from_version: min_from_version.to_string(),
            },
        }
    }
}
