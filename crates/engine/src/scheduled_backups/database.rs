use std::time::Duration;

use ferrodock_core::{AppError, AppResult};
use ferrodock_db::databases::DatabaseRecord;
use ferrodock_domain::databases::{RestoreMethod, driver, volume};
use ferrodock_infra::docker::VolumeTask;
use tokio_util::sync::CancellationToken;

use super::work::Work;

const READY_TIMEOUT: Duration = Duration::from_secs(300);

impl Work<'_> {
    async fn database_container(&self, database: &DatabaseRecord) -> AppResult<String> {
        self.ctx
            .docker
            .database_containers(Some(&database.id.to_string()))
            .await?
            .into_iter()
            .find(|container| container.running)
            .map(|container| container.id)
            .ok_or_else(|| AppError::conflict("DATABASE_NOT_RUNNING"))
    }

    pub(super) async fn dump_database(
        &self,
        database: &DatabaseRecord,
        key: &str,
    ) -> AppResult<i64> {
        let engine = driver(&database.engine)?;
        let container = self.database_container(database).await?;
        let pipe = self
            .ctx
            .docker
            .open_pipe(&container, &engine.dump_script(), false)
            .await?;
        self.upload(pipe, key).await
    }

    pub(super) async fn restore_database(
        &self,
        database: &DatabaseRecord,
        key: &str,
    ) -> AppResult<()> {
        let container = self.database_container(database).await?;
        match driver(&database.engine)?.restore_method() {
            RestoreMethod::Stream(script) => {
                let pipe = self.ctx.docker.open_pipe(&container, &script, true).await?;
                self.download_into(key, pipe).await
            }
            RestoreMethod::ReplaceDataFile { staging, prepare } => {
                self.replace_data_file(database, &container, (staging, &prepare), key)
                    .await
            }
        }
    }

    async fn replace_data_file(
        &self,
        database: &DatabaseRecord,
        container: &str,
        (staging, prepare): (&str, &str),
        key: &str,
    ) -> AppResult<()> {
        let docker = &self.ctx.docker;
        let engine = driver(&database.engine)?;
        let upload = format!("cat > {staging}");
        let pipe = docker.open_pipe(container, &upload, true).await?;
        self.download_into(key, pipe).await?;
        docker.stop_workload(container).await?;
        let task = VolumeTask {
            image: format!("{}:{}", engine.image(), database.version),
            volume: volume(database.id),
            target: engine.data_dir(&database.version).to_owned(),
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
