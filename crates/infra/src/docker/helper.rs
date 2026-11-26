use std::collections::HashMap;

use bollard::models::{ContainerCreateBody, HostConfig, Mount, MountType};
use bollard::query_parameters::{
    CreateContainerOptionsBuilder, RemoveContainerOptionsBuilder, WaitContainerOptions,
};
use ferrodock_core::{AppError, AppResult};
use futures_util::StreamExt;

use super::{DockerEngine, MANAGED_LABEL, docker_error};

const IDLE_SCRIPT: &str = "sleep 3600";

#[derive(Debug, Clone)]
pub struct VolumeTask {
    pub image: String,
    pub volume: String,
    pub target: String,
    pub script: String,
}

fn body(task: &VolumeTask) -> ContainerCreateBody {
    let mount = Mount {
        target: Some(task.target.clone()),
        source: Some(task.volume.clone()),
        typ: Some(MountType::VOLUME),
        read_only: Some(false),
        ..Mount::default()
    };
    ContainerCreateBody {
        image: Some(task.image.clone()),
        entrypoint: Some(vec!["sh".to_owned(), "-c".to_owned()]),
        cmd: Some(vec![task.script.clone()]),
        user: Some("0".to_owned()),
        labels: Some(HashMap::from([(
            MANAGED_LABEL.to_owned(),
            "true".to_owned(),
        )])),
        host_config: Some(HostConfig {
            mounts: Some(vec![mount]),
            network_mode: Some("none".to_owned()),
            ..HostConfig::default()
        }),
        ..ContainerCreateBody::default()
    }
}

fn idle_body(task: &VolumeTask, read_only: bool) -> ContainerCreateBody {
    let mut created = body(task);
    created.cmd = Some(vec![IDLE_SCRIPT.to_owned()]);
    if let Some(mount) = created
        .host_config
        .as_mut()
        .and_then(|host| host.mounts.as_mut())
        .and_then(|mounts| mounts.first_mut())
    {
        mount.read_only = Some(read_only);
    }
    created
}

impl DockerEngine {
    pub async fn start_volume_helper(
        &self,
        task: &VolumeTask,
        read_only: bool,
    ) -> AppResult<String> {
        self.ensure_image(&task.image).await?;
        let options = CreateContainerOptionsBuilder::new().build();
        let created = self
            .client
            .create_container(Some(options), idle_body(task, read_only))
            .await
            .map_err(docker_error)?;
        if let Err(error) = self.start_workload(&created.id).await {
            self.remove_volume_helper(&created.id).await;
            return Err(error);
        }
        Ok(created.id)
    }

    pub async fn remove_volume_helper(&self, id: &str) {
        let remove = RemoveContainerOptionsBuilder::new().force(true).build();
        let _ = self.client.remove_container(id, Some(remove)).await;
    }

    pub async fn run_volume_task(&self, task: &VolumeTask) -> AppResult<()> {
        let options = CreateContainerOptionsBuilder::new().build();
        let created = self
            .client
            .create_container(Some(options), body(task))
            .await
            .map_err(docker_error)?;
        let outcome = self.finish_task(&created.id).await;
        let remove = RemoveContainerOptionsBuilder::new().force(true).build();
        let _ = self
            .client
            .remove_container(&created.id, Some(remove))
            .await;
        outcome
    }

    async fn finish_task(&self, id: &str) -> AppResult<()> {
        self.start_workload(id).await?;
        let mut waits = self.client.wait_container(id, None::<WaitContainerOptions>);
        let status = match waits.next().await {
            Some(Ok(response)) => response.status_code,
            Some(Err(bollard::errors::Error::DockerContainerWaitError { code, .. })) => code,
            Some(Err(error)) => return Err(docker_error(error)),
            None => -1,
        };
        if status == 0 {
            Ok(())
        } else {
            Err(AppError::validation("BACKUP_RESTORE_FAILED").with_param("exit_code", status))
        }
    }
}
