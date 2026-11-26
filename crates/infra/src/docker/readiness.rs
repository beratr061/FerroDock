use std::time::{Duration, Instant};

use bollard::models::{
    ContainerInspectResponse, ContainerState, ContainerStateStatusEnum, HealthStatusEnum,
};
use ferrodock_core::{AppError, AppResult};
use tokio_util::sync::CancellationToken;

use super::port_probe::{PortState, probe, probe_target};
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

struct PortGate {
    preferred: Option<u16>,
    enabled: bool,
    closed_port: Option<u16>,
}

impl PortGate {
    async fn assess(&mut self, details: &ContainerInspectResponse) -> Readiness {
        let Some(target) = probe_target(details, self.preferred).filter(|_| self.enabled) else {
            return Readiness::Ready;
        };
        match probe(target).await {
            PortState::Open => Readiness::Ready,
            PortState::Closed => {
                self.closed_port = Some(target.port());
                Readiness::Waiting
            }
            PortState::Unreachable => {
                tracing::info!(
                    port = target.port(),
                    "container port not reachable from the panel; skipping the port check"
                );
                self.enabled = false;
                Readiness::Ready
            }
        }
    }

    fn timeout_error(&self) -> AppError {
        match self.closed_port {
            Some(port) => AppError::validation("PORT_NOT_LISTENING").with_param("port", port),
            None => AppError::validation("READINESS_TIMEOUT"),
        }
    }
}

impl DockerEngine {
    pub async fn wait_ready(
        &self,
        id: &str,
        port: Option<u16>,
        timeout: Duration,
        cancel: &CancellationToken,
    ) -> AppResult<()> {
        let started = Instant::now();
        let mut gate = PortGate {
            preferred: port,
            enabled: true,
            closed_port: None,
        };
        loop {
            let details = self
                .client
                .inspect_container(id, None)
                .await
                .map_err(docker_error)?;
            let state = details.state.clone().unwrap_or_default();
            let readiness = match assess(&state, started.elapsed()) {
                Readiness::Ready if !has_healthcheck(&state) => gate.assess(&details).await,
                other => other,
            };
            match readiness {
                Readiness::Ready => return Ok(()),
                Readiness::Failed(code, exit_code) => {
                    return Err(AppError::validation(code).with_param("exit_code", exit_code));
                }
                Readiness::Waiting if started.elapsed() >= timeout => {
                    return Err(gate.timeout_error());
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
