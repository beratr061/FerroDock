use ferrodock_core::{AppError, AppResult};
use ferrodock_db::applications::{ApplicationRecord, GitSourceRecord, SourceRecord};
use ferrodock_infra::docker::validate_image;
use ferrodock_infra::git::{validate_branch, validate_relative_path};

use super::model::{Application, BuildKind, GitSource, Source, SourceInput, Strategy};

const DEFAULT_BRANCH: &str = "main";
const DEFAULT_CONTEXT: &str = ".";

impl BuildKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Dockerfile => "dockerfile",
            Self::Compose => "compose",
        }
    }

    fn default_path(self) -> &'static str {
        match self {
            Self::Dockerfile => "Dockerfile",
            Self::Compose => "docker-compose.yml",
        }
    }

    fn parse(value: &str) -> AppResult<Self> {
        match value {
            "dockerfile" => Ok(Self::Dockerfile),
            "compose" => Ok(Self::Compose),
            _ => Err(AppError::validation("BUILD_KIND_INVALID")),
        }
    }
}

impl Strategy {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Recreate => "recreate",
            Self::BlueGreen => "blue_green",
        }
    }

    fn parse(value: &str) -> AppResult<Self> {
        match value {
            "recreate" => Ok(Self::Recreate),
            "blue_green" => Ok(Self::BlueGreen),
            _ => Err(AppError::validation("STRATEGY_INVALID")),
        }
    }
}

fn trimmed_or(value: Option<&str>, fallback: &str) -> String {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(fallback)
        .to_owned()
}

pub(super) fn validate_source(input: &SourceInput) -> AppResult<Source> {
    match input {
        SourceInput::Image(image) => {
            let image = image.trim();
            validate_image(image)?;
            Ok(Source::Image(image.to_owned()))
        }
        SourceInput::Git(git) => {
            let url = git.url.trim().to_owned();
            ferrodock_infra::git::parse_repository_url(&url)?;
            let branch = trimmed_or(git.branch.as_deref(), DEFAULT_BRANCH);
            validate_branch(&branch)?;
            let build_path = trimmed_or(git.build_path.as_deref(), git.build_kind.default_path());
            let build_context = trimmed_or(git.build_context.as_deref(), DEFAULT_CONTEXT);
            validate_relative_path(&build_path)?;
            validate_relative_path(&build_context)?;
            Ok(Source::Git(GitSource {
                url,
                branch,
                build_kind: git.build_kind,
                build_path,
                build_context,
            }))
        }
    }
}

pub(super) fn to_record(source: &Source) -> SourceRecord {
    match source {
        Source::Image(image) => SourceRecord::Image(image.clone()),
        Source::Git(git) => SourceRecord::Git(GitSourceRecord {
            url: git.url.clone(),
            branch: git.branch.clone(),
            build_kind: git.build_kind.as_str().to_owned(),
            build_path: git.build_path.clone(),
            build_context: git.build_context.clone(),
        }),
    }
}

fn from_record(source: SourceRecord) -> AppResult<Source> {
    Ok(match source {
        SourceRecord::Image(image) => Source::Image(image),
        SourceRecord::Git(git) => Source::Git(GitSource {
            build_kind: BuildKind::parse(&git.build_kind)?,
            url: git.url,
            branch: git.branch,
            build_path: git.build_path,
            build_context: git.build_context,
        }),
    })
}

pub(super) fn parse_strategy(value: &str) -> AppResult<Strategy> {
    Strategy::parse(value)
}

impl TryFrom<ApplicationRecord> for Application {
    type Error = AppError;

    fn try_from(record: ApplicationRecord) -> AppResult<Self> {
        Ok(Self {
            strategy: Strategy::parse(&record.strategy)?,
            source: from_record(record.source)?,
            deploy_key: record.deploy_key.map(|key| key.public),
            id: record.id,
            environment_id: record.environment_id,
            project_id: record.project_id,
            name: record.name,
            slug: record.slug,
            description: record.description,
            created_at: record.created_at,
            updated_at: record.updated_at,
        })
    }
}
