use std::path::{Path, PathBuf};

use ferrodock_core::Id;
use serde_json::{Value, json};

use super::audit_compose;

fn repository() -> PathBuf {
    let root = std::env::temp_dir().join(format!("fd-compose-{}", Id::new()));
    let _ = std::fs::create_dir_all(root.join("conf"));
    let _ = std::fs::write(root.join("app.env"), "MODE=prod\n");
    let _ = std::fs::write(root.join("Dockerfile"), "FROM busybox\n");
    root
}

fn inside(root: &Path, name: &str) -> String {
    root.join(name).to_string_lossy().into_owned()
}

fn benign(root: &Path) -> Value {
    json!({
        "name": "fd-abc",
        "services": {
            "web": {
                "image": "nginx:1.29-alpine",
                "build": { "context": inside(root, "."), "dockerfile": "Dockerfile" },
                "env_file": [{ "path": inside(root, "app.env"), "required": true }],
                "environment": { "MODE": "prod" },
                "ports": [{ "target": 80, "published": "8080", "protocol": "tcp" }],
                "volumes": [
                    { "type": "bind", "source": inside(root, "conf"), "target": "/etc/nginx/conf.d", "read_only": true },
                    { "type": "volume", "source": "data", "target": "/data" }
                ],
                "labels": { "team": "shop" },
                "security_opt": ["no-new-privileges:true"],
                "cap_drop": ["ALL"],
                "networks": { "default": null }
            }
        },
        "volumes": { "data": { "name": "fd-abc_data" } },
        "networks": { "default": { "name": "fd-abc_default" } }
    })
}

fn rejected_option(root: &Path, config: &Value) -> Option<String> {
    audit_compose(&config.to_string(), root)
        .err()
        .and_then(|error| {
            error
                .params()
                .get("option")
                .and_then(Value::as_str)
                .map(str::to_owned)
        })
}

fn with_service(root: &Path, key: &str, value: Value) -> Value {
    let mut config = benign(root);
    config["services"]["web"][key] = value;
    config
}

#[test]
fn a_regular_stack_passes() {
    let root = repository();
    assert!(audit_compose(&benign(&root).to_string(), &root).is_ok());
}

fn assert_refused(cases: &[(&str, Value, &str)]) {
    let root = repository();
    for (key, value, option) in cases {
        let config = with_service(&root, key, value.clone());
        assert_eq!(
            rejected_option(&root, &config).as_deref(),
            Some(*option),
            "{key}"
        );
    }
}

#[test]
fn host_level_options_are_refused() {
    assert_refused(&[
        ("privileged", json!(true), "privileged"),
        ("cap_add", json!(["SYS_ADMIN"]), "cap_add"),
        ("devices", json!(["/dev/sda:/dev/sda"]), "devices"),
        ("pid", json!("host"), "pid"),
        (
            "security_opt",
            json!(["apparmor=unconfined"]),
            "security_opt",
        ),
        ("use_api_socket", json!(true), "use_api_socket"),
    ]);
}

#[test]
fn reaching_other_containers_is_refused() {
    assert_refused(&[
        ("network_mode", json!("host"), "network_mode"),
        (
            "network_mode",
            json!("container:ferrodock-ferrodock-1"),
            "network_mode",
        ),
        (
            "container_name",
            json!("ferrodock-postgres-1"),
            "container_name",
        ),
        (
            "volumes_from",
            json!(["container:ferrodock-ferrodock-1"]),
            "volumes_from",
        ),
        (
            "labels",
            json!({ "ferrodock.application": "other" }),
            "labels.ferrodock.application",
        ),
        (
            "ports",
            json!([{ "target": 80, "published": "440-445" }]),
            "ports",
        ),
    ]);
}

#[test]
fn files_outside_the_repository_are_refused() {
    let root = repository();
    let socket = json!([{ "type": "bind", "source": "/var/run/docker.sock", "target": "/s" }]);
    let escape = json!([{ "type": "bind", "source": inside(&root, "../secrets"), "target": "/s" }]);
    let key = json!([{ "path": "/etc/ferrodock/master.key", "required": true }]);
    let context = json!({ "context": "/etc/ferrodock" });
    assert_eq!(
        rejected_option(&root, &with_service(&root, "volumes", socket)).as_deref(),
        Some("volumes")
    );
    assert_eq!(
        rejected_option(&root, &with_service(&root, "volumes", escape)).as_deref(),
        Some("volumes")
    );
    assert_eq!(
        rejected_option(&root, &with_service(&root, "env_file", key)).as_deref(),
        Some("env_file")
    );
    assert_eq!(
        rejected_option(&root, &with_service(&root, "build", context)).as_deref(),
        Some("build.context")
    );
}

#[test]
fn shared_resources_are_refused() {
    let root = repository();
    let mut external = benign(&root);
    external["volumes"]["data"] = json!({ "external": true, "name": "fd-other-data" });
    assert_eq!(
        rejected_option(&root, &external).as_deref(),
        Some("external")
    );
    let mut host = benign(&root);
    host["volumes"]["data"] =
        json!({ "driver": "local", "driver_opts": { "type": "none", "o": "bind", "device": "/" } });
    assert_eq!(rejected_option(&root, &host).as_deref(), Some("driver"));
    let mut secret = benign(&root);
    secret["secrets"] = json!({ "key": { "file": "/etc/ferrodock/master.key" } });
    assert_eq!(rejected_option(&root, &secret).as_deref(), Some("file"));
}

#[cfg(unix)]
#[test]
fn symlinks_out_of_the_repository_are_refused() {
    let root = repository();
    let _ = std::os::unix::fs::symlink("/etc", root.join("conf/etc"));
    let link = json!([{ "type": "bind", "source": inside(&root, "conf/etc"), "target": "/s" }]);
    assert_eq!(
        rejected_option(&root, &with_service(&root, "volumes", link)).as_deref(),
        Some("volumes")
    );
}
