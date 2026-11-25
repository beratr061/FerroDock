use std::net::{IpAddr, SocketAddr};

use axum::extract::ConnectInfo;
use axum::http::{HeaderMap, Request};

const FORWARDED_FOR: &str = "x-forwarded-for";

fn is_trusted_proxy(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => v4.is_loopback() || v4.is_private() || v4.is_link_local(),
        IpAddr::V6(v6) => v6.is_loopback() || v6.is_unique_local(),
    }
}

fn last_forwarded(headers: &HeaderMap) -> Option<IpAddr> {
    headers
        .get_all(FORWARDED_FOR)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|value| value.split(','))
        .filter_map(|part| part.trim().parse::<IpAddr>().ok())
        .next_back()
}

pub fn resolve(
    peer: Option<SocketAddr>,
    headers: &HeaderMap,
    trust_forwarded: bool,
) -> Option<IpAddr> {
    let peer_ip = peer.map(|addr| addr.ip());
    match peer_ip {
        Some(ip) if trust_forwarded && is_trusted_proxy(ip) => last_forwarded(headers).or(Some(ip)),
        other => other,
    }
}

pub fn from_request<B>(request: &Request<B>, trust_forwarded: bool) -> Option<IpAddr> {
    let peer = request
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|info| info.0);
    resolve(peer, request.headers(), trust_forwarded)
}

#[cfg(test)]
mod tests {
    use axum::http::HeaderValue;

    use super::*;

    fn headers(value: &str) -> HeaderMap {
        let mut map = HeaderMap::new();
        if let Ok(header) = HeaderValue::from_str(value) {
            map.insert(FORWARDED_FOR, header);
        }
        map
    }

    #[test]
    fn trusts_forwarded_header_only_from_private_proxy() {
        let proxy = SocketAddr::from(([172, 18, 0, 2], 443));
        let public = SocketAddr::from(([203, 0, 113, 9], 443));
        let spoofed = headers("1.1.1.1, 198.51.100.7");
        assert_eq!(
            resolve(Some(proxy), &spoofed, true),
            "198.51.100.7".parse().ok()
        );
        assert_eq!(resolve(Some(proxy), &spoofed, false), Some(proxy.ip()));
        assert_eq!(resolve(Some(public), &spoofed, true), Some(public.ip()));
        assert_eq!(
            resolve(Some(proxy), &HeaderMap::new(), true),
            Some(proxy.ip())
        );
    }
}
