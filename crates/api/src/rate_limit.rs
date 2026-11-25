use std::net::{IpAddr, Ipv4Addr};
use std::sync::{Arc, Weak};
use std::time::Duration;

use axum::body::Body;
use axum::http::Request;
use governor::middleware::NoOpMiddleware;
use tower_governor::GovernorError;
use tower_governor::GovernorLayer;
use tower_governor::governor::{GovernorConfigBuilder, SharedRateLimiter};
use tower_governor::key_extractor::KeyExtractor;

use crate::client_ip;

const UNKNOWN_CLIENT: IpAddr = IpAddr::V4(Ipv4Addr::UNSPECIFIED);
const CLEANUP_INTERVAL: Duration = Duration::from_secs(60);

#[derive(Debug, Clone, Copy)]
pub struct ClientIpKey {
    trust_forwarded: bool,
}

impl KeyExtractor for ClientIpKey {
    type Key = IpAddr;

    fn extract<T>(&self, request: &Request<T>) -> Result<Self::Key, GovernorError> {
        Ok(client_ip::from_request(request, self.trust_forwarded).unwrap_or(UNKNOWN_CLIENT))
    }
}

pub type CredentialLimiter = GovernorLayer<ClientIpKey, NoOpMiddleware, Body>;

pub fn credential_limiter(trust_forwarded: bool) -> Option<CredentialLimiter> {
    let config = GovernorConfigBuilder::default()
        .key_extractor(ClientIpKey { trust_forwarded })
        .period(Duration::from_secs(6))
        .burst_size(10)
        .finish()?;
    let config = Arc::new(config);
    spawn_cleanup(config.limiter());
    Some(GovernorLayer::new(config))
}

type ClientLimiter = SharedRateLimiter<IpAddr, NoOpMiddleware>;

fn spawn_cleanup(limiter: &ClientLimiter) {
    let Ok(runtime) = tokio::runtime::Handle::try_current() else {
        return;
    };
    let limiter: Weak<_> = Arc::downgrade(limiter);
    runtime.spawn(async move {
        let mut ticker = tokio::time::interval(CLEANUP_INTERVAL);
        loop {
            ticker.tick().await;
            let Some(active) = limiter.upgrade() else {
                break;
            };
            active.retain_recent();
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn falls_back_to_shared_key_without_peer_address() {
        let key = ClientIpKey {
            trust_forwarded: true,
        };
        let Ok(request) = Request::builder().body(()) else {
            return;
        };
        assert!(matches!(key.extract(&request), Ok(UNKNOWN_CLIENT)));
    }

    #[test]
    fn builds_without_a_runtime() {
        assert!(credential_limiter(false).is_some());
    }
}
