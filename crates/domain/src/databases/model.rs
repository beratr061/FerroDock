use ferrodock_core::{AppResult, Id};
use ferrodock_db::databases::DatabaseRecord;
use time::OffsetDateTime;

use super::drivers::driver;
use super::spec::alias;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatabaseStatus {
    Deploying,
    Running,
    Stopped,
    Failed,
}

impl DatabaseStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Deploying => "deploying",
            Self::Running => "running",
            Self::Stopped => "stopped",
            Self::Failed => "failed",
        }
    }

    pub fn parse(value: &str) -> Self {
        match value {
            "deploying" => Self::Deploying,
            "running" => Self::Running,
            "stopped" => Self::Stopped,
            _ => Self::Failed,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Database {
    pub id: Id,
    pub environment_id: Id,
    pub project_id: Id,
    pub name: String,
    pub slug: String,
    pub engine: String,
    pub version: String,
    pub database_name: String,
    pub username: String,
    pub internal_host: String,
    pub internal_port: u16,
    pub external_port: Option<u16>,
    pub status: DatabaseStatus,
    pub error_code: Option<String>,
    pub running: bool,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

impl Database {
    pub(super) fn from_record(record: DatabaseRecord, running: bool) -> AppResult<Self> {
        let engine = driver(&record.engine)?;
        Ok(Self {
            internal_host: alias(record.id),
            internal_port: engine.port(),
            status: DatabaseStatus::parse(&record.status),
            id: record.id,
            environment_id: record.environment_id,
            project_id: record.project_id,
            name: record.name,
            slug: record.slug,
            engine: record.engine,
            version: record.version,
            database_name: record.database_name,
            username: record.username,
            external_port: record.external_port,
            error_code: record.error_code,
            running,
            created_at: record.created_at,
            updated_at: record.updated_at,
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct NewDatabaseInput {
    pub name: String,
    pub engine: String,
    pub version: Option<String>,
    pub database_name: Option<String>,
    pub username: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatabaseAction {
    Deploy,
    Start,
    Stop,
}

#[derive(Debug, Clone)]
pub struct DatabaseCredentials {
    pub username: String,
    pub password: String,
    pub root_password: Option<String>,
    pub internal_url: String,
    pub external_url: Option<String>,
}
