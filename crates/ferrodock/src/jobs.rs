use std::sync::Arc;
use std::time::Duration;

use ferrodock_db::pg::{JobSignals, PgStore};
use ferrodock_engine::jobs::{JobRunner, JobSignal, RunnerConfig};
use tokio_util::sync::CancellationToken;

pub const LOCAL_QUEUE: &str = "server:local";
const RECONNECT_DELAY: Duration = Duration::from_secs(5);

pub fn start(store: &PgStore, concurrency: u32, shutdown: &CancellationToken) {
    let signal = JobSignal::default();
    let config = RunnerConfig {
        queue: LOCAL_QUEUE.to_owned(),
        worker_id: format!("panel-{}", std::process::id()),
        concurrency: i64::from(concurrency.max(1)),
        poll_interval: Duration::from_secs(5),
        cancel_check_interval: Duration::from_secs(2),
        cancel_grace: Duration::from_secs(15),
    };
    let runner = JobRunner::new(Arc::new(store.clone()), config, signal.clone());
    let runner_shutdown = shutdown.clone();
    tokio::spawn(async move {
        if let Err(error) = runner.run(runner_shutdown).await {
            tracing::error!(code = error.code(), "job runner stopped");
        }
    });
    tokio::spawn(forward_signals(store.clone(), signal, shutdown.clone()));
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
