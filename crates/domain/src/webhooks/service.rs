use std::sync::Arc;

use ferrodock_core::crypto::{SecretBox, hex, random_bytes};
use ferrodock_core::{AppError, AppResult, Id};
use ferrodock_db::applications::SourceRecord;
use ferrodock_db::webhooks::{WebhookRecord, WebhookTarget};
use ferrodock_db::{ApplicationRepo, WebhookRepo};
use serde_json::json;
use time::OffsetDateTime;

use super::delivery::{Delivery, Event, Push};
use crate::audit::{Actor, AuditLog};
use crate::auth::{Principal, Role};
use crate::deployments::DeploymentService;

#[derive(Debug, Clone)]
pub struct Webhook {
    pub created_at: OffsetDateTime,
    pub last_delivery_at: Option<OffsetDateTime>,
    pub last_outcome: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    Deployed(Id),
    Ignored(&'static str),
}

impl Outcome {
    fn label(&self) -> &'static str {
        match self {
            Self::Deployed(_) => "deployed",
            Self::Ignored(reason) => reason,
        }
    }
}

#[derive(Clone)]
pub struct WebhookService {
    webhooks: Arc<dyn WebhookRepo>,
    applications: Arc<dyn ApplicationRepo>,
    deployments: DeploymentService,
    secrets: SecretBox,
    audit: AuditLog,
}

impl std::fmt::Debug for WebhookService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("WebhookService")
    }
}

fn present(record: WebhookRecord) -> Webhook {
    Webhook {
        created_at: record.created_at,
        last_delivery_at: record.last_delivery_at,
        last_outcome: record.last_outcome,
    }
}

fn rejected() -> AppError {
    AppError::unauthorized()
}

impl WebhookService {
    pub fn new(
        repos: (Arc<dyn WebhookRepo>, Arc<dyn ApplicationRepo>),
        deployments: DeploymentService,
        secrets: SecretBox,
        audit: AuditLog,
    ) -> Self {
        let (webhooks, applications) = repos;
        Self {
            webhooks,
            applications,
            deployments,
            secrets,
            audit,
        }
    }

    pub async fn get(
        &self,
        principal: &Principal,
        application_id: Id,
    ) -> AppResult<Option<Webhook>> {
        self.git_application(principal, application_id).await?;
        Ok(self.webhooks.find(application_id).await?.map(present))
    }

    pub async fn rotate(
        &self,
        actor: Actor<'_>,
        application_id: Id,
    ) -> AppResult<(Webhook, String)> {
        actor.principal.require(Role::Developer)?;
        self.git_application(actor.principal, application_id)
            .await?;
        let secret = hex(&random_bytes::<32>());
        let sealed = self.secrets.encrypt(secret.as_bytes())?;
        let record = self.webhooks.save(application_id, &sealed).await?;
        self.audit
            .change(actor, "webhook.secret_created", application_id, json!({}))
            .await;
        Ok((present(record), secret))
    }

    pub async fn disable(&self, actor: Actor<'_>, application_id: Id) -> AppResult<()> {
        actor.principal.require(Role::Developer)?;
        self.git_application(actor.principal, application_id)
            .await?;
        if self.webhooks.remove(application_id).await? {
            self.audit
                .change(actor, "webhook.disabled", application_id, json!({}))
                .await;
        }
        Ok(())
    }

    pub async fn receive(&self, application_id: Id, delivery: Delivery<'_>) -> AppResult<Outcome> {
        let target = self
            .webhooks
            .target(application_id)
            .await?
            .ok_or_else(rejected)?;
        let secret = self.secrets.decrypt(&target.secret_enc)?;
        let secret = std::str::from_utf8(&secret).map_err(|_| rejected())?;
        if !delivery.is_signed_with(secret) {
            let _ = self
                .webhooks
                .record_delivery(application_id, "rejected")
                .await;
            return Err(rejected());
        }
        let outcome = self.act(&target, &delivery).await?;
        self.webhooks
            .record_delivery(application_id, outcome.label())
            .await?;
        Ok(outcome)
    }

    async fn act(&self, target: &WebhookTarget, delivery: &Delivery<'_>) -> AppResult<Outcome> {
        match delivery.event() {
            Event::Ping => return Ok(Outcome::Ignored("ping")),
            Event::Other => return Ok(Outcome::Ignored("ignored_event")),
            Event::Push => {}
        }
        let (name, commit) = match delivery.push() {
            Some(Push::Branch { name, commit }) => (name, commit),
            Some(Push::Deleted) => return Ok(Outcome::Ignored("ignored_deleted")),
            Some(Push::Tag) => return Ok(Outcome::Ignored("ignored_tag")),
            None => return Err(AppError::validation("WEBHOOK_PAYLOAD_INVALID")),
        };
        if target.git_branch.as_deref() != Some(name.as_str()) {
            return Ok(Outcome::Ignored("ignored_branch"));
        }
        let metadata =
            json!({ "application_id": target.application_id, "branch": name, "commit": commit });
        let record = self
            .deployments
            .deploy_from_webhook(target.organization_id, target.application_id, metadata)
            .await?;
        Ok(Outcome::Deployed(record.id))
    }

    async fn git_application(&self, principal: &Principal, id: Id) -> AppResult<()> {
        let application = self
            .applications
            .find(principal.organization_id, id)
            .await?
            .ok_or_else(|| AppError::not_found("APPLICATION_NOT_FOUND"))?;
        match application.source {
            SourceRecord::Git(_) => Ok(()),
            SourceRecord::Image(_) => Err(AppError::validation("WEBHOOK_NOT_APPLICABLE")),
        }
    }
}
