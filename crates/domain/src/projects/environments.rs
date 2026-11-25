use ferrodock_core::{AppError, AppResult, Id};
use ferrodock_db::projects::{EnvironmentRemoval, NewEnvironment};
use serde_json::json;

use super::model::Environment;
use super::service::{Actor, ProjectService, conflict_as};
use crate::auth::Role;
use crate::text::{clean_name, slugify};

impl ProjectService {
    pub async fn add_environment(
        &self,
        actor: Actor<'_>,
        project_id: Id,
        name: &str,
    ) -> AppResult<Environment> {
        actor.principal.require(Role::Developer)?;
        let project = self.owned(actor.principal, project_id).await?;
        let name = clean_name(name, "ENVIRONMENT_NAME_INVALID")?;
        let environment = NewEnvironment {
            id: Id::new(),
            project_id: project.id,
            slug: slugify(&name),
            name,
        };
        let record = self
            .repo
            .create_environment(environment)
            .await
            .map_err(conflict_as("ENVIRONMENT_EXISTS"))?;
        let metadata = json!({ "project_id": project.id, "slug": record.slug });
        self.record(actor, "environment.created", record.id, metadata)
            .await;
        Ok(record.into())
    }

    pub async fn remove_environment(
        &self,
        actor: Actor<'_>,
        project_id: Id,
        environment_id: Id,
    ) -> AppResult<()> {
        actor.principal.require(Role::Admin)?;
        let project = self.owned(actor.principal, project_id).await?;
        match self
            .repo
            .delete_environment(project.id, environment_id)
            .await?
        {
            EnvironmentRemoval::Removed => {
                let metadata = json!({ "project_id": project.id });
                self.record(actor, "environment.deleted", environment_id, metadata)
                    .await;
                Ok(())
            }
            EnvironmentRemoval::NotFound => Err(AppError::not_found("ENVIRONMENT_NOT_FOUND")),
            EnvironmentRemoval::LastRemaining => Err(AppError::conflict("ENVIRONMENT_LAST")),
        }
    }
}
