mod runner;

use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use ferrodock_core::AppResult;
use ferrodock_db::jobs::JobRecord;
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;

pub use runner::{JobRunner, RunnerConfig};

#[async_trait]
pub trait JobHandler: Send + Sync + 'static {
    fn kind(&self) -> &'static str;
    async fn run(&self, job: &JobRecord, cancel: CancellationToken) -> AppResult<()>;
}

#[derive(Debug, Clone, Default)]
pub struct JobSignal(Arc<Notify>);

impl JobSignal {
    pub fn notify(&self) {
        self.0.notify_one();
    }

    pub async fn wait(&self, timeout: Duration) {
        let _ = tokio::time::timeout(timeout, self.0.notified()).await;
    }
}
