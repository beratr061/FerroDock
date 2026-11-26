use std::collections::HashMap;
use std::time::Duration;

use bollard::models::{
    ContainerCreateBody, ContainerSummary, EndpointSettings, HealthConfig, HostConfig,
    NetworkingConfig, RestartPolicy, RestartPolicyNameEnum,
};
use bollard::query_parameters::{
    CreateContainerOptionsBuilder, ListContainersOptionsBuilder, RemoveVolumeOptionsBuilder,
};
use ferrodock_core::{AppError, AppResult};

use super::images::WORKLOAD_NETWORK;
use super::workload_storage::{
    MountSource, WorkloadMount, WorkloadPort, exposed_ports, with_storage,
};
use super::{DockerEngine, MANAGED_LABEL, docker_error};

pub const DATABASE_LABEL: &str = "ferrodock.database";
const HEALTH_INTERVAL: Duration = Duration::from_secs(5);
const HEALTH_TIMEOUT: Duration = Duration::from_secs(5);
const HEALTH_START_PERIOD: Duration = Duration::from_secs(60);
const HEALTH_RETRIES: i64 = 6;

#[derive(Debug, Clone)]
pub struct DatabaseSpec {
    pub name: String,
    pub image: String,
    pub database_id: String,
    pub alias: String,
    pub env: Vec<String>,
    pub command: Option<Vec<String>>,
    pub healthcheck: String,
    pub volume: String,
    pub data_dir: String,
    pub port: u16,
    pub published: Option<u16>,
    pub memory_bytes: Option<i64>,
    pub nano_cpus: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct DatabaseContainer {
    pub id: String,
    pub database_id: String,
    pub running: bool,
}

fn nanos(duration: Duration) -> i64 {
    i64::try_from(duration.as_nanos()).unwrap_or(i64::MAX)
}

fn healthcheck(spec: &DatabaseSpec) -> HealthConfig {
    HealthConfig {
        test: Some(vec!["CMD-SHELL".to_owned(), spec.healthcheck.clone()]),
        interval: Some(nanos(HEALTH_INTERVAL)),
        timeout: Some(nanos(HEALTH_TIMEOUT)),
        start_period: Some(nanos(HEALTH_START_PERIOD)),
        start_interval: Some(nanos(Duration::from_secs(1))),
        retries: Some(HEALTH_RETRIES),
    }
}

fn storage(spec: &DatabaseSpec) -> (Vec<WorkloadMount>, Vec<WorkloadPort>) {
    let mount = WorkloadMount {
        kind: MountSource::Volume,
        source: spec.volume.clone(),
        target: spec.data_dir.clone(),
        read_only: false,
    };
    let ports = spec.published.map(|published| WorkloadPort {
        published,
        target: spec.port,
        protocol: "tcp".to_owned(),
    });
    (vec![mount], ports.into_iter().collect())
}

fn container_body(spec: &DatabaseSpec) -> ContainerCreateBody {
    let (mounts, ports) = storage(spec);
    let host = HostConfig {
        restart_policy: Some(RestartPolicy {
            name: Some(RestartPolicyNameEnum::UNLESS_STOPPED),
            maximum_retry_count: None,
        }),
        network_mode: Some(WORKLOAD_NETWORK.to_owned()),
        memory: spec.memory_bytes,
        memory_swap: spec.memory_bytes,
        nano_cpus: spec.nano_cpus,
        ..HostConfig::default()
    };
    let endpoint = EndpointSettings {
        aliases: Some(vec![spec.alias.clone()]),
        ..EndpointSettings::default()
    };
    ContainerCreateBody {
        image: Some(spec.image.clone()),
        env: Some(spec.env.clone()),
        cmd: spec.command.clone(),
        labels: Some(HashMap::from([
            (MANAGED_LABEL.to_owned(), "true".to_owned()),
            (DATABASE_LABEL.to_owned(), spec.database_id.clone()),
        ])),
        healthcheck: Some(healthcheck(spec)),
        exposed_ports: exposed_ports(&ports),
        host_config: Some(with_storage(host, &mounts, &ports)),
        networking_config: Some(NetworkingConfig {
            endpoints_config: Some(HashMap::from([(WORKLOAD_NETWORK.to_owned(), endpoint)])),
        }),
        ..ContainerCreateBody::default()
    }
}

fn summary(container: ContainerSummary) -> DatabaseContainer {
    DatabaseContainer {
        id: container.id.unwrap_or_default(),
        database_id: container
            .labels
            .and_then(|mut labels| labels.remove(DATABASE_LABEL))
            .unwrap_or_default(),
        running: container
            .state
            .is_some_and(|state| state.to_string() == "running"),
    }
}

impl DockerEngine {
    pub async fn run_database(&self, spec: &DatabaseSpec) -> AppResult<String> {
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

    pub async fn database_containers(
        &self,
        database_id: Option<&str>,
    ) -> AppResult<Vec<DatabaseContainer>> {
        let filter = database_id.map_or_else(
            || DATABASE_LABEL.to_owned(),
            |id| format!("{DATABASE_LABEL}={id}"),
        );
        let options = ListContainersOptionsBuilder::new()
            .all(true)
            .filters(&HashMap::from([("label", vec![filter.as_str()])]))
            .build();
        let containers = self
            .client
            .list_containers(Some(options))
            .await
            .map_err(docker_error)?;
        Ok(containers.into_iter().map(summary).collect())
    }

    pub async fn remove_volume(&self, name: &str) -> AppResult<()> {
        let options = RemoveVolumeOptionsBuilder::new().force(true).build();
        let Err(error) = self.client.remove_volume(name, Some(options)).await else {
            return Ok(());
        };
        match docker_error(error) {
            missing if missing.code() == "CONTAINER_NOT_FOUND" => Ok(()),
            busy if busy.code() == "CONTAINER_STATE_CONFLICT" => {
                Err(AppError::conflict("VOLUME_IN_USE").with_source(busy))
            }
            other => Err(other),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn database_containers_carry_storage_and_healthcheck() {
        let spec = DatabaseSpec {
            name: "orders-db".to_owned(),
            image: "postgres:17".to_owned(),
            database_id: "id".to_owned(),
            alias: "fd-abc".to_owned(),
            env: vec!["POSTGRES_DB=orders".to_owned()],
            command: None,
            healthcheck: "pg_isready".to_owned(),
            volume: "fd-abc-data".to_owned(),
            data_dir: "/var/lib/postgresql/data".to_owned(),
            port: 5432,
            published: Some(15432),
            memory_bytes: Some(256 * 1024 * 1024),
            nano_cpus: Some(500_000_000),
        };
        let body = container_body(&spec);
        let test = body.healthcheck.and_then(|check| check.test);
        assert_eq!(
            test,
            Some(vec!["CMD-SHELL".to_owned(), "pg_isready".to_owned()])
        );
        let host = body.host_config.unwrap_or_default();
        assert_eq!(host.memory, Some(256 * 1024 * 1024));
        assert_eq!(host.memory_swap, host.memory);
        assert_eq!(host.nano_cpus, Some(500_000_000));
        assert_eq!(host.mounts.unwrap_or_default().len(), 1);
        assert!(
            host.port_bindings
                .is_some_and(|ports| ports.contains_key("5432/tcp"))
        );
    }
}
