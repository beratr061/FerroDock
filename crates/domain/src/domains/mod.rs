mod routes;
mod service;

pub use ferrodock_db::domains::DomainRecord as Domain;
pub use ferrodock_infra::proxy::{DnsReport, DnsVerdict};
pub use routes::{is_compose, publish_routes, routes_for};
pub use service::{DomainService, DomainSettings, NewDomainInput};
