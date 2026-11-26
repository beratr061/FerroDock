mod model;
mod policies;
mod runs;
mod schedule;
mod service;
mod storage;
mod targets;
mod validate;

pub use ferrodock_db::backups::BackupTarget;
pub use model::{
    BackupRun, Destination, NewDestinationInput, Policy, PolicyInput, S3Details, S3Input,
};
pub use service::{ScheduledBackupParts, ScheduledBackupService};
pub use storage::open_destination;
pub use targets::storage_scope;
pub use validate::MAX_RETENTION;
