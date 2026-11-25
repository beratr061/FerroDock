use ferrodock_domain::domains::{DnsReport, DnsVerdict, Domain};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, ToSchema)]
pub struct DomainDto {
    pub id: Uuid,
    pub host: String,
    pub path_prefix: String,
    pub container_port: u16,
    pub https: bool,
    #[serde(with = "time::serde::rfc3339")]
    #[schema(value_type = String, format = DateTime)]
    pub created_at: OffsetDateTime,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DomainListDto {
    pub proxy_enabled: bool,
    pub domains: Vec<DomainDto>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateDomainRequest {
    pub host: String,
    pub path_prefix: Option<String>,
    pub container_port: u16,
    pub https: Option<bool>,
}

#[derive(Debug, Clone, Copy, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum DnsVerdictDto {
    PointsHere,
    PointsElsewhere,
    Proxied,
    Unresolved,
    Unverified,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DnsCheckDto {
    pub verdict: DnsVerdictDto,
    pub addresses: Vec<String>,
}

impl From<Domain> for DomainDto {
    fn from(domain: Domain) -> Self {
        Self {
            id: domain.id.as_uuid(),
            host: domain.host,
            path_prefix: domain.path_prefix,
            container_port: domain.container_port,
            https: domain.https,
            created_at: domain.created_at,
        }
    }
}

impl From<DnsReport> for DnsCheckDto {
    fn from(report: DnsReport) -> Self {
        Self {
            verdict: match report.verdict {
                DnsVerdict::PointsHere => DnsVerdictDto::PointsHere,
                DnsVerdict::PointsElsewhere => DnsVerdictDto::PointsElsewhere,
                DnsVerdict::Proxied => DnsVerdictDto::Proxied,
                DnsVerdict::Unresolved => DnsVerdictDto::Unresolved,
                DnsVerdict::Unverified => DnsVerdictDto::Unverified,
            },
            addresses: report.addresses.iter().map(ToString::to_string).collect(),
        }
    }
}
