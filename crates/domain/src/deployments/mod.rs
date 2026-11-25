mod service;

pub use ferrodock_db::deployments::{DeploymentKind, DeploymentRecord as Deployment};
pub use ferrodock_db::jobs::JobStatus as DeploymentStatus;
pub use ferrodock_infra::deploy_log::LogChunk;
pub use service::{DeploymentRepos, DeploymentService};
