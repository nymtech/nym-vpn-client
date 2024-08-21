// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: Apache-2.0

use nix::sys::socket::{SockaddrIn, SockaddrIn6, SockaddrLike};
use nym_wg_go::PeerConfig;
use std::{
    ffi::CString,
    net::{SocketAddr, SocketAddrV4, SocketAddrV6},
};

use super::{Error, Result};

/// Re-resolve an endpoint with DNS64
pub fn reresolve_endpoint(endpoint: SocketAddr) -> Result<SocketAddr> {
    resolve_addr(endpoint).inspect(|resolved_endpoint| {
        if resolved_endpoint == &endpoint {
            tracing::info!("Resolved {} to self", endpoint);
        } else {
            tracing::info!("Resolved {} to {}", endpoint, resolved_endpoint);
        }
    })
}

/// Returns new socket address resolved with DNS64.
/// This should produce an IPv4-mapped IPv6 address usable in IPv6 only networks when connecting to IPv4-only host.
fn resolve_addr(socket_addr: SocketAddr) -> Result<SocketAddr> {
    let mut hints: libc::addrinfo = unsafe { std::mem::zeroed() };
    hints.ai_flags = 0; // Set to zero to resolve using DNS64
    hints.ai_family = libc::AF_UNSPEC;
    hints.ai_socktype = libc::SOCK_DGRAM;
    hints.ai_protocol = libc::IPPROTO_UDP;

    let node = CString::new(socket_addr.ip().to_string()).map_err(|_| Error::ConvertIpToCstr)?;
    let service =
        CString::new(socket_addr.port().to_string()).map_err(|_| Error::ConvertPortToCstr)?;

    let mut res = std::ptr::null_mut();

    let err_code = unsafe { libc::getaddrinfo(node.as_ptr(), service.as_ptr(), &hints, &mut res) };
    if err_code != 0 {
        return Err(Error::DnsLookup {
            code: err_code,
            addr: socket_addr,
        })?;
    }

    let addr_info = unsafe { *res };

    let resolved_ip_addr = match addr_info.ai_family {
        libc::AF_INET => {
            unsafe { SockaddrIn::from_raw(addr_info.ai_addr, Some(addr_info.ai_addrlen)) }
                .map(|x| SocketAddr::V4(SocketAddrV4::new(x.ip(), x.port())))
        }
        libc::AF_INET6 => {
            unsafe { SockaddrIn6::from_raw(addr_info.ai_addr, Some(addr_info.ai_addrlen)) }.map(
                |x| {
                    SocketAddr::V6(SocketAddrV6::new(
                        x.ip(),
                        x.port(),
                        x.flowinfo(),
                        x.scope_id(),
                    ))
                },
            )
        }
        _ => None,
    };

    unsafe { libc::freeaddrinfo(res) };

    if let Some(resolved_ip_addr) = resolved_ip_addr {
        Ok(resolved_ip_addr)
    } else {
        Err(Error::EmptyDnsLookupResult)?
    }
}
