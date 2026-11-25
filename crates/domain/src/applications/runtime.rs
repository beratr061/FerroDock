use ferrodock_core::AppResult;
use ferrodock_db::applications::{ApplicationRecord, SourceRecord};
use ferrodock_infra::docker::{
    DockerEngine, application_key, compose_down, compose_project, network_alias,
};
use ferrodock_infra::proxy::ProxyRoutes;

#[derive(Debug, Clone)]
pub struct Runtime {
    pub docker: DockerEngine,
    pub proxy: ProxyRoutes,
}

fn uses_compose(record: &ApplicationRecord) -> bool {
    matches!(&record.source, SourceRecord::Git(git) if git.build_kind == "compose")
}

impl Runtime {
    pub(super) async fn teardown(&self, record: &ApplicationRecord) -> AppResult<()> {
        let id = record.id;
        for container in self.docker.application_workloads(&id.to_string()).await? {
            self.docker.remove_workload(&container.id).await?;
        }
        if uses_compose(record) {
            compose_down(&compose_project(id)).await?;
        }
        self.proxy
            .publish(&application_key(id), &network_alias(id), &[])
            .await
    }
}
