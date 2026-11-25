use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use ferrodock_core::AppResult;
use ferrodock_db::JobQueue;
use ferrodock_db::jobs::{JobOutcome, JobRecord};
use tokio_util::sync::CancellationToken;

use super::{JobHandler, JobSignal};

#[derive(Debug, Clone)]
pub struct RunnerConfig {
    pub queue: String,
    pub worker_id: String,
    pub concurrency: i64,
    pub poll_interval: Duration,
    pub cancel_check_interval: Duration,
    pub cancel_grace: Duration,
}

#[derive(Clone)]
pub struct JobRunner {
    repo: Arc<dyn JobQueue>,
    handlers: Arc<HashMap<&'static str, Arc<dyn JobHandler>>>,
    signal: JobSignal,
    config: Arc<RunnerConfig>,
}

impl std::fmt::Debug for JobRunner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JobRunner")
            .field("config", &self.config)
            .finish_non_exhaustive()
    }
}

impl JobRunner {
    pub fn new(repo: Arc<dyn JobQueue>, config: RunnerConfig, signal: JobSignal) -> Self {
        Self {
            repo,
            handlers: Arc::new(HashMap::new()),
            signal,
            config: Arc::new(config),
        }
    }

    #[must_use]
    pub fn register(mut self, handler: Arc<dyn JobHandler>) -> Self {
        Arc::make_mut(&mut self.handlers).insert(handler.kind(), handler);
        self
    }

    pub async fn run(self, shutdown: CancellationToken) -> AppResult<()> {
        self.recover_interrupted().await?;
        while !shutdown.is_cancelled() {
            self.drain(&shutdown).await;
            tokio::select! {
                () = shutdown.cancelled() => {}
                () = self.signal.wait(self.config.poll_interval) => {}
            }
        }
        Ok(())
    }

    async fn recover_interrupted(&self) -> AppResult<()> {
        let interrupted = self.repo.fail_interrupted(&self.config.queue).await?;
        if interrupted > 0 {
            tracing::warn!(interrupted, queue = %self.config.queue, "marked interrupted jobs as failed");
        }
        Ok(())
    }

    async fn drain(&self, shutdown: &CancellationToken) {
        let config = &self.config;
        loop {
            match self
                .repo
                .claim(&config.queue, &config.worker_id, config.concurrency)
                .await
            {
                Ok(Some(job)) => self.spawn(job, shutdown.child_token()),
                Ok(None) => break,
                Err(error) => {
                    tracing::warn!(code = error.code(), "job claim failed");
                    break;
                }
            }
        }
    }

    fn spawn(&self, job: JobRecord, cancel: CancellationToken) {
        let runner = self.clone();
        tokio::spawn(async move {
            let outcome = runner.execute(&job, cancel).await;
            if let Err(error) = runner.repo.complete(job.id, outcome).await {
                tracing::error!(code = error.code(), job = %job.id, "job completion failed");
            }
            runner.signal.notify();
        });
    }

    async fn execute(&self, job: &JobRecord, cancel: CancellationToken) -> JobOutcome {
        let Some(handler) = self.handlers.get(job.kind.as_str()).cloned() else {
            return JobOutcome::Failed {
                code: "JOB_KIND_UNKNOWN".to_owned(),
            };
        };
        let watcher = self.watch_cancellation(job, cancel.clone());
        let mut work = std::pin::pin!(handler.run(job, cancel.clone()));
        let outcome = tokio::select! {
            result = &mut work => outcome_of(result, &cancel),
            () = cancel.cancelled() => {
                let _ = tokio::time::timeout(self.config.cancel_grace, &mut work).await;
                JobOutcome::Cancelled
            }
        };
        watcher.abort();
        outcome
    }

    fn watch_cancellation(
        &self,
        job: &JobRecord,
        cancel: CancellationToken,
    ) -> tokio::task::JoinHandle<()> {
        let repo = Arc::clone(&self.repo);
        let interval = self.config.cancel_check_interval;
        let id = job.id;
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(interval).await;
                if repo.cancel_requested(id).await.unwrap_or(false) {
                    cancel.cancel();
                    return;
                }
            }
        })
    }
}

fn outcome_of(result: AppResult<()>, cancel: &CancellationToken) -> JobOutcome {
    match result {
        Ok(()) => JobOutcome::Succeeded,
        Err(_) if cancel.is_cancelled() => JobOutcome::Cancelled,
        Err(error) => JobOutcome::Failed {
            code: error.code().to_owned(),
        },
    }
}
