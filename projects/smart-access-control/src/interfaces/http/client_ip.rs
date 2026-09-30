//! Which address a request came from, for per-client limits.

use std::net::{IpAddr, SocketAddr};

use axum::{
    extract::{ConnectInfo, FromRef, FromRequestParts},
    http::{HeaderMap, request::Parts},
};

use super::AppState;
use crate::config::TrustedProxies;

const FORWARDED_FOR: &str = "x-forwarded-for";

/// The client's address: the TCP peer, or, when the peer is a trusted
/// reverse proxy, the address that proxy reports in `X-Forwarded-For`.
///
/// `None` when the server does not record peers (in-process test routers).
pub struct ClientIp(pub Option<IpAddr>);

impl<S> FromRequestParts<S> for ClientIp
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        // Axum's own extractor, rather than reading the extension directly:
        // it also understands `MockConnectInfo`, which tests use.
        let Ok(ConnectInfo(peer)) =
            ConnectInfo::<SocketAddr>::from_request_parts(parts, state).await
        else {
            return Ok(Self(None));
        };
        let trusted = &AppState::from_ref(state).trusted_proxies;
        Ok(Self(Some(client_ip(peer.ip(), &parts.headers, trusted))))
    }
}

/// Resolves the client address from the peer and `X-Forwarded-For`.
///
/// Each proxy appends the address it received the request from, so the
/// header reads `client, proxy1, proxy2` and the peer is the last proxy.
/// Only the right-hand end can be trusted: anything to the left of the
/// first untrusted hop was written by that hop and may be invented. So we
/// walk from the right, skipping our own proxies, and stop at the first
/// address that is not one of them.
///
/// If an entry is not an IP address, the walk stops at the last address
/// known to be real (a trusted proxy): unparseable input never lets a
/// caller choose its address.
pub fn client_ip(peer: IpAddr, headers: &HeaderMap, trusted: &TrustedProxies) -> IpAddr {
    let mut client = peer;
    if !trusted.contains(peer) {
        return client;
    }
    // Several `X-Forwarded-For` headers are one list, in order.
    let hops: Vec<&str> = headers
        .get_all(FORWARDED_FOR)
        .iter()
        .flat_map(|value| value.to_str().unwrap_or("").split(','))
        .map(str::trim)
        .collect();
    for hop in hops.iter().rev() {
        let Ok(ip) = hop.parse::<IpAddr>() else {
            break;
        };
        client = ip;
        if !trusted.contains(ip) {
            break;
        }
    }
    client
}

#[cfg(test)]
mod tests {
    use axum::http::HeaderValue;

    use super::*;
    use crate::config::IpNetwork;

    const PROXY: &str = "172.30.83.10";

    fn trusted() -> TrustedProxies {
        TrustedProxies::new(vec![
            IpNetwork::parse(PROXY).unwrap(),
            IpNetwork::parse("10.0.0.0/8").unwrap(),
        ])
    }

    fn resolve(peer: &str, forwarded_for: &[&str]) -> String {
        let mut headers = HeaderMap::new();
        for value in forwarded_for {
            headers.append(FORWARDED_FOR, HeaderValue::from_str(value).unwrap());
        }
        client_ip(peer.parse().unwrap(), &headers, &trusted()).to_string()
    }

    #[test]
    fn untrusted_peers_cannot_choose_their_address() {
        assert_eq!(resolve("203.0.113.9", &["198.51.100.1"]), "203.0.113.9");
        assert_eq!(resolve("203.0.113.9", &[]), "203.0.113.9");
    }

    #[test]
    fn a_trusted_proxy_reports_the_client() {
        assert_eq!(resolve(PROXY, &["198.51.100.1"]), "198.51.100.1");
        // IPv4 peer seen through a dual-stack socket.
        assert_eq!(
            resolve("::ffff:172.30.83.10", &["198.51.100.1"]),
            "198.51.100.1"
        );
        assert_eq!(resolve(PROXY, &["2001:db8::7"]), "2001:db8::7");
    }

    #[test]
    fn spoofed_entries_left_of_the_real_client_are_ignored() {
        // The client sent `X-Forwarded-For: 1.2.3.4`; our proxy appended
        // the address it actually saw.
        assert_eq!(resolve(PROXY, &["1.2.3.4, 198.51.100.1"]), "198.51.100.1");
        assert_eq!(resolve(PROXY, &["1.2.3.4", "198.51.100.1"]), "198.51.100.1");
    }

    #[test]
    fn chains_of_trusted_proxies_are_skipped() {
        assert_eq!(
            resolve(PROXY, &["198.51.100.1, 10.1.1.1, 10.2.2.2"]),
            "198.51.100.1"
        );
        // Every hop is ours: the leftmost is the best we know.
        assert_eq!(resolve(PROXY, &["10.1.1.1, 10.2.2.2"]), "10.1.1.1");
    }

    #[test]
    fn missing_or_garbage_headers_fall_back_to_a_known_address() {
        assert_eq!(resolve(PROXY, &[]), PROXY);
        assert_eq!(resolve(PROXY, &[""]), PROXY);
        assert_eq!(resolve(PROXY, &["unknown"]), PROXY);
        // Garbage after a trusted hop: stop at that hop.
        assert_eq!(
            resolve(PROXY, &["198.51.100.1, junk, 10.2.2.2"]),
            "10.2.2.2"
        );
        // Ports are not part of the format.
        assert_eq!(resolve(PROXY, &["198.51.100.1:4000"]), PROXY);
    }
}
