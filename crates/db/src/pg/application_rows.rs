use ferrodock_core::{AppError, AppResult, ErrorKind};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::applications::{ApplicationRecord, DeployKeyRecord, GitSourceRecord, SourceRecord};

pub(super) struct ApplicationRow {
    pub(super) id: Uuid,
    pub(super) environment_id: Uuid,
    pub(super) project_id: Uuid,
    pub(super) name: String,
    pub(super) slug: String,
    pub(super) description: String,
    pub(super) source_kind: String,
    pub(super) git_url: Option<String>,
    pub(super) git_branch: Option<String>,
    pub(super) build_kind: Option<String>,
    pub(super) build_path: Option<String>,
    pub(super) build_context: Option<String>,
    pub(super) image: Option<String>,
    pub(super) deploy_key_public: Option<String>,
    pub(super) deploy_key_private_enc: Option<String>,
    pub(super) strategy: String,
    pub(super) active_container: Option<String>,
    pub(super) compose_trusted: bool,
    pub(super) created_at: OffsetDateTime,
    pub(super) updated_at: OffsetDateTime,
}

#[derive(Default)]
pub(super) struct SourceColumns<'a> {
    pub(super) kind: &'static str,
    pub(super) git_url: Option<&'a str>,
    pub(super) git_branch: Option<&'a str>,
    pub(super) build_kind: Option<&'a str>,
    pub(super) build_path: Option<&'a str>,
    pub(super) build_context: Option<&'a str>,
    pub(super) image: Option<&'a str>,
}

pub(super) fn source_columns(source: &SourceRecord) -> SourceColumns<'_> {
    match source {
        SourceRecord::Git(git) => SourceColumns {
            kind: "git",
            git_url: Some(&git.url),
            git_branch: Some(&git.branch),
            build_kind: Some(&git.build_kind),
            build_path: Some(&git.build_path),
            build_context: Some(&git.build_context),
            image: None,
        },
        SourceRecord::Image(image) => SourceColumns {
            kind: "image",
            image: Some(image),
            ..SourceColumns::default()
        },
    }
}

fn invalid_row() -> AppError {
    AppError::new(ErrorKind::Internal, "APPLICATION_ROW_INVALID")
}

fn git_source(row: &mut ApplicationRow) -> Option<SourceRecord> {
    Some(SourceRecord::Git(GitSourceRecord {
        url: row.git_url.take()?,
        branch: row.git_branch.take()?,
        build_kind: row.build_kind.take()?,
        build_path: row.build_path.take()?,
        build_context: row.build_context.take()?,
    }))
}

impl TryFrom<ApplicationRow> for ApplicationRecord {
    type Error = AppError;

    fn try_from(mut row: ApplicationRow) -> AppResult<Self> {
        let source = match row.source_kind.as_str() {
            "git" => git_source(&mut row),
            "image" => row.image.take().map(SourceRecord::Image),
            _ => None,
        }
        .ok_or_else(invalid_row)?;
        let deploy_key = match (row.deploy_key_public, row.deploy_key_private_enc) {
            (Some(public), Some(private_enc)) => Some(DeployKeyRecord {
                public,
                private_enc,
            }),
            _ => None,
        };
        Ok(Self {
            id: row.id.into(),
            environment_id: row.environment_id.into(),
            project_id: row.project_id.into(),
            name: row.name,
            slug: row.slug,
            description: row.description,
            source,
            deploy_key,
            strategy: row.strategy,
            active_container: row.active_container,
            compose_trusted: row.compose_trusted,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}

pub(super) fn into_records(rows: Vec<ApplicationRow>) -> AppResult<Vec<ApplicationRecord>> {
    rows.into_iter().map(ApplicationRecord::try_from).collect()
}
