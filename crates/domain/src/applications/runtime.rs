use ferrodock_core::AppResult;
use ferrodock_db::applications::ApplicationRecord;
use ferrodock_infra::docker::{
    APPLICATION_LABEL, DockerEngine, application_key, compose_down, compose_project,
};
use ferrodock_infra::proxy::ProxyRoutes;

use crate::domains::is_compose;

#[derive(Debug, Clone)]
pub struct Runtime {
    pub docker: DockerEngine,
    pub proxy: ProxyRoutes,
}

impl Runtime {
    pub(super) async fn teardown(
        &self,
        record: &ApplicationRecord,
        remove_data: bool,
    ) -> AppResult<()> {
        let id = record.id;
        for container in self.docker.application_workloads(&id.to_string()).await? {
            self.docker.remove_workload(&container.id).await?;
        }
        if is_compose(record) {
            compose_down(&compose_project(id), remove_data).await?;
        }
        if remove_data {
            let owner = id.to_string();
            for volume in self
                .docker
                .labelled_volumes((APPLICATION_LABEL, &owner))
                .await?
            {
                self.docker.remove_volume(&volume).await?;
            }
        }
        self.proxy.publish(&application_key(id), &[]).await
    }
}
