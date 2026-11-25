use std::collections::HashMap;
use std::pin::Pin;

use bollard::models::EventMessage;
use bollard::query_parameters::EventsOptionsBuilder;
use ferrodock_core::AppResult;
use futures_util::{Stream, StreamExt};
use serde::Serialize;

use super::{DockerEngine, docker_error};

const TRACKED_ACTIONS: [&str; 7] = [
    "create",
    "start",
    "stop",
    "die",
    "restart",
    "destroy",
    "health_status",
];

#[derive(Debug, Clone, Serialize)]
pub struct ContainerEvent {
    pub id: String,
    pub name: String,
    pub action: String,
    pub time: i64,
}

pub type EventStream = Pin<Box<dyn Stream<Item = AppResult<ContainerEvent>> + Send>>;

fn to_event(message: EventMessage) -> Option<ContainerEvent> {
    let action = message.action?;
    let base_action = action.split(':').next().unwrap_or_default().to_owned();
    if !TRACKED_ACTIONS.contains(&base_action.as_str()) {
        return None;
    }
    let actor = message.actor?;
    let name = actor
        .attributes
        .and_then(|attributes| attributes.get("name").cloned())
        .unwrap_or_default();
    Some(ContainerEvent {
        id: actor.id?,
        name,
        action: base_action,
        time: message.time.unwrap_or_default(),
    })
}

impl DockerEngine {
    pub fn container_events(&self) -> EventStream {
        let filters = HashMap::from([("type".to_owned(), vec!["container".to_owned()])]);
        let options = EventsOptionsBuilder::new().filters(&filters).build();
        let stream = self
            .client
            .events(Some(options))
            .filter_map(|item| async move {
                match item {
                    Ok(message) => to_event(message).map(Ok),
                    Err(error) => Some(Err(docker_error(error))),
                }
            });
        Box::pin(stream)
    }
}
