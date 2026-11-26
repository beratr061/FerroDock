use ferrodock_core::{AppError, AppResult, Id};
use ferrodock_infra::docker::{LogQuery, LogStream};

use super::service::ApplicationService;
use crate::auth::{Principal, Role};

impl ApplicationService {
    pub async fn container_logs(
        &self,
        principal: &Principal,
        id: Id,
        container: &str,
        tail: u32,
    ) -> AppResult<LogStream> {
        principal.require(Role::Developer)?;
        let target = self
            .containers(principal, id)
            .await?
            .into_iter()
            .find(|candidate| candidate.id == container || candidate.name == container)
            .ok_or_else(|| AppError::not_found("CONTAINER_NOT_FOUND"))?;
        self.runtime
            .docker
            .container_logs(&target.id, LogQuery { tail, follow: true })
    }
}
