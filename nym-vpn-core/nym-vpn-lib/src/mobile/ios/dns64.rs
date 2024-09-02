// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: Apache-2.0

use std::{
    ffi::CString,
    net::{SocketAddr, SocketAddrV4, SocketAddrV6},
};

use nix::{
    libc,
    sys::socket::{SockaddrIn, SockaddrIn6, SockaddrLike},
};

use crate::mobile::wg_config::WgPeer;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Failed to resolve {} (error code: {})", addr, code)]
    DnsLookup { code: i32, addr: SocketAddr },

    #[error("DNS lookup has seemingly succeeded without any results")]
    EmptyDnsLookupResult,

    #[error("Failed to convert port number to string")]
    PortToString,

    #[error("Failed to convert IP address to string")]
    IpAddressToString,
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Types implementing dns64 resolution.
pub trait Dns64Resolution: Sized {
    /// Replace the peer endpoint after re-resolving it with dns64.
    fn resolve_in_place(&mut self) -> Result<()>;

    /// Returns a new peer with dns64 re-resolved peer endpoint.
    fn resolved(&self) -> Result<Self>;
}

impl Dns64Resolution for WgPeer {
    fn resolve_in_place(&mut self) -> Result<()> {
        self.endpoint = reresolve_endpoint(self.endpoint)?;
        Ok(())
    }

    fn resolved(&self) -> Result<Self> {
        Ok(WgPeer {
            endpoint: reresolve_endpoint(self.endpoint)?,
            public_key: self.public_key.clone(),
        })
    }
}

/// Re-resolve an endpoint with dns64
pub(crate) fn reresolve_endpoint(endpoint: SocketAddr) -> Result<SocketAddr> {
    reresolve_addr(endpoint).inspect(|resolved_endpoint| {
        if resolved_endpoint == &endpoint {
            tracing::info!("Resolved {} to self", endpoint);
        } else {
            tracing::info!("Resolved {} to {}", endpoint, resolved_endpoint);
        }
    })
}

/// Returns the new socket address re-resolved with dns64.
/// This should produce an IPv4-mapped IPv6 address usable in IPv6 only networks
/// when connecting to IPv4-only host.
fn reresolve_addr(socket_addr: SocketAddr) -> Result<SocketAddr> {
    let mut hints: libc::addrinfo = unsafe { std::mem::zeroed() };
    hints.ai_flags = 0; // Set to zero to resolve using dns64
    hints.ai_family = libc::AF_UNSPEC;
    hints.ai_socktype = libc::SOCK_DGRAM;
    hints.ai_protocol = libc::IPPROTO_UDP;

    let node = CString::new(socket_addr.ip().to_string()).map_err(|_| Error::IpAddressToString)?;
    let service = CString::new(socket_addr.port().to_string()).map_err(|_| Error::PortToString)?;

    let mut result = std::ptr::null_mut();

    let err_code =
        unsafe { libc::getaddrinfo(node.as_ptr(), service.as_ptr(), &hints, &mut result) };
    if err_code != 0 {
        return Err(Error::DnsLookup {
            code: err_code,
            addr: socket_addr,
        })?;
    }

    if result.is_null() {
        return Err(Error::EmptyDnsLookupResult);
    };

    let addr_info = unsafe { *result };
    let resolved_sockaddr = match addr_info.ai_family {
        libc::AF_INET => {
            unsafe { SockaddrIn::from_raw(addr_info.ai_addr, Some(addr_info.ai_addrlen)) }
                .map(|sin| SocketAddr::V4(SocketAddrV4::from(sin)))
        }
        libc::AF_INET6 => {
            unsafe { SockaddrIn6::from_raw(addr_info.ai_addr, Some(addr_info.ai_addrlen)) }
                .map(|sin6| SocketAddr::V6(SocketAddrV6::from(sin6)))
        }
        _ => None,
    };

    unsafe { libc::freeaddrinfo(result) };

    resolved_sockaddr.ok_or(Error::EmptyDnsLookupResult)
}
