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
    ) -> AppResult<(Vec<u8>, &'static str)> {
        let engine = driver(&database.engine)?;
        let container = self.database_container(database).await?;
        let dump = self
            .ctx
            .docker
            .dump_with(&container, &engine.dump_script())
            .await?;
        Ok((dump, engine.dump_extension()))
    }

    pub(super) async fn restore_database(
        &self,
        database: &DatabaseRecord,
        dump: Vec<u8>,
    ) -> AppResult<()> {
        let container = self.database_container(database).await?;
        match driver(&database.engine)?.restore_method() {
            RestoreMethod::Stream(script) => {
                self.ctx
                    .docker
                    .restore_with(&container, &script, dump)
                    .await
            }
            RestoreMethod::ReplaceDataFile { staging, prepare } => {
                self.replace_data_file(database, &container, (staging, &prepare), dump)
                    .await
            }
        }
    }

    async fn replace_data_file(
        &self,
        database: &DatabaseRecord,
        container: &str,
        (staging, prepare): (&str, &str),
        dump: Vec<u8>,
    ) -> AppResult<()> {
        let docker = &self.ctx.docker;
        let engine = driver(&database.engine)?;
        let upload = format!("cat > {staging}");
        docker.restore_with(container, &upload, dump).await?;
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
