mod dns;
mod host;
mod traefik;

pub use dns::{DnsReport, DnsVerdict, check_dns, is_cloudflare};
pub use host::{normalize_host, normalize_path_prefix};
pub use traefik::{ProxyRoutes, Route, render};
