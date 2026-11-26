use ferrodock_core::Id;
use time::OffsetDateTime;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildKind {
    Dockerfile,
    Compose,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Strategy {
    #[default]
    Recreate,
    BlueGreen,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitSource {
    pub url: String,
    pub branch: String,
    pub build_kind: BuildKind,
    pub build_path: String,
    pub build_context: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Source {
    Git(GitSource),
    Image(String),
}

#[derive(Debug, Clone)]
pub struct Application {
    pub id: Id,
    pub environment_id: Id,
    pub project_id: Id,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub source: Source,
    pub deploy_key: Option<String>,
    pub strategy: Strategy,
    pub compose_trusted: bool,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug, Clone)]
pub struct GitSourceInput {
    pub url: String,
    pub branch: Option<String>,
    pub build_kind: BuildKind,
    pub build_path: Option<String>,
    pub build_context: Option<String>,
}

#[derive(Debug, Clone)]
pub enum SourceInput {
    Git(GitSourceInput),
    Image(String),
}

#[derive(Debug)]
pub struct NewApplicationInput {
    pub name: String,
    pub description: Option<String>,
    pub source: SourceInput,
    pub strategy: Option<Strategy>,
}

#[derive(Debug, Default)]
pub struct ApplicationUpdate {
    pub name: Option<String>,
    pub description: Option<String>,
    pub source: Option<SourceInput>,
    pub strategy: Option<Strategy>,
}
