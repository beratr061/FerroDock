use ferrodock_domain::databases::{
    Database, DatabaseAction, DatabaseCredentials, DatabaseDriver, DatabaseStatus,
    NewDatabaseInput, ResourceLimits,
};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum DatabaseStatusDto {
    Deploying,
    Running,
    Stopped,
    Failed,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DatabaseDto {
    pub id: Uuid,
    pub environment_id: Uuid,
    pub project_id: Uuid,
    pub name: String,
    pub slug: String,
    pub engine: String,
    pub version: String,
    pub database_name: String,
    pub username: String,
    pub internal_host: String,
    pub internal_port: u16,
    pub external_port: Option<u16>,
    pub memory_limit_mb: Option<u32>,
    pub cpu_limit_millis: Option<u32>,
    pub status: DatabaseStatusDto,
    pub error_code: Option<String>,
    pub running: bool,
    #[serde(with = "time::serde::rfc3339")]
    #[schema(value_type = String, format = DateTime)]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    #[schema(value_type = String, format = DateTime)]
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateDatabaseRequest {
    pub name: String,
    pub engine: String,
    pub version: Option<String>,
    pub database_name: Option<String>,
    pub username: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateDatabaseRequest {
    pub external_port: Option<u16>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct DatabaseLimitsRequest {
    pub memory_mb: Option<u32>,
    pub cpu_millis: Option<u32>,
}

impl From<DatabaseLimitsRequest> for ResourceLimits {
    fn from(request: DatabaseLimitsRequest) -> Self {
        Self {
            memory_mb: request.memory_mb,
            cpu_millis: request.cpu_millis,
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum DatabaseActionDto {
    Deploy,
    Start,
    Stop,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct DatabaseActionRequest {
    pub action: DatabaseActionDto,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DatabaseCredentialsDto {
    pub username: String,
    pub password: String,
    pub root_password: Option<String>,
    pub internal_url: String,
    pub external_url: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DatabaseEngineDto {
    pub engine: String,
    pub versions: Vec<String>,
    pub port: u16,
    pub has_username: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DatabaseLogDto {
    pub text: String,
    pub next_offset: u64,
    pub status: DatabaseStatusDto,
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct RemoveDataQuery {
    pub remove_data: Option<bool>,
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct DatabaseLogQuery {
    pub offset: Option<u64>,
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct DatabaseLogsQuery {
    pub tail: Option<u32>,
}

impl From<DatabaseStatus> for DatabaseStatusDto {
    fn from(status: DatabaseStatus) -> Self {
        match status {
            DatabaseStatus::Deploying => Self::Deploying,
            DatabaseStatus::Running => Self::Running,
            DatabaseStatus::Stopped => Self::Stopped,
            DatabaseStatus::Failed => Self::Failed,
        }
    }
}

impl From<Database> for DatabaseDto {
    fn from(database: Database) -> Self {
        Self {
            id: database.id.as_uuid(),
            environment_id: database.environment_id.as_uuid(),
            project_id: database.project_id.as_uuid(),
            name: database.name,
            slug: database.slug,
            engine: database.engine,
            version: database.version,
            database_name: database.database_name,
            username: database.username,
            internal_host: database.internal_host,
            internal_port: database.internal_port,
            external_port: database.external_port,
            memory_limit_mb: database.limits.memory_mb,
            cpu_limit_millis: database.limits.cpu_millis,
            status: database.status.into(),
            error_code: database.error_code,
            running: database.running,
            created_at: database.created_at,
            updated_at: database.updated_at,
        }
    }
}

impl From<CreateDatabaseRequest> for NewDatabaseInput {
    fn from(request: CreateDatabaseRequest) -> Self {
        Self {
            name: request.name,
            engine: request.engine,
            version: request.version,
            database_name: request.database_name,
            username: request.username,
        }
    }
}

impl From<DatabaseActionDto> for DatabaseAction {
    fn from(action: DatabaseActionDto) -> Self {
        match action {
            DatabaseActionDto::Deploy => Self::Deploy,
            DatabaseActionDto::Start => Self::Start,
            DatabaseActionDto::Stop => Self::Stop,
        }
    }
}

impl From<DatabaseCredentials> for DatabaseCredentialsDto {
    fn from(credentials: DatabaseCredentials) -> Self {
        Self {
            username: credentials.username,
            password: credentials.password,
            root_password: credentials.root_password,
            internal_url: credentials.internal_url,
            external_url: credentials.external_url,
        }
    }
}

impl From<&dyn DatabaseDriver> for DatabaseEngineDto {
    fn from(driver: &dyn DatabaseDriver) -> Self {
        Self {
            engine: driver.engine().to_owned(),
            versions: driver.versions().iter().map(|v| (*v).to_owned()).collect(),
            port: driver.port(),
            has_username: driver.fixed_username().is_none(),
        }
    }
}
