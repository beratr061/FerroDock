use ferrodock_core::AppResult;
use ferrodock_db::runtime_config::MountRecord;
use ferrodock_infra::docker::{APPLICATION_LABEL, MountSource, WorkloadMount, WorkloadPort};

use super::artifact::Job;

#[derive(Debug, Clone, Default)]
pub struct Storage {
    pub mounts: Vec<WorkloadMount>,
    pub ports: Vec<WorkloadPort>,
}

impl Job<'_> {
    pub async fn storage(&self) -> AppResult<Storage> {
        let config = self
            .ctx
            .runtime_configs
            .runtime_config(self.application.id)
            .await?;
        let mut mounts = Vec::with_capacity(config.mounts.len());
        for record in config.mounts {
            mounts.push(self.prepare_mount(record).await?);
        }
        let ports = config
            .ports
            .into_iter()
            .map(|port| WorkloadPort {
                published: port.published,
                target: port.target,
                protocol: port.protocol,
            })
            .collect();
        Ok(Storage { mounts, ports })
    }

    async fn prepare_mount(&self, record: MountRecord) -> AppResult<WorkloadMount> {
        let (kind, source) = if record.kind == "volume" {
            let name = self.names.volume(&record.source);
            let application_id = self.application.id.to_string();
            self.ctx
                .docker
                .ensure_volume(&name, (APPLICATION_LABEL, &application_id))
                .await?;
            (MountSource::Volume, name)
        } else {
            (MountSource::Bind, record.source)
        };
        Ok(WorkloadMount {
            kind,
            source,
            target: record.target,
            read_only: record.read_only,
        })
    }

    pub async fn replaces_running(&self, storage: &Storage) -> bool {
        let blue_green = self.application.strategy == "blue_green";
        if blue_green && !storage.ports.is_empty() {
            self.log
                .step("Published host ports cannot be shared; replacing the running container")
                .await;
        }
        !blue_green || !storage.ports.is_empty()
    }
}
