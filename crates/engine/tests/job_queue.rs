use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use ferrodock_core::{AppError, AppResult, Id};
use ferrodock_db::JobQueue;
use ferrodock_db::jobs::{CancelResult, JobOutcome, JobRecord, JobStatus, NewJob};
use ferrodock_db::pg::PgStore;
use ferrodock_engine::jobs::{JobHandler, JobRunner, JobSignal, RunnerConfig};
use sqlx::PgPool;
use tokio_util::sync::CancellationToken;

const QUEUE: &str = "server:local";

struct Handler(&'static str);

#[async_trait]
impl JobHandler for Handler {
    fn kind(&self) -> &'static str {
        self.0
    }

    async fn run(&self, _job: &JobRecord, cancel: CancellationToken) -> AppResult<()> {
        match self.0 {
            "fail" => Err(AppError::validation("BUILD_FAILED")),
            "wait" => {
                cancel.cancelled().await;
                Err(AppError::validation("STOPPED"))
            }
            _ => Ok(()),
        }
    }
}

fn job(kind: &str, group: Option<&str>) -> NewJob {
    NewJob {
        organization_id: None,
        kind: kind.to_owned(),
        queue: QUEUE.to_owned(),
        group_key: group.map(str::to_owned),
        payload: serde_json::json!({}),
    }
}

fn start_runner(store: Arc<PgStore>) -> CancellationToken {
    let config = RunnerConfig {
        queue: QUEUE.to_owned(),
        worker_id: "test".to_owned(),
        concurrency: 2,
        poll_interval: Duration::from_millis(100),
        cancel_check_interval: Duration::from_millis(50),
        cancel_grace: Duration::from_secs(1),
    };
    let runner = ["ok", "fail", "wait"].into_iter().fold(
        JobRunner::new(store, config, JobSignal::default()),
        |runner, kind| runner.register(Arc::new(Handler(kind))),
    );
    let shutdown = CancellationToken::new();
    tokio::spawn(runner.run(shutdown.clone()));
    shutdown
}

async fn settle(store: &PgStore, id: Id, wanted: JobStatus) -> AppResult<JobRecord> {
    for _ in 0..100 {
        if let Some(record) = store.find(id).await?
            && record.status == wanted
        {
            return Ok(record);
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    Err(AppError::validation("JOB_DID_NOT_SETTLE").with_param("wanted", wanted.as_str()))
}

#[sqlx::test(migrator = "ferrodock_db::pg::MIGRATOR")]
async fn claims_respect_concurrency_and_group_order(pool: PgPool) -> AppResult<()> {
    let store = PgStore::from_pool(pool);
    let a1 = store.enqueue(job("ok", Some("app-a"))).await?;
    let a2 = store.enqueue(job("ok", Some("app-a"))).await?;
    let b1 = store.enqueue(job("ok", Some("app-b"))).await?;
    let first = store.claim(QUEUE, "w", 2).await?.map(|j| j.id);
    let second = store.claim(QUEUE, "w", 2).await?.map(|j| j.id);
    assert_eq!((first, second), (Some(a1.id), Some(b1.id)));
    assert!(store.claim(QUEUE, "w", 3).await?.is_none());
    store.complete(a1.id, JobOutcome::Succeeded).await?;
    assert_eq!(store.claim(QUEUE, "w", 2).await?.map(|j| j.id), Some(a2.id));
    Ok(())
}

#[sqlx::test(migrator = "ferrodock_db::pg::MIGRATOR")]
async fn runner_records_outcomes_and_cancellation(pool: PgPool) -> AppResult<()> {
    let store = Arc::new(PgStore::from_pool(pool));
    let queued = store.enqueue(job("ok", Some("blocked"))).await?;
    assert_eq!(
        store.request_cancel(queued.id).await?,
        CancelResult::Cancelled
    );
    let shutdown = start_runner(Arc::clone(&store));
    let ok = store.enqueue(job("ok", None)).await?;
    let failed = store.enqueue(job("fail", None)).await?;
    let unknown = store.enqueue(job("mystery", None)).await?;
    settle(&store, ok.id, JobStatus::Succeeded).await?;
    let failed = settle(&store, failed.id, JobStatus::Failed).await?;
    assert_eq!(failed.error_code.as_deref(), Some("BUILD_FAILED"));
    let unknown = settle(&store, unknown.id, JobStatus::Failed).await?;
    assert_eq!(unknown.error_code.as_deref(), Some("JOB_KIND_UNKNOWN"));
    let waiting = store.enqueue(job("wait", None)).await?;
    settle(&store, waiting.id, JobStatus::Running).await?;
    assert_eq!(
        store.request_cancel(waiting.id).await?,
        CancelResult::Requested
    );
    settle(&store, waiting.id, JobStatus::Cancelled).await?;
    assert_eq!(
        store.request_cancel(ok.id).await?,
        CancelResult::NotCancellable
    );
    shutdown.cancel();
    Ok(())
}

#[sqlx::test(migrator = "ferrodock_db::pg::MIGRATOR")]
async fn interrupted_jobs_are_failed_on_startup(pool: PgPool) -> AppResult<()> {
    let store = Arc::new(PgStore::from_pool(pool));
    let orphan = store.enqueue(job("ok", None)).await?;
    store.claim(QUEUE, "crashed-worker", 1).await?;
    let shutdown = start_runner(Arc::clone(&store));
    let failed = settle(&store, orphan.id, JobStatus::Failed).await?;
    assert_eq!(failed.error_code.as_deref(), Some("JOB_INTERRUPTED"));
    shutdown.cancel();
    Ok(())
}
