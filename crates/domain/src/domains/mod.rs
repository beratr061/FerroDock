mod service;

pub use ferrodock_db::domains::DomainRecord as Domain;
pub use ferrodock_infra::proxy::{DnsReport, DnsVerdict};
pub use service::{DomainService, DomainSettings, NewDomainInput};
