use nym_wg_go::PeerConfig;
use std::net::SocketAddr;

use super::{Error, Result};

/// Resolve each peer with DNS64 and update the endpoint.
pub fn resolve_peers(peers: &mut [PeerConfig]) -> Result<()> {
    for peer_config in peers.iter_mut() {
        let resolved_endpoint = resolve_addr(peer_config.endpoint)?;

        if resolved_endpoint == peer_config.endpoint {
            tracing::info!("Resolved {} to self", peer_config.endpoint);
        } else {
            tracing::info!("Resolved {} to {}", peer_config.endpoint, resolved_endpoint);
            peer_config.endpoint = resolved_endpoint;
        }
    }
    Ok(())
}

/// Returns new socket address resolved with DNS64.
/// This should produce an IPv4-mapped IPv6 address usable in IPv6 only networks when connecting to IPv4-only host.
fn resolve_addr(socket_addr: SocketAddr) -> Result<SocketAddr> {
    let hints = dns_lookup::AddrInfoHints {
        flags: 0,                 // set to zero to resolve using DNS64
        address: libc::AF_UNSPEC, // corresponds to ai_family
        socktype: libc::SOCK_DGRAM,
        protocol: libc::IPPROTO_UDP,
    };

    let ip_str = socket_addr.ip().to_string();
    let port = socket_addr.port().to_string();
    let mut addrinfo_iter = dns_lookup::getaddrinfo(Some(&ip_str), Some(&port), Some(hints))
        .map_err(Error::DnsLookup)?;

    let first_addr_info = addrinfo_iter
        .next()
        .ok_or(Error::EmptyDnsLookupResult)?
        .map_err(Error::ParseAddrInfo)?;

    Ok(first_addr_info.sockaddr)
}
