use std::time::Duration;

use ferrodock_core::{AppError, AppResult};
use ferrodock_db::backups::RunOutcome;
use ferrodock_domain::databases::{RestoreMethod, driver, volume};
use ferrodock_infra::docker::VolumeTask;
use tokio_util::sync::CancellationToken;

use super::work::Work;

const READY_TIMEOUT: Duration = Duration::from_secs(300);

impl Work<'_> {
    pub async fn restore(&self) -> AppResult<RunOutcome> {
        let source_id = self
            .run
            .source_run_id
            .ok_or_else(|| AppError::not_found("BACKUP_NOT_FOUND"))?;
        let source = self
            .ctx
            .runs
            .run(source_id)
            .await?
            .ok_or_else(|| AppError::not_found("BACKUP_NOT_FOUND"))?;
        let key = source
            .object_key
            .ok_or_else(|| AppError::not_found("BACKUP_NOT_FOUND"))?;
        let sealed = self.storage.get(&key).await?;
        let dump = self.ctx.secrets.open_bytes(&sealed)?;
        let container = self.container().await?;
        let engine = driver(&self.database.engine)?;
        match engine.restore_method() {
            RestoreMethod::Stream(script) => {
                self.ctx
                    .docker
                    .restore_with(&container, &script, dump.to_vec())
                    .await?;
            }
            RestoreMethod::ReplaceDataFile { staging, prepare } => {
                self.replace_data_file(&container, staging, &prepare, dump.to_vec())
                    .await?;
            }
        }
        Ok(RunOutcome {
            status: "succeeded",
            object_key: None,
            size_bytes: None,
            error_code: None,
        })
    }

    async fn replace_data_file(
        &self,
        container: &str,
        staging: &str,
        prepare: &str,
        dump: Vec<u8>,
    ) -> AppResult<()> {
        let docker = &self.ctx.docker;
        let engine = driver(&self.database.engine)?;
        let upload = format!("cat > {staging}");
        docker.restore_with(container, &upload, dump).await?;
        docker.stop_workload(container).await?;
        let task = VolumeTask {
            image: format!("{}:{}", engine.image(), self.database.version),
            volume: volume(self.database.id),
            target: engine.data_dir(&self.database.version).to_owned(),
            script: prepare.to_owned(),
        };
        let prepared = docker.run_volume_task(&task).await;
        docker.start_workload(container).await?;
        prepared?;
        docker
            .wait_ready(container, None, READY_TIMEOUT, &CancellationToken::new())
            .await
    }
}
