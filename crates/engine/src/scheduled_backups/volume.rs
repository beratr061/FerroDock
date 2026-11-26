use ferrodock_core::{AppResult, Id};
use ferrodock_infra::docker::VolumeTask;

use super::work::Work;

const HELPER_IMAGE: &str = "busybox:1.37";
const MOUNT_POINT: &str = "/volume";
const ARCHIVE_SCRIPT: &str = "exec tar -czf - -C /volume .";
const EXTRACT_SCRIPT: &str =
    "find /volume -mindepth 1 -maxdepth 1 -exec rm -rf {} + && exec tar -xzf - -C /volume";
const EXTENSION: &str = "tar.gz";

fn task(docker_volume: &str) -> VolumeTask {
    VolumeTask {
        image: HELPER_IMAGE.to_owned(),
        volume: docker_volume.to_owned(),
        target: MOUNT_POINT.to_owned(),
        script: String::new(),
    }
}

impl Work<'_> {
    pub(super) async fn archive_volume(
        &self,
        docker_volume: &str,
    ) -> AppResult<(Vec<u8>, &'static str)> {
        let docker = &self.ctx.docker;
        let helper = docker
            .start_volume_helper(&task(docker_volume), true)
            .await?;
        let archive = docker.dump_with(&helper, ARCHIVE_SCRIPT).await;
        docker.remove_volume_helper(&helper).await;
        Ok((archive?, EXTENSION))
    }

    pub(super) async fn restore_volume(
        &self,
        application_id: Id,
        docker_volume: &str,
        archive: Vec<u8>,
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
        let restored = self.extract(docker_volume, archive).await;
        for id in &running {
            docker.start_workload(id).await?;
        }
        restored
    }

    async fn extract(&self, docker_volume: &str, archive: Vec<u8>) -> AppResult<()> {
        let docker = &self.ctx.docker;
        let helper = docker
            .start_volume_helper(&task(docker_volume), false)
            .await?;
        let extracted = docker.restore_with(&helper, EXTRACT_SCRIPT, archive).await;
        docker.remove_volume_helper(&helper).await;
        extracted
    }
}
