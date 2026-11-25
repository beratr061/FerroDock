use std::time::{Duration, Instant};

use bollard::models::{ContainerState, ContainerStateStatusEnum, HealthStatusEnum};
use ferrodock_core::{AppError, AppResult};
use tokio_util::sync::CancellationToken;

use super::{DockerEngine, docker_error};
use crate::process::cancelled;

const POLL_INTERVAL: Duration = Duration::from_secs(1);
const STABLE_WITHOUT_HEALTHCHECK: Duration = Duration::from_secs(5);

#[derive(Debug, PartialEq, Eq)]
enum Readiness {
    Ready,
    Waiting,
    Failed(&'static str, i64),
}

fn has_healthcheck(state: &ContainerState) -> bool {
    state
        .health
        .as_ref()
        .and_then(|health| health.status)
        .is_some_and(|status| !matches!(status, HealthStatusEnum::NONE | HealthStatusEnum::EMPTY))
}

fn assess(state: &ContainerState, elapsed: Duration) -> Readiness {
    match state.status {
        Some(ContainerStateStatusEnum::EXITED | ContainerStateStatusEnum::DEAD) => {
            return Readiness::Failed("CONTAINER_EXITED", state.exit_code.unwrap_or(-1));
        }
        Some(ContainerStateStatusEnum::RESTARTING) => {
            return Readiness::Failed("CONTAINER_CRASHING", state.exit_code.unwrap_or(-1));
        }
        _ => {}
    }
    if has_healthcheck(state) {
        return match state.health.as_ref().and_then(|health| health.status) {
            Some(HealthStatusEnum::HEALTHY) => Readiness::Ready,
            Some(HealthStatusEnum::UNHEALTHY) => Readiness::Failed("HEALTHCHECK_FAILED", 0),
            _ => Readiness::Waiting,
        };
    }
    let running = state.status == Some(ContainerStateStatusEnum::RUNNING);
    if running && elapsed >= STABLE_WITHOUT_HEALTHCHECK {
        Readiness::Ready
    } else {
        Readiness::Waiting
    }
}

impl DockerEngine {
    pub async fn wait_ready(
        &self,
        id: &str,
        timeout: Duration,
        cancel: &CancellationToken,
    ) -> AppResult<()> {
        let started = Instant::now();
        loop {
            let details = self
                .client
                .inspect_container(id, None)
                .await
                .map_err(docker_error)?;
            let state = details.state.unwrap_or_default();
            match assess(&state, started.elapsed()) {
                Readiness::Ready => return Ok(()),
                Readiness::Failed(code, exit_code) => {
                    return Err(AppError::validation(code).with_param("exit_code", exit_code));
                }
                Readiness::Waiting if started.elapsed() >= timeout => {
                    return Err(AppError::validation("READINESS_TIMEOUT"));
                }
                Readiness::Waiting => {}
            }
            tokio::select! {
                () = cancel.cancelled() => return Err(cancelled()),
                () = tokio::time::sleep(POLL_INTERVAL) => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use bollard::models::Health;

    use super::*;

    fn state(status: ContainerStateStatusEnum, health: Option<HealthStatusEnum>) -> ContainerState {
        ContainerState {
            status: Some(status),
            health: health.map(|status| Health {
                status: Some(status),
                ..Health::default()
            }),
            exit_code: Some(3),
            ..ContainerState::default()
        }
    }

    #[test]
    fn judges_readiness_from_state() {
        let short = Duration::from_secs(1);
        let long = Duration::from_secs(10);
        let running = ContainerStateStatusEnum::RUNNING;
        assert_eq!(assess(&state(running, None), short), Readiness::Waiting);
        assert_eq!(assess(&state(running, None), long), Readiness::Ready);
        let starting = state(running, Some(HealthStatusEnum::STARTING));
        assert_eq!(assess(&starting, long), Readiness::Waiting);
        let healthy = state(running, Some(HealthStatusEnum::HEALTHY));
        assert_eq!(assess(&healthy, short), Readiness::Ready);
        let exited = state(ContainerStateStatusEnum::EXITED, None);
        assert_eq!(
            assess(&exited, short),
            Readiness::Failed("CONTAINER_EXITED", 3)
        );
    }
}
