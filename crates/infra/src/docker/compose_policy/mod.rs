mod allowed;
mod paths;
mod service;
#[cfg(test)]
mod tests;

use std::path::Path;

use ferrodock_core::{AppError, AppResult};
use serde_json::{Map, Value};

use self::paths::check_path;
use self::service::check_service;

const TOP_KEYS: &[&str] = &[
    "name", "services", "networks", "volumes", "secrets", "configs",
];

pub(super) fn forbidden(scope: &str, option: &str) -> AppError {
    AppError::validation("COMPOSE_OPTION_FORBIDDEN")
        .with_param("service", scope)
        .with_param("option", option)
}

pub(super) fn object<'a>(
    value: &'a Value,
    scope: &str,
    option: &str,
) -> AppResult<&'a Map<String, Value>> {
    value.as_object().ok_or_else(|| forbidden(scope, option))
}

fn check_resources(root: &Path, kind: &str, resources: Option<&Value>) -> AppResult<()> {
    let Some(resources) = resources.and_then(Value::as_object) else {
        return Ok(());
    };
    for (name, resource) in resources {
        let scope = format!("{kind}.{name}");
        let resource = object(resource, &scope, kind)?;
        if resource.get("external").and_then(Value::as_bool) == Some(true) {
            return Err(forbidden(&scope, "external"));
        }
        let driver = resource.get("driver").and_then(Value::as_str);
        if resource.contains_key("driver_opts")
            || driver.is_some_and(|d| !matches!(d, "local" | "bridge"))
        {
            return Err(forbidden(&scope, "driver"));
        }
        check_path(
            root,
            &scope,
            "file",
            resource.get("file").and_then(Value::as_str),
        )?;
    }
    Ok(())
}

pub fn audit_compose(rendered: &str, directory: &Path) -> AppResult<()> {
    let config: Value = serde_json::from_str(rendered)
        .map_err(|error| AppError::validation("COMPOSE_CONFIG_INVALID").with_source(error))?;
    let config = object(&config, "compose", "document")?;
    if let Some(key) = config
        .keys()
        .find(|key| !TOP_KEYS.contains(&key.as_str()) && !key.starts_with("x-"))
    {
        return Err(forbidden("compose", key));
    }
    let root = std::fs::canonicalize(directory).unwrap_or_else(|_| directory.to_path_buf());
    for kind in ["volumes", "networks", "secrets", "configs"] {
        check_resources(&root, kind, config.get(kind))?;
    }
    let services = config
        .get("services")
        .and_then(Value::as_object)
        .into_iter()
        .flatten();
    for (name, service) in services {
        check_service(&root, name, service)?;
    }
    Ok(())
}
