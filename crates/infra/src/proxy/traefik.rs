use std::path::PathBuf;

use ferrodock_core::{AppError, AppResult};
use serde_json::{Map, Value, json};

const CERT_RESOLVER: &str = "letsencrypt";
const SECURE_ENTRYPOINT: &str = "websecure";
const PLAIN_ENTRYPOINT: &str = "web";

#[derive(Debug, Clone)]
pub struct Route {
    pub key: String,
    pub host: String,
    pub path_prefix: String,
    pub port: u16,
    pub https: bool,
    pub target: String,
    pub basic_auth: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ProxyRoutes {
    dir: Option<PathBuf>,
}

fn rule(route: &Route) -> String {
    let host = format!("Host(`{}`)", route.host);
    if route.path_prefix == "/" {
        host
    } else {
        format!("{host} && PathPrefix(`{}`)", route.path_prefix)
    }
}

fn auth_middleware(app: &str, route: &Route) -> Option<String> {
    route
        .basic_auth
        .as_ref()
        .map(|_| format!("fd-{app}-{}-auth", route.key))
}

fn insert_routers(routers: &mut Map<String, Value>, app: &str, route: &Route, service: &str) {
    let name = format!("fd-{app}-{}", route.key);
    let redirect = format!("fd-{app}-https");
    let guarded: Vec<String> = auth_middleware(app, route).into_iter().collect();
    if route.https {
        routers.insert(
            name.clone(),
            json!({ "rule": rule(route), "entryPoints": [SECURE_ENTRYPOINT], "middlewares": guarded, "service": service, "tls": { "certResolver": CERT_RESOLVER } }),
        );
        routers.insert(
            format!("{name}-http"),
            json!({ "rule": rule(route), "entryPoints": [PLAIN_ENTRYPOINT], "middlewares": [redirect], "service": service }),
        );
    } else {
        routers.insert(
            name,
            json!({ "rule": rule(route), "entryPoints": [PLAIN_ENTRYPOINT], "middlewares": guarded, "service": service }),
        );
    }
}

fn middlewares(app: &str, entries: &[Route]) -> Map<String, Value> {
    let mut middlewares = Map::new();
    middlewares.insert(
        format!("fd-{app}-https"),
        json!({ "redirectScheme": { "scheme": "https", "permanent": true } }),
    );
    for route in entries {
        if let (Some(name), Some(user)) = (auth_middleware(app, route), &route.basic_auth) {
            middlewares.insert(name, json!({ "basicAuth": { "users": [user] } }));
        }
    }
    middlewares
}

pub fn render(app: &str, entries: &[Route]) -> Value {
    let mut routers = Map::new();
    let mut services = Map::new();
    for route in entries {
        let service = format!("fd-{app}-{}", route.key);
        let url = format!("http://{}:{}", route.target, route.port);
        services.insert(
            service.clone(),
            json!({ "loadBalancer": { "servers": [{ "url": url }] } }),
        );
        insert_routers(&mut routers, app, route, &service);
    }
    json!({ "http": { "routers": routers, "services": services, "middlewares": middlewares(app, entries) } })
}

impl ProxyRoutes {
    pub fn new(dir: Option<PathBuf>) -> Self {
        Self { dir }
    }

    pub fn enabled(&self) -> bool {
        self.dir.is_some()
    }

    pub async fn publish(&self, app: &str, routes: &[Route]) -> AppResult<()> {
        let Some(dir) = &self.dir else { return Ok(()) };
        let target = dir.join(format!("app-{app}.yml"));
        if routes.is_empty() {
            return match tokio::fs::remove_file(&target).await {
                Err(error) if error.kind() != std::io::ErrorKind::NotFound => {
                    Err(AppError::internal(error))
                }
                _ => Ok(()),
            };
        }
        let body = serde_json::to_vec_pretty(&render(app, routes)).map_err(AppError::internal)?;
        let staging = dir.join(format!(".app-{app}.yml.tmp"));
        tokio::fs::write(&staging, body)
            .await
            .map_err(AppError::internal)?;
        tokio::fs::rename(&staging, &target)
            .await
            .map_err(AppError::internal)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn route(https: bool, path: &str) -> Route {
        Route {
            key: "d1".to_owned(),
            host: "app.example.com".to_owned(),
            path_prefix: path.to_owned(),
            port: 8080,
            https,
            target: "web-abc-d1".to_owned(),
            basic_auth: None,
        }
    }

    #[test]
    fn renders_https_with_redirect() {
        let config = render("abc", &[route(true, "/api")]);
        let router = &config["http"]["routers"]["fd-abc-d1"];
        assert_eq!(
            router["rule"],
            "Host(`app.example.com`) && PathPrefix(`/api`)"
        );
        assert_eq!(router["tls"]["certResolver"], "letsencrypt");
        assert_eq!(
            config["http"]["routers"]["fd-abc-d1-http"]["middlewares"][0],
            "fd-abc-https"
        );
        let url = &config["http"]["services"]["fd-abc-d1"]["loadBalancer"]["servers"][0]["url"];
        assert_eq!(url, "http://web-abc-d1:8080");
    }

    #[test]
    fn renders_plain_http_without_tls() {
        let config = render("abc", &[route(false, "/")]);
        let router = &config["http"]["routers"]["fd-abc-d1"];
        assert_eq!(router["entryPoints"][0], "web");
        assert!(router.get("tls").is_none());
    }

    #[test]
    fn guards_both_entrypoints_with_basic_auth() {
        let guarded = Route {
            basic_auth: Some("ops:$2b$10$hash".to_owned()),
            ..route(false, "/")
        };
        let config = render("abc", &[guarded]);
        let router = &config["http"]["routers"]["fd-abc-d1"];
        assert_eq!(router["middlewares"][0], "fd-abc-d1-auth");
        let users = &config["http"]["middlewares"]["fd-abc-d1-auth"]["basicAuth"]["users"];
        assert_eq!(users[0], "ops:$2b$10$hash");
        let open = render("abc", &[route(true, "/")]);
        assert_eq!(
            open["http"]["routers"]["fd-abc-d1"]["middlewares"],
            json!([])
        );
    }
}
