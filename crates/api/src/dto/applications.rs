use ferrodock_domain::applications::{
    Application, ApplicationUpdate, BuildKind, GitSourceInput, NewApplicationInput, Source,
    SourceInput, Strategy,
};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum BuildKindDto {
    Dockerfile,
    Compose,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum StrategyDto {
    Recreate,
    BlueGreen,
}

#[derive(Debug, Deserialize, ToSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SourceRequest {
    Git {
        url: String,
        branch: Option<String>,
        build_kind: BuildKindDto,
        build_path: Option<String>,
        build_context: Option<String>,
    },
    Image {
        image: String,
    },
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SourceDto {
    Git {
        url: String,
        branch: String,
        build_kind: BuildKindDto,
        build_path: String,
        build_context: String,
    },
    Image {
        image: String,
    },
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ApplicationDto {
    pub id: Uuid,
    pub environment_id: Uuid,
    pub project_id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub source: SourceDto,
    pub deploy_key: Option<String>,
    pub strategy: StrategyDto,
    #[serde(with = "time::serde::rfc3339")]
    #[schema(value_type = String, format = DateTime)]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    #[schema(value_type = String, format = DateTime)]
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateApplicationRequest {
    pub name: String,
    pub description: Option<String>,
    pub source: SourceRequest,
    pub strategy: Option<StrategyDto>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateApplicationRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub source: Option<SourceRequest>,
    pub strategy: Option<StrategyDto>,
}

impl From<BuildKind> for BuildKindDto {
    fn from(kind: BuildKind) -> Self {
        match kind {
            BuildKind::Dockerfile => Self::Dockerfile,
            BuildKind::Compose => Self::Compose,
        }
    }
}

impl From<BuildKindDto> for BuildKind {
    fn from(kind: BuildKindDto) -> Self {
        match kind {
            BuildKindDto::Dockerfile => Self::Dockerfile,
            BuildKindDto::Compose => Self::Compose,
        }
    }
}

impl From<Strategy> for StrategyDto {
    fn from(strategy: Strategy) -> Self {
        match strategy {
            Strategy::Recreate => Self::Recreate,
            Strategy::BlueGreen => Self::BlueGreen,
        }
    }
}

impl From<StrategyDto> for Strategy {
    fn from(strategy: StrategyDto) -> Self {
        match strategy {
            StrategyDto::Recreate => Self::Recreate,
            StrategyDto::BlueGreen => Self::BlueGreen,
        }
    }
}

impl From<SourceRequest> for SourceInput {
    fn from(source: SourceRequest) -> Self {
        match source {
            SourceRequest::Image { image } => Self::Image(image),
            SourceRequest::Git {
                url,
                branch,
                build_kind,
                build_path,
                build_context,
            } => Self::Git(GitSourceInput {
                url,
                branch,
                build_kind: build_kind.into(),
                build_path,
                build_context,
            }),
        }
    }
}

impl From<Source> for SourceDto {
    fn from(source: Source) -> Self {
        match source {
            Source::Image(image) => Self::Image { image },
            Source::Git(git) => Self::Git {
                url: git.url,
                branch: git.branch,
                build_kind: git.build_kind.into(),
                build_path: git.build_path,
                build_context: git.build_context,
            },
        }
    }
}

impl From<Application> for ApplicationDto {
    fn from(application: Application) -> Self {
        Self {
            id: application.id.as_uuid(),
            environment_id: application.environment_id.as_uuid(),
            project_id: application.project_id.as_uuid(),
            name: application.name,
            slug: application.slug,
            description: application.description,
            source: application.source.into(),
            deploy_key: application.deploy_key,
            strategy: application.strategy.into(),
            created_at: application.created_at,
            updated_at: application.updated_at,
        }
    }
}

impl From<CreateApplicationRequest> for NewApplicationInput {
    fn from(request: CreateApplicationRequest) -> Self {
        Self {
            name: request.name,
            description: request.description,
            source: request.source.into(),
            strategy: request.strategy.map(Into::into),
        }
    }
}

impl From<UpdateApplicationRequest> for ApplicationUpdate {
    fn from(request: UpdateApplicationRequest) -> Self {
        Self {
            name: request.name,
            description: request.description,
            source: request.source.map(Into::into),
            strategy: request.strategy.map(Into::into),
        }
    }
}
