use std::sync::Arc;
use std::time::Duration;

use ferrodock_db::jobs::LOCAL_QUEUE;
use ferrodock_db::pg::{JobSignals, PgStore};
use ferrodock_engine::backup::{BackupContext, BackupHandler};
use ferrodock_engine::databases::{DatabaseContext, DatabaseHandler};
use ferrodock_engine::deploy::{DeployContext, DeployHandler};
use ferrodock_engine::jobs::{JobRunner, JobSignal, RunnerConfig};
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;

const RECONNECT_DELAY: Duration = Duration::from_secs(5);
const DRAIN_TIMEOUT: Duration = Duration::from_secs(60);

#[derive(Debug)]
pub struct Contexts {
    pub deploy: DeployContext,
    pub backup: BackupContext,
    pub databases: DatabaseContext,
}

pub fn start(
    store: &PgStore,
    contexts: Contexts,
    concurrency: u32,
    shutdown: &CancellationToken,
) -> JoinHandle<()> {
    let signal = JobSignal::default();
    let config = RunnerConfig {
        queue: LOCAL_QUEUE.to_owned(),
        worker_id: format!("panel-{}", std::process::id()),
        concurrency: i64::from(concurrency.max(1)),
        poll_interval: Duration::from_secs(5),
        cancel_check_interval: Duration::from_secs(2),
        cancel_grace: Duration::from_secs(15),
        drain_timeout: DRAIN_TIMEOUT,
    };
    let runner = JobRunner::new(Arc::new(store.clone()), config, signal.clone())
        .register(Arc::new(DeployHandler::new(contexts.deploy)))
        .register(Arc::new(BackupHandler::new(contexts.backup)))
        .register(Arc::new(DatabaseHandler::new(contexts.databases)));
    tokio::spawn(forward_signals(store.clone(), signal, shutdown.clone()));
    let runner_shutdown = shutdown.clone();
    tokio::spawn(async move {
        if let Err(error) = runner.run(runner_shutdown).await {
            tracing::error!(code = error.code(), "job runner stopped");
        }
    })
}

async fn forward_signals(store: PgStore, signal: JobSignal, shutdown: CancellationToken) {
    while !shutdown.is_cancelled() {
        listen_once(&store, &signal, &shutdown).await;
        pause(&shutdown).await;
    }
}

async fn listen_once(store: &PgStore, signal: &JobSignal, shutdown: &CancellationToken) {
    match store.job_signals().await {
        Ok(signals) => relay(signals, signal, shutdown).await,
        Err(error) => tracing::warn!(code = error.code(), "job signal listener unavailable"),
    }
}

async fn pause(shutdown: &CancellationToken) {
    tokio::select! {
        () = shutdown.cancelled() => {}
        () = tokio::time::sleep(RECONNECT_DELAY) => {}
    }
}

async fn relay(mut signals: JobSignals, signal: &JobSignal, shutdown: &CancellationToken) {
    loop {
        tokio::select! {
            () = shutdown.cancelled() => return,
            next = signals.next() => {
                if next.is_err() {
                    return;
                }
                signal.notify();
            }
        }
    }
}
