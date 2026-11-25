use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use ferrodock_core::config::AppConfig;
use ferrodock_core::{AppError, AppResult};
use ferrodock_domain::auth::AuthService;
use tokio::net::TcpListener;

use tokio_util::sync::CancellationToken;

use crate::{backups, container_events, jobs, wiring};

const SESSION_PURGE_INTERVAL: Duration = Duration::from_secs(3600);

pub fn print_openapi() -> AppResult<()> {
    let spec = ferrodock_api::openapi_json().map_err(AppError::internal)?;
    let mut stdout = std::io::stdout().lock();
    std::io::Write::write_all(&mut stdout, spec.as_bytes()).map_err(AppError::internal)
}

pub async fn run(config: AppConfig) -> AppResult<()> {
    let wired = wiring::build(&config).await?;
    spawn_session_purger(wired.state.auth.clone());
    let background = CancellationToken::new();
    jobs::start(
        &wired.store,
        wired.deploy.clone(),
        wired.backup.clone(),
        config.job_concurrency,
        &background,
    );
    backups::schedule(&config, Arc::new(wired.store.clone()), background.clone());
    container_events::forward(
        wired.state.docker.clone(),
        wired.state.hub.clone(),
        background.clone(),
    );
    let listener = TcpListener::bind(config.bind_addr)
        .await
        .map_err(AppError::internal)?;
    let served = serve_http(listener, wired.state).await;
    background.cancel();
    wired.store.close().await;
    served?;
    Ok(())
}

async fn serve_http(listener: TcpListener, state: ferrodock_api::AppState) -> AppResult<()> {
    let address = listener.local_addr().map_err(AppError::internal)?;
    tracing::info!(%address, "ferrodock is listening");
    let app = ferrodock_api::router(state).into_make_service_with_connect_info::<SocketAddr>();
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(AppError::internal)
}

fn spawn_session_purger(auth: AuthService) {
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(SESSION_PURGE_INTERVAL);
        loop {
            ticker.tick().await;
            purge_auth_state(&auth).await;
        }
    });
}

async fn purge_auth_state(auth: &AuthService) {
    log_purge("sessions", auth.purge_expired_sessions().await);
    log_purge("login_attempts", auth.purge_stale_login_attempts().await);
}

fn log_purge(kind: &'static str, result: ferrodock_core::AppResult<u64>) {
    if let Err(error) = result {
        tracing::warn!(kind, code = error.code(), "auth purge failed");
    }
}

async fn shutdown_signal() {
    let interrupt = async {
        if let Err(error) = tokio::signal::ctrl_c().await {
            tracing::warn!(%error, "ctrl-c handler failed");
        }
    };
    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut signal) => {
                signal.recv().await;
            }
            Err(error) => tracing::warn!(%error, "SIGTERM handler failed"),
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! {
        () = interrupt => {},
        () = terminate => {},
    }
    tracing::info!("shutdown signal received");
}
