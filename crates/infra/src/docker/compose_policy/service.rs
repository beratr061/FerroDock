use std::path::Path;

use ferrodock_core::AppResult;
use serde_json::{Map, Value};

use super::allowed::{BUILD_KEYS, RESERVED_LABELS, RESERVED_PORTS, SERVICE_KEYS};
use super::paths::check_path;
use super::{forbidden, object};

fn check_labels(scope: &str, labels: Option<&Value>) -> AppResult<()> {
    let names: Vec<&str> = match labels {
        Some(Value::Object(map)) => map.keys().map(String::as_str).collect(),
        Some(Value::Array(items)) => items.iter().filter_map(Value::as_str).collect(),
        _ => Vec::new(),
    };
    match names.iter().find(|name| {
        RESERVED_LABELS
            .iter()
            .any(|prefix| name.starts_with(prefix))
    }) {
        Some(name) => Err(forbidden(scope, &format!("labels.{name}"))),
        None => Ok(()),
    }
}

fn check_volume(root: &Path, scope: &str, volume: &Value) -> AppResult<()> {
    let mount = object(volume, scope, "volumes")?;
    match mount.get("type").and_then(Value::as_str) {
        Some("volume" | "tmpfs") => Ok(()),
        Some("bind") => check_path(
            root,
            scope,
            "volumes",
            mount.get("source").and_then(Value::as_str),
        )
        .and_then(|()| match mount.get("source") {
            Some(_) => Ok(()),
            None => Err(forbidden(scope, "volumes")),
        }),
        _ => Err(forbidden(scope, "volumes")),
    }
}

fn is_remote(context: &str) -> bool {
    ["https://", "http://", "git@", "git://", "docker-image://"]
        .iter()
        .any(|prefix| context.starts_with(prefix))
}

fn check_build(root: &Path, scope: &str, build: &Value) -> AppResult<()> {
    let build = object(build, scope, "build")?;
    if let Some(key) = build.keys().find(|key| !BUILD_KEYS.contains(&key.as_str())) {
        return Err(forbidden(scope, &format!("build.{key}")));
    }
    let context = build.get("context").and_then(Value::as_str).unwrap_or(".");
    if !is_remote(context) {
        check_path(root, scope, "build.context", Some(context))?;
        let dockerfile = build.get("dockerfile").and_then(Value::as_str);
        let base = root.join(context);
        check_path(
            root,
            scope,
            "build.dockerfile",
            dockerfile
                .map(|file| base.join(file))
                .as_deref()
                .and_then(Path::to_str),
        )?;
    }
    let extra = build.get("additional_contexts").and_then(Value::as_object);
    for value in extra
        .into_iter()
        .flat_map(Map::values)
        .filter_map(Value::as_str)
    {
        if !is_remote(value) {
            check_path(root, scope, "build.additional_contexts", Some(value))?;
        }
    }
    check_labels(scope, build.get("labels"))
}

fn published(port: &Value) -> Vec<u16> {
    let text = match port.get("published") {
        Some(Value::String(text)) => text.clone(),
        Some(Value::Number(number)) => number.to_string(),
        _ => return Vec::new(),
    };
    let mut bounds = text.split('-').filter_map(|part| part.parse::<u16>().ok());
    match (bounds.next(), bounds.next()) {
        (Some(start), Some(end)) if end >= start => (start..=end).collect(),
        (Some(single), _) => vec![single],
        _ => Vec::new(),
    }
}

fn check_modes(scope: &str, service: &Map<String, Value>) -> AppResult<()> {
    if service.get("privileged").and_then(Value::as_bool) == Some(true) {
        return Err(forbidden(scope, "privileged"));
    }
    if let Some(mode) = service.get("network_mode").and_then(Value::as_str)
        && !(mode == "none" || mode == "bridge" || mode.starts_with("service:"))
    {
        return Err(forbidden(scope, "network_mode"));
    }
    let options = service
        .get("security_opt")
        .and_then(Value::as_array)
        .into_iter()
        .flatten();
    if options
        .filter_map(Value::as_str)
        .any(|option| !option.starts_with("no-new-privileges"))
    {
        return Err(forbidden(scope, "security_opt"));
    }
    let shared = service
        .get("volumes_from")
        .and_then(Value::as_array)
        .into_iter()
        .flatten();
    if shared
        .filter_map(Value::as_str)
        .any(|source| source.starts_with("container:"))
    {
        return Err(forbidden(scope, "volumes_from"));
    }
    if service
        .get("oom_score_adj")
        .and_then(Value::as_i64)
        .is_some_and(|score| score < 0)
    {
        return Err(forbidden(scope, "oom_score_adj"));
    }
    Ok(())
}

fn check_hooks(scope: &str, service: &Map<String, Value>) -> AppResult<()> {
    for hook in ["post_start", "pre_stop"] {
        let hooks = service
            .get(hook)
            .and_then(Value::as_array)
            .into_iter()
            .flatten();
        for entry in hooks {
            if entry.get("privileged").and_then(Value::as_bool) == Some(true) {
                return Err(forbidden(scope, hook));
            }
        }
    }
    let devices = service
        .get("deploy")
        .and_then(|deploy| deploy.pointer("/resources/reservations/devices"));
    if devices.is_some_and(|value| !value.is_null()) {
        return Err(forbidden(scope, "deploy.resources.reservations.devices"));
    }
    Ok(())
}

fn check_files(root: &Path, scope: &str, service: &Map<String, Value>) -> AppResult<()> {
    for option in ["env_file", "label_file"] {
        let entries = service
            .get(option)
            .and_then(Value::as_array)
            .into_iter()
            .flatten();
        for entry in entries {
            let path = entry
                .as_str()
                .or_else(|| entry.get("path").and_then(Value::as_str));
            check_path(root, scope, option, path)?;
        }
    }
    Ok(())
}

pub(super) fn check_service(root: &Path, scope: &str, service: &Value) -> AppResult<()> {
    let service = object(service, scope, "service")?;
    if let Some(key) = service
        .keys()
        .find(|key| !SERVICE_KEYS.contains(&key.as_str()))
    {
        return Err(forbidden(scope, key));
    }
    check_modes(scope, service)?;
    check_hooks(scope, service)?;
    check_files(root, scope, service)?;
    check_labels(scope, service.get("labels"))?;
    if let Some(build) = service.get("build") {
        check_build(root, scope, build)?;
    }
    for volume in service
        .get("volumes")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        check_volume(root, scope, volume)?;
    }
    let ports = service
        .get("ports")
        .and_then(Value::as_array)
        .into_iter()
        .flatten();
    if ports
        .flat_map(published)
        .any(|port| RESERVED_PORTS.contains(&port))
    {
        return Err(forbidden(scope, "ports"));
    }
    Ok(())
}
