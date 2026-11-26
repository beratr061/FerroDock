use ferrodock_domain::webhooks::{Outcome, Webhook};
use serde::Serialize;
use time::OffsetDateTime;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, ToSchema)]
pub struct WebhookDto {
    pub enabled: bool,
    #[serde(with = "time::serde::rfc3339::option")]
    #[schema(value_type = Option<String>, format = DateTime)]
    pub created_at: Option<OffsetDateTime>,
    #[serde(with = "time::serde::rfc3339::option")]
    #[schema(value_type = Option<String>, format = DateTime)]
    pub last_delivery_at: Option<OffsetDateTime>,
    pub last_outcome: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct CreatedWebhookDto {
    pub webhook: WebhookDto,
    pub secret: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct WebhookDeliveryDto {
    pub outcome: String,
    pub deployment_id: Option<Uuid>,
}

impl From<Option<Webhook>> for WebhookDto {
    fn from(webhook: Option<Webhook>) -> Self {
        Self {
            enabled: webhook.is_some(),
            created_at: webhook.as_ref().map(|hook| hook.created_at),
            last_delivery_at: webhook.as_ref().and_then(|hook| hook.last_delivery_at),
            last_outcome: webhook.and_then(|hook| hook.last_outcome),
        }
    }
}

impl From<Outcome> for WebhookDeliveryDto {
    fn from(outcome: Outcome) -> Self {
        match outcome {
            Outcome::Deployed(id) => Self {
                outcome: "deployed".to_owned(),
                deployment_id: Some(id.as_uuid()),
            },
            Outcome::Ignored(reason) => Self {
                outcome: reason.to_owned(),
                deployment_id: None,
            },
        }
    }
}
