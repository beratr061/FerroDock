use async_trait::async_trait;
use ferrodock_core::{AppResult, Id};
use time::OffsetDateTime;

use crate::jobs::{JobStatus, NewJob};

pub const DEPLOYMENT_JOB: &str = "deployment";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeploymentKind {
    Deploy,
    Rollback,
}

impl DeploymentKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Deploy => "deploy",
            Self::Rollback => "rollback",
        }
    }

    pub fn parse(value: &str) -> Self {
        if value == "rollback" {
            Self::Rollback
        } else {
            Self::Deploy
        }
    }
}

#[derive(Debug, Clone)]
pub struct DeploymentRecord {
    pub id: Id,
    pub application_id: Id,
    pub job_id: Id,
    pub kind: DeploymentKind,
    pub rollback_of: Option<Id>,
    pub commit_sha: Option<String>,
    pub image_ref: Option<String>,
    pub image_id: Option<String>,
    pub created_by: Option<Id>,
    pub created_at: OffsetDateTime,
    pub status: JobStatus,
    pub error_code: Option<String>,
    pub started_at: Option<OffsetDateTime>,
    pub finished_at: Option<OffsetDateTime>,
}

#[derive(Debug)]
pub struct NewDeployment {
    pub id: Id,
    pub application_id: Id,
    pub kind: DeploymentKind,
    pub rollback_of: Option<Id>,
    pub created_by: Option<Id>,
    pub job: NewJob,
}

#[derive(Debug, Clone)]
pub struct DeploymentArtifact {
    pub commit_sha: Option<String>,
    pub image_ref: String,
    pub image_id: String,
}

#[async_trait]
pub trait DeploymentRepo: Send + Sync + 'static {
    async fn create(&self, deployment: NewDeployment) -> AppResult<DeploymentRecord>;
    async fn list(&self, application_id: Id, limit: i64) -> AppResult<Vec<DeploymentRecord>>;
    async fn find(&self, id: Id) -> AppResult<Option<DeploymentRecord>>;
    async fn find_in_organization(
        &self,
        organization_id: Id,
        id: Id,
    ) -> AppResult<Option<DeploymentRecord>>;
    async fn record_artifact(&self, id: Id, artifact: DeploymentArtifact) -> AppResult<()>;
    async fn successful(&self, application_id: Id) -> AppResult<Vec<DeploymentRecord>>;
}
