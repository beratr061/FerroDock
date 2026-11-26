use std::time::Duration;

use ferrodock_core::AppResult;
use ferrodock_infra::docker::{WorkloadContainer, WorkloadSpec};

use super::artifact::Job;

const READINESS_TIMEOUT: Duration = Duration::from_secs(180);

impl Job<'_> {
    pub async fn release(&self, image: &str) -> AppResult<()> {
        let docker = &self.ctx.docker;
        docker.ensure_network().await?;
        let previous = docker
            .application_workloads(&self.application.id.to_string())
            .await?;
        let recreate = self.application.strategy != "blue_green";
        if recreate {
            self.stop_previous(&previous).await?;
        }
        let spec = self.workload_spec(image);
        self.log
            .step(&format!("Starting container {}", spec.name))
            .await;
        let outcome = match self.start_and_verify(&spec).await {
            Ok(id) => self.switch_or_discard(&id, &spec.name).await,
            Err(error) => Err(error),
        };
        match outcome {
            Ok(()) => {
                self.retire(&previous).await;
                Ok(())
            }
            Err(error) => {
                if recreate {
                    self.restore(&previous).await;
                }
                Err(error)
            }
        }
    }

    fn workload_spec(&self, image: &str) -> WorkloadSpec {
        WorkloadSpec {
            name: self.names.container(),
            image: image.to_owned(),
            application_id: self.application.id.to_string(),
            deployment_id: self.deployment.id.to_string(),
            alias: self.names.alias(),
            env: self
                .variables
                .iter()
                .map(|(key, value)| format!("{key}={value}"))
                .collect(),
        }
    }

    async fn switch_or_discard(&self, id: &str, name: &str) -> AppResult<()> {
        let switched = self.switch_traffic(name).await;
        if switched.is_err() {
            let _ = self.ctx.docker.remove_workload(id).await;
        }
        switched
    }

    async fn stop_previous(&self, previous: &[WorkloadContainer]) -> AppResult<()> {
        for container in previous.iter().filter(|container| container.running) {
            self.log.step("Stopping previous container").await;
            self.ctx.docker.stop_workload(&container.id).await?;
        }
        Ok(())
    }

    async fn start_and_verify(&self, spec: &WorkloadSpec) -> AppResult<String> {
        let docker = &self.ctx.docker;
        let port = self.probe_port().await;
        let id = docker.run_workload(spec).await?;
        self.log
            .step("Waiting for the container to become ready")
            .await;
        let ready = docker
            .wait_ready(&id, port, READINESS_TIMEOUT, self.cancel)
            .await;
        if ready.is_err() && !self.cancel.is_cancelled() {
            self.show_container_output(&id).await;
        }
        if ready.is_err() {
            let _ = docker.remove_workload(&id).await;
        }
        ready.map(|()| id)
    }

    async fn show_container_output(&self, id: &str) {
        self.log
            .step("Container did not become ready; last output:")
            .await;
        for line in self.ctx.docker.recent_output(id).await {
            self.log.line(&line).await;
        }
    }

    async fn retire(&self, previous: &[WorkloadContainer]) {
        for container in previous {
            self.log.step("Removing previous container").await;
            if container.running {
                let _ = self.ctx.docker.stop_workload(&container.id).await;
            }
            if let Err(error) = self.ctx.docker.remove_workload(&container.id).await {
                tracing::warn!(code = error.code(), "previous container not removed");
            }
        }
    }

    async fn restore(&self, previous: &[WorkloadContainer]) {
        for container in previous.iter().filter(|container| container.running) {
            self.log.step("Restarting previous container").await;
            if let Err(error) = self.ctx.docker.start_workload(&container.id).await {
                tracing::warn!(code = error.code(), "previous container not restarted");
            }
        }
    }
}
