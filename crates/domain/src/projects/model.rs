use ferrodock_core::Id;
use ferrodock_db::projects::{EnvironmentRecord, ProjectRecord};
use time::OffsetDateTime;

#[derive(Debug, Clone)]
pub struct Environment {
    pub id: Id,
    pub name: String,
    pub slug: String,
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Clone)]
pub struct Project {
    pub id: Id,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub environments: Vec<Environment>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

#[derive(Debug)]
pub struct NewProjectInput {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Default)]
pub struct ProjectUpdate {
    pub name: Option<String>,
    pub description: Option<String>,
}

impl From<EnvironmentRecord> for Environment {
    fn from(record: EnvironmentRecord) -> Self {
        Self {
            id: record.id,
            name: record.name,
            slug: record.slug,
            created_at: record.created_at,
        }
    }
}

impl Project {
    pub(super) fn assemble(record: ProjectRecord, environments: Vec<EnvironmentRecord>) -> Self {
        Self {
            id: record.id,
            name: record.name,
            slug: record.slug,
            description: record.description,
            environments: environments.into_iter().map(Environment::from).collect(),
            created_at: record.created_at,
            updated_at: record.updated_at,
        }
    }
}
