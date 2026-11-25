use std::collections::HashMap;

use bollard::models::{
    ContainerCreateBody, EndpointSettings, HostConfig, NetworkingConfig, RestartPolicy,
    RestartPolicyNameEnum,
};
use bollard::query_parameters::{
    CreateContainerOptionsBuilder, ListContainersOptionsBuilder, LogsOptionsBuilder,
    RemoveContainerOptionsBuilder, StopContainerOptionsBuilder,
};
use ferrodock_core::AppResult;
use futures_util::StreamExt;

use super::images::WORKLOAD_NETWORK;
use super::{DockerEngine, MANAGED_LABEL, docker_error, is_not_modified};

pub const APPLICATION_LABEL: &str = "ferrodock.application";
pub const DEPLOYMENT_LABEL: &str = "ferrodock.deployment";
const STOP_GRACE_SECONDS: i32 = 15;
const DIAGNOSTIC_LINES: &str = "40";

#[derive(Debug, Clone)]
pub struct WorkloadSpec {
    pub name: String,
    pub image: String,
    pub application_id: String,
    pub deployment_id: String,
    pub alias: String,
}

#[derive(Debug, Clone)]
pub struct WorkloadContainer {
    pub id: String,
    pub deployment_id: Option<String>,
    pub running: bool,
}

fn labels(spec: &WorkloadSpec) -> HashMap<String, String> {
    HashMap::from([
        (MANAGED_LABEL.to_owned(), "true".to_owned()),
        (APPLICATION_LABEL.to_owned(), spec.application_id.clone()),
        (DEPLOYMENT_LABEL.to_owned(), spec.deployment_id.clone()),
    ])
}

fn container_body(spec: &WorkloadSpec) -> ContainerCreateBody {
    let endpoint = EndpointSettings {
        aliases: Some(vec![spec.alias.clone()]),
        ..EndpointSettings::default()
    };
    ContainerCreateBody {
        image: Some(spec.image.clone()),
        labels: Some(labels(spec)),
        host_config: Some(HostConfig {
            restart_policy: Some(RestartPolicy {
                name: Some(RestartPolicyNameEnum::UNLESS_STOPPED),
                maximum_retry_count: None,
            }),
            init: Some(true),
            network_mode: Some(WORKLOAD_NETWORK.to_owned()),
            ..HostConfig::default()
        }),
        networking_config: Some(NetworkingConfig {
            endpoints_config: Some(HashMap::from([(WORKLOAD_NETWORK.to_owned(), endpoint)])),
        }),
        ..ContainerCreateBody::default()
    }
}

impl DockerEngine {
    pub async fn run_workload(&self, spec: &WorkloadSpec) -> AppResult<String> {
        let options = CreateContainerOptionsBuilder::new()
            .name(&spec.name)
            .build();
        let created = self
            .client
            .create_container(Some(options), container_body(spec))
            .await
            .map_err(docker_error)?;
        self.start_workload(&created.id).await?;
        Ok(created.id)
    }

    pub async fn start_workload(&self, id: &str) -> AppResult<()> {
        match self.client.start_container(id, None).await {
            Err(error) if !is_not_modified(&error) => Err(docker_error(error)),
            _ => Ok(()),
        }
    }

    pub async fn stop_workload(&self, id: &str) -> AppResult<()> {
        let options = StopContainerOptionsBuilder::new()
            .t(STOP_GRACE_SECONDS)
            .build();
        match self.client.stop_container(id, Some(options)).await {
            Err(error) if !is_not_modified(&error) => Err(docker_error(error)),
            _ => Ok(()),
        }
    }

    pub async fn remove_workload(&self, id: &str) -> AppResult<()> {
        let options = RemoveContainerOptionsBuilder::new().force(true).build();
        self.client
            .remove_container(id, Some(options))
            .await
            .map_err(docker_error)
    }

    pub async fn application_workloads(
        &self,
        application_id: &str,
    ) -> AppResult<Vec<WorkloadContainer>> {
        let filter = format!("{APPLICATION_LABEL}={application_id}");
        let options = ListContainersOptionsBuilder::new()
            .all(true)
            .filters(&HashMap::from([("label", vec![filter.as_str()])]))
            .build();
        let containers = self
            .client
            .list_containers(Some(options))
            .await
            .map_err(docker_error)?;
        Ok(containers
            .into_iter()
            .map(|summary| WorkloadContainer {
                id: summary.id.unwrap_or_default(),
                deployment_id: summary
                    .labels
                    .and_then(|mut labels| labels.remove(DEPLOYMENT_LABEL)),
                running: summary
                    .state
                    .is_some_and(|state| state.to_string() == "running"),
            })
            .collect())
    }

    pub async fn recent_output(&self, id: &str) -> Vec<String> {
        let options = LogsOptionsBuilder::new()
            .stdout(true)
            .stderr(true)
            .tail(DIAGNOSTIC_LINES)
            .build();
        let mut stream = self.client.logs(id, Some(options));
        let mut lines = Vec::new();
        while let Some(Ok(chunk)) = stream.next().await {
            lines.extend(chunk.to_string().lines().map(str::to_owned));
        }
        lines
    }
}
