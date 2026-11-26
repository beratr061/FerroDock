use std::net::IpAddr;
use std::sync::Arc;

use ferrodock_core::{AppError, AppResult, Id};
use ferrodock_db::applications::ApplicationRecord;
use ferrodock_db::domains::{DomainRecord, NewDomain};
use ferrodock_db::{ApplicationRepo, DomainRepo};
use ferrodock_infra::proxy::{
    DnsReport, ProxyRoutes, check_dns, normalize_host, normalize_path_prefix,
};
use serde_json::json;

use crate::audit::{Actor, AuditLog};
use crate::auth::{Principal, Role};

use super::routes::{is_compose, publish_routes};

#[derive(Debug, Clone)]
pub struct NewDomainInput {
    pub host: String,
    pub path_prefix: Option<String>,
    pub container_port: u16,
    pub https: bool,
    pub service: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct DomainSettings {
    pub public_ip: Option<IpAddr>,
    pub panel_host: Option<String>,
}

#[derive(Clone)]
pub struct DomainService {
    domains: Arc<dyn DomainRepo>,
    applications: Arc<dyn ApplicationRepo>,
    proxy: ProxyRoutes,
    settings: DomainSettings,
    audit: AuditLog,
}

impl std::fmt::Debug for DomainService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("DomainService")
    }
}

fn compose_service(
    application: &ApplicationRecord,
    service: Option<&str>,
) -> AppResult<Option<String>> {
    let service = service.map(str::trim).filter(|value| !value.is_empty());
    match (is_compose(application), service) {
        (true, Some(name)) if valid_service_name(name) => Ok(Some(name.to_owned())),
        (true, _) => Err(AppError::validation("DOMAIN_SERVICE_REQUIRED")),
        (false, Some(_)) => Err(AppError::validation("DOMAIN_SERVICE_UNSUPPORTED")),
        (false, None) => Ok(None),
    }
}

fn valid_service_name(name: &str) -> bool {
    name.len() <= 63
        && name.starts_with(|c: char| c.is_ascii_alphanumeric())
        && name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '-' | '_' | '.'))
}

fn taken(error: AppError) -> AppError {
    if error.code() == "ALREADY_EXISTS" {
        AppError::conflict("DOMAIN_TAKEN")
    } else {
        error
    }
}

impl DomainService {
    pub fn new(
        repos: (Arc<dyn DomainRepo>, Arc<dyn ApplicationRepo>),
        proxy: ProxyRoutes,
        settings: DomainSettings,
        audit: AuditLog,
    ) -> Self {
        let (domains, applications) = repos;
        Self {
            domains,
            applications,
            proxy,
            settings,
            audit,
        }
    }

    pub fn proxy_enabled(&self) -> bool {
        self.proxy.enabled()
    }

    pub async fn list(
        &self,
        principal: &Principal,
        application_id: Id,
    ) -> AppResult<Vec<DomainRecord>> {
        self.owned(principal, application_id).await?;
        self.domains.list(application_id).await
    }

    pub async fn add(
        &self,
        actor: Actor<'_>,
        application_id: Id,
        input: NewDomainInput,
    ) -> AppResult<DomainRecord> {
        actor.principal.require(Role::Developer)?;
        let application = self.owned(actor.principal, application_id).await?;
        let host = normalize_host(&input.host)?;
        if self.settings.panel_host.as_deref() == Some(host.as_str()) {
            return Err(AppError::conflict("DOMAIN_RESERVED"));
        }
        if input.container_port == 0 {
            return Err(AppError::validation("DOMAIN_PORT_INVALID"));
        }
        let domain = NewDomain {
            id: Id::new(),
            application_id,
            host,
            path_prefix: normalize_path_prefix(input.path_prefix.as_deref())?,
            container_port: input.container_port,
            https: input.https,
            service: compose_service(&application, input.service.as_deref())?,
        };
        let record = self.domains.create(domain).await.map_err(taken)?;
        self.sync(&application).await?;
        let metadata = json!({ "application_id": application_id, "host": record.host });
        self.audit
            .change(actor, "domain.created", record.id, metadata)
            .await;
        Ok(record)
    }

    pub async fn remove(&self, actor: Actor<'_>, application_id: Id, id: Id) -> AppResult<()> {
        actor.principal.require(Role::Developer)?;
        let application = self.owned(actor.principal, application_id).await?;
        if !self.domains.delete(application_id, id).await? {
            return Err(AppError::not_found("DOMAIN_NOT_FOUND"));
        }
        self.sync(&application).await?;
        let metadata = json!({ "application_id": application_id });
        self.audit
            .change(actor, "domain.deleted", id, metadata)
            .await;
        Ok(())
    }

    pub async fn check(
        &self,
        principal: &Principal,
        application_id: Id,
        id: Id,
    ) -> AppResult<DnsReport> {
        let domain = self
            .list(principal, application_id)
            .await?
            .into_iter()
            .find(|domain| domain.id == id)
            .ok_or_else(|| AppError::not_found("DOMAIN_NOT_FOUND"))?;
        Ok(check_dns(&domain.host, self.settings.public_ip).await)
    }

    async fn sync(&self, application: &ApplicationRecord) -> AppResult<()> {
        let domains = self.domains.list(application.id).await?;
        publish_routes(&self.proxy, application, domains)
            .await
            .map_err(|error| AppError::validation("PROXY_WRITE_FAILED").with_source(error))
    }

    async fn owned(&self, principal: &Principal, id: Id) -> AppResult<ApplicationRecord> {
        self.applications
            .find(principal.organization_id, id)
            .await?
            .ok_or_else(|| AppError::not_found("APPLICATION_NOT_FOUND"))
    }
}
