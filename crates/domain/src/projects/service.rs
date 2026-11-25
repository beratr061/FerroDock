use std::collections::HashMap;
use std::sync::Arc;

use ferrodock_core::{AppError, AppResult, Id};
use ferrodock_db::ProjectRepo;
use ferrodock_db::projects::{NewEnvironment, NewProject, ProjectChanges, ProjectRecord};
use serde_json::{Value, json};

use super::model::{NewProjectInput, Project, ProjectUpdate};
use crate::audit::{Actor, AuditLog};
use crate::auth::{Principal, Role};
use crate::text::{clean_description, clean_name, slugify};

pub(super) const DEFAULT_ENVIRONMENT: &str = "production";

#[derive(Clone)]
pub struct ProjectService {
    pub(super) repo: Arc<dyn ProjectRepo>,
    audit: AuditLog,
}

impl std::fmt::Debug for ProjectService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ProjectService")
    }
}

pub(super) fn conflict_as(code: &'static str) -> impl FnOnce(AppError) -> AppError {
    move |error| {
        if error.code() == "ALREADY_EXISTS" {
            AppError::conflict(code)
        } else {
            error
        }
    }
}

impl ProjectService {
    pub fn new(repo: Arc<dyn ProjectRepo>, audit: AuditLog) -> Self {
        Self { repo, audit }
    }

    pub async fn list(&self, principal: &Principal) -> AppResult<Vec<Project>> {
        let records = self.repo.list(principal.organization_id).await?;
        let mut grouped: HashMap<Id, Vec<_>> = HashMap::new();
        for environment in self
            .repo
            .list_environments(principal.organization_id)
            .await?
        {
            grouped
                .entry(environment.project_id)
                .or_default()
                .push(environment);
        }
        Ok(records
            .into_iter()
            .map(|record| {
                let environments = grouped.remove(&record.id).unwrap_or_default();
                Project::assemble(record, environments)
            })
            .collect())
    }

    pub async fn get(&self, principal: &Principal, id: Id) -> AppResult<Project> {
        let record = self.owned(principal, id).await?;
        let environments = self.repo.environments(record.id).await?;
        Ok(Project::assemble(record, environments))
    }

    pub async fn create(&self, actor: Actor<'_>, input: NewProjectInput) -> AppResult<Project> {
        actor.principal.require(Role::Developer)?;
        let name = clean_name(&input.name, "PROJECT_NAME_INVALID")?;
        let description = clean_description(input.description.as_deref().unwrap_or_default())?;
        let id = Id::new();
        let project = NewProject {
            id,
            organization_id: actor.principal.organization_id,
            slug: slugify(&name),
            name,
            description,
            first_environment: NewEnvironment {
                id: Id::new(),
                project_id: id,
                name: DEFAULT_ENVIRONMENT.to_owned(),
                slug: DEFAULT_ENVIRONMENT.to_owned(),
            },
        };
        let (record, environment) = self
            .repo
            .create(project)
            .await
            .map_err(conflict_as("PROJECT_EXISTS"))?;
        self.record(actor, "project.created", id, json!({ "slug": record.slug }))
            .await;
        Ok(Project::assemble(record, vec![environment]))
    }

    pub async fn update(
        &self,
        actor: Actor<'_>,
        id: Id,
        update: ProjectUpdate,
    ) -> AppResult<Project> {
        actor.principal.require(Role::Developer)?;
        let current = self.owned(actor.principal, id).await?;
        let changes = ProjectChanges {
            name: match update.name {
                Some(name) => clean_name(&name, "PROJECT_NAME_INVALID")?,
                None => current.name,
            },
            description: match update.description {
                Some(description) => clean_description(&description)?,
                None => current.description,
            },
        };
        let record = self
            .repo
            .update(actor.principal.organization_id, id, changes)
            .await?
            .ok_or_else(|| AppError::not_found("PROJECT_NOT_FOUND"))?;
        self.record(actor, "project.updated", id, Value::Null).await;
        self.get(actor.principal, record.id).await
    }

    pub async fn delete(&self, actor: Actor<'_>, id: Id) -> AppResult<()> {
        actor.principal.require(Role::Admin)?;
        if !self
            .repo
            .delete(actor.principal.organization_id, id)
            .await?
        {
            return Err(AppError::not_found("PROJECT_NOT_FOUND"));
        }
        self.record(actor, "project.deleted", id, Value::Null).await;
        Ok(())
    }

    pub(super) async fn owned(&self, principal: &Principal, id: Id) -> AppResult<ProjectRecord> {
        self.repo
            .find(principal.organization_id, id)
            .await?
            .ok_or_else(|| AppError::not_found("PROJECT_NOT_FOUND"))
    }

    pub(super) async fn record(
        &self,
        actor: Actor<'_>,
        action: &'static str,
        resource_id: Id,
        metadata: Value,
    ) {
        self.audit
            .change(actor, action, resource_id, metadata)
            .await;
    }
}
