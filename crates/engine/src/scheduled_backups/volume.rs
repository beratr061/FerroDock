use ferrodock_core::{AppResult, Id};
use ferrodock_infra::docker::VolumeTask;

use super::work::Work;

const HELPER_IMAGE: &str = "busybox:1.37";
const MOUNT_POINT: &str = "/volume";
const ARCHIVE_SCRIPT: &str = "exec tar -czf - -C /volume .";
const EXTRACT_SCRIPT: &str =
    "find /volume -mindepth 1 -maxdepth 1 -exec rm -rf {} + && exec tar -xzf - -C /volume";

fn task(docker_volume: &str) -> VolumeTask {
    VolumeTask {
        image: HELPER_IMAGE.to_owned(),
        volume: docker_volume.to_owned(),
        target: MOUNT_POINT.to_owned(),
        script: String::new(),
    }
}

impl Work<'_> {
    pub(super) async fn archive_volume(&self, docker_volume: &str, key: &str) -> AppResult<i64> {
        let docker = &self.ctx.docker;
        let helper = docker
            .start_volume_helper(&task(docker_volume), true)
            .await?;
        let uploaded = match docker.open_pipe(&helper, ARCHIVE_SCRIPT, false).await {
            Ok(pipe) => self.upload(pipe, key).await,
            Err(error) => Err(error),
        };
        docker.remove_volume_helper(&helper).await;
        uploaded
    }

    pub(super) async fn restore_volume(
        &self,
        application_id: Id,
        docker_volume: &str,
        key: &str,
    ) -> AppResult<()> {
        let docker = &self.ctx.docker;
        let running: Vec<String> = docker
            .application_workloads(&application_id.to_string())
            .await?
            .into_iter()
            .filter(|container| container.running)
            .map(|container| container.id)
            .collect();
        for id in &running {
            docker.stop_workload(id).await?;
        }
        let restored = self.extract(docker_volume, key).await;
        for id in &running {
            docker.start_workload(id).await?;
        }
        restored
    }

    async fn extract(&self, docker_volume: &str, key: &str) -> AppResult<()> {
        let docker = &self.ctx.docker;
        let helper = docker
            .start_volume_helper(&task(docker_volume), false)
            .await?;
        let extracted = match docker.open_pipe(&helper, EXTRACT_SCRIPT, true).await {
            Ok(pipe) => self.download_into(key, pipe).await,
            Err(error) => Err(error),
        };
        docker.remove_volume_helper(&helper).await;
        extracted
    }
}
