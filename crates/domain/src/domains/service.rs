use std::net::IpAddr;
use std::sync::Arc;

use ferrodock_core::{AppError, AppResult, Id};
use ferrodock_db::domains::{DomainRecord, NewDomain};
use ferrodock_db::{ApplicationRepo, DomainRepo};
use ferrodock_infra::docker::{application_key, network_alias, short_key};
use ferrodock_infra::proxy::{
    DnsReport, ProxyRoutes, Route, check_dns, normalize_host, normalize_path_prefix,
};
use serde_json::json;

use crate::audit::{Actor, AuditLog};
use crate::auth::{Principal, Role};

#[derive(Debug, Clone)]
pub struct NewDomainInput {
    pub host: String,
    pub path_prefix: Option<String>,
    pub container_port: u16,
    pub https: bool,
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
        self.owned(actor.principal, application_id).await?;
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
        };
        let record = self.domains.create(domain).await.map_err(taken)?;
        self.sync(application_id).await?;
        let metadata = json!({ "application_id": application_id, "host": record.host });
        self.audit
            .change(actor, "domain.created", record.id, metadata)
            .await;
        Ok(record)
    }

    pub async fn remove(&self, actor: Actor<'_>, application_id: Id, id: Id) -> AppResult<()> {
        actor.principal.require(Role::Developer)?;
        self.owned(actor.principal, application_id).await?;
        if !self.domains.delete(application_id, id).await? {
            return Err(AppError::not_found("DOMAIN_NOT_FOUND"));
        }
        self.sync(application_id).await?;
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

    async fn sync(&self, application_id: Id) -> AppResult<()> {
        let routes: Vec<Route> = self
            .domains
            .list(application_id)
            .await?
            .into_iter()
            .map(|domain| Route {
                key: short_key(domain.id, 8),
                host: domain.host,
                path_prefix: domain.path_prefix,
                port: domain.container_port,
                https: domain.https,
            })
            .collect();
        let app = application_key(application_id);
        self.proxy
            .publish(&app, &network_alias(application_id), &routes)
            .await
            .map_err(|error| AppError::validation("PROXY_WRITE_FAILED").with_source(error))
    }

    async fn owned(&self, principal: &Principal, id: Id) -> AppResult<()> {
        self.applications
            .find(principal.organization_id, id)
            .await?
            .map(|_| ())
            .ok_or_else(|| AppError::not_found("APPLICATION_NOT_FOUND"))
    }
}
