use ferrodock_core::{AppError, AppResult, Id};
use serde_json::{Map, Value, json};

use super::MANAGED_LABEL;
use super::images::WORKLOAD_NETWORK;
use super::naming::compose_service_alias;
use super::workloads::APPLICATION_LABEL;

pub const OVERRIDE_FILE: &str = ".ferrodock.override.yml";

fn invalid(error: serde_json::Error) -> AppError {
    AppError::validation("COMPOSE_CONFIG_INVALID").with_source(error)
}

fn attached_networks(application_id: Id, name: &str, service: &Value) -> Map<String, Value> {
    let mut networks: Map<String, Value> = service
        .get("networks")
        .and_then(Value::as_object)
        .map(|existing| {
            existing
                .keys()
                .map(|key| (key.clone(), json!({})))
                .collect()
        })
        .unwrap_or_default();
    if networks.is_empty() {
        networks.insert("default".to_owned(), json!({}));
    }
    let alias = compose_service_alias(application_id, name);
    networks.insert(WORKLOAD_NETWORK.to_owned(), json!({ "aliases": [alias] }));
    networks
}

fn attachment(application_id: Id, name: &str, service: &Value) -> Value {
    json!({
        "labels": {
            MANAGED_LABEL: "true",
            APPLICATION_LABEL: application_id.to_string(),
        },
        "networks": attached_networks(application_id, name, service),
    })
}

pub fn render_override(rendered_config: &str, application_id: Id) -> AppResult<String> {
    let config: Value = serde_json::from_str(rendered_config).map_err(invalid)?;
    let services = config
        .get("services")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    let attached: Map<String, Value> = services
        .iter()
        .filter(|(_, service)| service.get("network_mode").is_none())
        .map(|(name, service)| (name.clone(), attachment(application_id, name, service)))
        .collect();
    let document = json!({
        "services": attached,
        "networks": { WORKLOAD_NETWORK: { "external": true, "name": WORKLOAD_NETWORK } },
    });
    serde_json::to_string_pretty(&document).map_err(invalid)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rendered(id: Id) -> Value {
        let config = json!({
            "services": {
                "web": { "image": "nginx", "networks": { "default": null } },
                "worker": { "image": "busybox", "networks": { "backend": { "aliases": ["jobs"] } } },
                "sidecar": { "image": "busybox", "network_mode": "service:web" }
            }
        });
        let output = render_override(&config.to_string(), id).unwrap_or_default();
        serde_json::from_str(&output).unwrap_or_default()
    }

    #[test]
    fn joins_services_to_the_workload_network_with_stable_aliases() {
        let id = Id::new();
        let document = rendered(id);
        let web = &document["services"]["web"];
        assert_eq!(web["labels"][APPLICATION_LABEL], id.to_string());
        assert!(web["networks"]["default"].is_object());
        let alias = compose_service_alias(id, "web");
        assert_eq!(web["networks"][WORKLOAD_NETWORK]["aliases"][0], alias);
        let worker = &document["services"]["worker"]["networks"];
        assert!(worker["backend"].is_object());
        assert!(worker.get("default").is_none());
        assert!(document["services"].get("sidecar").is_none());
        assert_eq!(document["networks"][WORKLOAD_NETWORK]["external"], true);
    }

    #[test]
    fn rejects_output_that_is_not_json() {
        assert!(render_override("services: {}", Id::new()).is_err());
    }
}
