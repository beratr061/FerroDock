use ferrodock_core::AppResult;
use ferrodock_db::applications::{ApplicationRecord, SourceRecord};
use ferrodock_db::domains::DomainRecord;
use ferrodock_infra::docker::{application_key, compose_service_alias, network_alias, short_key};
use ferrodock_infra::proxy::{ProxyRoutes, Route};

pub fn is_compose(application: &ApplicationRecord) -> bool {
    matches!(&application.source, SourceRecord::Git(git) if git.build_kind == "compose")
}

fn target(application: &ApplicationRecord, domain: &DomainRecord) -> Option<String> {
    if is_compose(application) {
        let service = domain.service.as_deref()?;
        return Some(compose_service_alias(application.id, service));
    }
    Some(
        application
            .active_container
            .clone()
            .unwrap_or_else(|| network_alias(application.id)),
    )
}

pub fn routes_for(application: &ApplicationRecord, domains: Vec<DomainRecord>) -> Vec<Route> {
    domains
        .into_iter()
        .filter_map(|domain| {
            let basic_auth = domain
                .basic_auth_user
                .as_ref()
                .zip(domain.basic_auth_hash.as_ref())
                .map(|(user, hash)| format!("{user}:{hash}"));
            Some(Route {
                basic_auth,
                key: short_key(domain.id, 8),
                target: target(application, &domain)?,
                host: domain.host,
                path_prefix: domain.path_prefix,
                port: domain.container_port,
                https: domain.https,
            })
        })
        .collect()
}

pub async fn publish_routes(
    proxy: &ProxyRoutes,
    application: &ApplicationRecord,
    domains: Vec<DomainRecord>,
) -> AppResult<()> {
    let routes = routes_for(application, domains);
    proxy
        .publish(&application_key(application.id), &routes)
        .await
}
