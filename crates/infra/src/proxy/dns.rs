use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::time::Duration;

const LOOKUP_TIMEOUT: Duration = Duration::from_secs(5);

const CLOUDFLARE_V4: [(Ipv4Addr, u8); 15] = [
    (Ipv4Addr::new(173, 245, 48, 0), 20),
    (Ipv4Addr::new(103, 21, 244, 0), 22),
    (Ipv4Addr::new(103, 22, 200, 0), 22),
    (Ipv4Addr::new(103, 31, 4, 0), 22),
    (Ipv4Addr::new(141, 101, 64, 0), 18),
    (Ipv4Addr::new(108, 162, 192, 0), 18),
    (Ipv4Addr::new(190, 93, 240, 0), 20),
    (Ipv4Addr::new(188, 114, 96, 0), 20),
    (Ipv4Addr::new(197, 234, 240, 0), 22),
    (Ipv4Addr::new(198, 41, 128, 0), 17),
    (Ipv4Addr::new(162, 158, 0, 0), 15),
    (Ipv4Addr::new(104, 16, 0, 0), 13),
    (Ipv4Addr::new(104, 24, 0, 0), 14),
    (Ipv4Addr::new(172, 64, 0, 0), 13),
    (Ipv4Addr::new(131, 0, 72, 0), 22),
];

const CLOUDFLARE_V6: [(Ipv6Addr, u8); 7] = [
    (Ipv6Addr::new(0x2400, 0xcb00, 0, 0, 0, 0, 0, 0), 32),
    (Ipv6Addr::new(0x2606, 0x4700, 0, 0, 0, 0, 0, 0), 32),
    (Ipv6Addr::new(0x2803, 0xf800, 0, 0, 0, 0, 0, 0), 32),
    (Ipv6Addr::new(0x2405, 0xb500, 0, 0, 0, 0, 0, 0), 32),
    (Ipv6Addr::new(0x2405, 0x8100, 0, 0, 0, 0, 0, 0), 32),
    (Ipv6Addr::new(0x2a06, 0x98c0, 0, 0, 0, 0, 0, 0), 29),
    (Ipv6Addr::new(0x2c0f, 0xf248, 0, 0, 0, 0, 0, 0), 32),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DnsVerdict {
    PointsHere,
    PointsElsewhere,
    Proxied,
    Unresolved,
    Unverified,
}

#[derive(Debug, Clone)]
pub struct DnsReport {
    pub addresses: Vec<IpAddr>,
    pub verdict: DnsVerdict,
}

fn in_v4(ip: Ipv4Addr, (net, bits): (Ipv4Addr, u8)) -> bool {
    let mask = u32::MAX.checked_shl(32 - u32::from(bits)).unwrap_or(0);
    u32::from(ip) & mask == u32::from(net) & mask
}

fn in_v6(ip: Ipv6Addr, (net, bits): (Ipv6Addr, u8)) -> bool {
    let mask = u128::MAX.checked_shl(128 - u32::from(bits)).unwrap_or(0);
    u128::from(ip) & mask == u128::from(net) & mask
}

pub fn is_cloudflare(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => CLOUDFLARE_V4.iter().any(|range| in_v4(v4, *range)),
        IpAddr::V6(v6) => CLOUDFLARE_V6.iter().any(|range| in_v6(v6, *range)),
    }
}

pub fn judge(addresses: &[IpAddr], expected: Option<IpAddr>) -> DnsVerdict {
    if addresses.is_empty() {
        return DnsVerdict::Unresolved;
    }
    if addresses.iter().all(|ip| is_cloudflare(*ip)) {
        return DnsVerdict::Proxied;
    }
    match expected {
        Some(ip) if addresses.contains(&ip) => DnsVerdict::PointsHere,
        Some(_) => DnsVerdict::PointsElsewhere,
        None => DnsVerdict::Unverified,
    }
}

pub async fn check_dns(host: &str, expected: Option<IpAddr>) -> DnsReport {
    let lookup = tokio::net::lookup_host((host, 80));
    let mut addresses: Vec<IpAddr> = match tokio::time::timeout(LOOKUP_TIMEOUT, lookup).await {
        Ok(Ok(found)) => found.map(|socket| socket.ip()).collect(),
        _ => Vec::new(),
    };
    addresses.sort();
    addresses.dedup();
    let verdict = judge(&addresses, expected);
    DnsReport { addresses, verdict }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognises_cloudflare_and_mismatches() {
        let cloudflare: IpAddr = "104.21.3.4"
            .parse()
            .unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED));
        let server: IpAddr = "203.0.113.10"
            .parse()
            .unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED));
        let v6: IpAddr = "2606:4700::6810:1"
            .parse()
            .unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED));
        assert!(is_cloudflare(cloudflare) && is_cloudflare(v6) && !is_cloudflare(server));
        assert_eq!(judge(&[], Some(server)), DnsVerdict::Unresolved);
        assert_eq!(judge(&[cloudflare], Some(server)), DnsVerdict::Proxied);
        assert_eq!(judge(&[server], Some(server)), DnsVerdict::PointsHere);
        assert_eq!(judge(&[cloudflare, server], None), DnsVerdict::Unverified);
        let other: IpAddr = "198.51.100.1"
            .parse()
            .unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED));
        assert_eq!(judge(&[other], Some(server)), DnsVerdict::PointsElsewhere);
    }
}
