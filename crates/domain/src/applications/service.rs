use std::sync::Arc;

use ferrodock_core::crypto::SecretBox;
use ferrodock_core::{AppError, AppResult, Id};
use ferrodock_db::ApplicationRepo;
use ferrodock_db::applications::{ApplicationFields, ApplicationRecord, NewApplication};
use serde_json::{Value, json};

use super::keys::{needs_deploy_key, new_deploy_key, settle_deploy_key};
use super::model::{Application, ApplicationUpdate, NewApplicationInput};
use super::source::{parse_strategy, to_record, validate_source};
use crate::audit::{Actor, AuditLog};
use crate::auth::{Principal, Role};
use crate::text::{clean_description, clean_name, slugify};

#[derive(Clone)]
pub struct ApplicationService {
    repo: Arc<dyn ApplicationRepo>,
    secrets: SecretBox,
    audit: AuditLog,
}

impl std::fmt::Debug for ApplicationService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ApplicationService")
    }
}

fn already_exists(error: AppError) -> AppError {
    if error.code() == "ALREADY_EXISTS" {
        AppError::conflict("APPLICATION_EXISTS")
    } else {
        error
    }
}

impl ApplicationService {
    pub fn new(repo: Arc<dyn ApplicationRepo>, secrets: SecretBox, audit: AuditLog) -> Self {
        Self {
            repo,
            secrets,
            audit,
        }
    }

    pub async fn list(&self, principal: &Principal, project_id: Id) -> AppResult<Vec<Application>> {
        let records = self
            .repo
            .list_for_project(principal.organization_id, project_id)
            .await?;
        records.into_iter().map(Application::try_from).collect()
    }

    pub async fn get(&self, principal: &Principal, id: Id) -> AppResult<Application> {
        self.owned(principal, id).await?.try_into()
    }

    pub async fn create(
        &self,
        actor: Actor<'_>,
        environment_id: Id,
        input: NewApplicationInput,
    ) -> AppResult<Application> {
        actor.principal.require(Role::Developer)?;
        self.repo
            .environment_project(actor.principal.organization_id, environment_id)
            .await?
            .ok_or_else(|| AppError::not_found("ENVIRONMENT_NOT_FOUND"))?;
        let name = clean_name(&input.name, "APPLICATION_NAME_INVALID")?;
        let slug = slugify(&name);
        let source = validate_source(&input.source)?;
        let fields = ApplicationFields {
            description: clean_description(input.description.as_deref().unwrap_or_default())?,
            deploy_key: settle_deploy_key(&self.secrets, &source, &slug, None)?,
            source: to_record(&source),
            strategy: input.strategy.unwrap_or_default().as_str().to_owned(),
            name,
        };
        let application = NewApplication {
            id: Id::new(),
            environment_id,
            slug,
            fields,
        };
        let record = self
            .repo
            .create(application)
            .await
            .map_err(already_exists)?;
        self.changed(actor, "application.created", &record).await;
        record.try_into()
    }

    pub async fn update(
        &self,
        actor: Actor<'_>,
        id: Id,
        update: ApplicationUpdate,
    ) -> AppResult<Application> {
        actor.principal.require(Role::Developer)?;
        let current = self.owned(actor.principal, id).await?;
        let source = match &update.source {
            Some(input) => validate_source(input)?,
            None => Application::try_from(current.clone())?.source,
        };
        let fields = ApplicationFields {
            name: match update.name {
                Some(name) => clean_name(&name, "APPLICATION_NAME_INVALID")?,
                None => current.name,
            },
            description: match update.description {
                Some(description) => clean_description(&description)?,
                None => current.description,
            },
            deploy_key: settle_deploy_key(
                &self.secrets,
                &source,
                &current.slug,
                current.deploy_key,
            )?,
            source: to_record(&source),
            strategy: match update.strategy {
                Some(strategy) => strategy.as_str().to_owned(),
                None => parse_strategy(&current.strategy)?.as_str().to_owned(),
            },
        };
        let record = self.repo.update(id, fields).await?;
        self.changed(actor, "application.updated", &record).await;
        record.try_into()
    }

    pub async fn rotate_deploy_key(&self, actor: Actor<'_>, id: Id) -> AppResult<Application> {
        actor.principal.require(Role::Developer)?;
        let current = self.owned(actor.principal, id).await?;
        let application = Application::try_from(current.clone())?;
        if !needs_deploy_key(&application.source)? {
            return Err(AppError::validation("DEPLOY_KEY_NOT_APPLICABLE"));
        }
        let fields = ApplicationFields {
            deploy_key: Some(new_deploy_key(&self.secrets, &current.slug)?),
            name: current.name,
            description: current.description,
            source: current.source,
            strategy: current.strategy,
        };
        let record = self.repo.update(id, fields).await?;
        self.changed(actor, "application.deploy_key_rotated", &record)
            .await;
        record.try_into()
    }

    pub async fn delete(&self, actor: Actor<'_>, id: Id) -> AppResult<()> {
        actor.principal.require(Role::Admin)?;
        let current = self.owned(actor.principal, id).await?;
        if !self.repo.delete(id).await? {
            return Err(AppError::not_found("APPLICATION_NOT_FOUND"));
        }
        self.changed(actor, "application.deleted", &current).await;
        Ok(())
    }

    async fn owned(&self, principal: &Principal, id: Id) -> AppResult<ApplicationRecord> {
        self.repo
            .find(principal.organization_id, id)
            .await?
            .ok_or_else(|| AppError::not_found("APPLICATION_NOT_FOUND"))
    }

    async fn changed(&self, actor: Actor<'_>, action: &'static str, record: &ApplicationRecord) {
        let metadata: Value = json!({
            "environment_id": record.environment_id,
            "slug": record.slug,
        });
        self.audit.change(actor, action, record.id, metadata).await;
    }
}
