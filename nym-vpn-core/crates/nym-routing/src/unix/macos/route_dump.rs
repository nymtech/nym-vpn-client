// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! One-shot dump of the macOS routing table via `sysctl(NET_RT_DUMP2)`, used
//! to find every interface currently holding a default-route-shaped entry.
//!
//! The live PF_ROUTE socket used elsewhere in this module (see
//! [`super::watch`]) only reports routing table *changes*, not a
//! point-in-time snapshot of the whole table, so it can't be reused here.

use std::{
    ffi::c_int,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
};

use nix::sys::socket::SockaddrStorage;

use super::data::{AddressFlag, RouteSockAddrIterator, RouteSocketAddress};

/// Not exposed by the `libc` crate. See `<net/route.h>` on macOS, where it's
/// defined immediately after `NET_RT_IFLIST2` (`libc::NET_RT_IFLIST2 == 6`).
const NET_RT_DUMP2: c_int = 7;

const HEADER_SIZE: usize = std::mem::size_of::<libc::rt_msghdr2>();

/// Errors that can occur while dumping the macOS routing table.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Failed to size or read the routing table via `sysctl`.
    #[error("sysctl(NET_RT_DUMP2) failed")]
    Sysctl(#[source] std::io::Error),

    /// A route message in the dump had an address bitmask with unrecognized
    /// bits set.
    #[error("route message had an unrecognized address bitmask: {0}")]
    UnknownAddressFlag(i32),
}

/// Get every interface currently holding a default-route-shaped entry, for
/// the given address family. See [`crate::DefaultRouteInterfaces`].
pub(super) fn get_default_route_interfaces(
    family: crate::AddressFamily,
) -> Result<crate::DefaultRouteInterfaces, Error> {
    let address_family = match family {
        crate::AddressFamily::Ipv4 => libc::AF_INET,
        crate::AddressFamily::Ipv6 => libc::AF_INET6,
    };

    let buffer = dump_routing_table(address_family)?;
    let mut result = crate::DefaultRouteInterfaces::default();

    for route in parse_dump(&buffer)? {
        if !route.is_default {
            continue;
        }

        let interface_index = u32::from(route.interface_index);
        if is_tunnel_like_interface(route.interface_index) {
            result.virtual_.insert(interface_index);
        } else {
            result.physical.insert(interface_index);
        }
    }

    Ok(result)
}

/// Best-effort classification of virtual/tunnel interfaces by name, mirroring
/// the "utun"-prefix convention already used by
/// [`super::interface::should_skip_interface`], extended to also cover the
/// prefixes used by other common VPN clients on macOS.
fn is_tunnel_like_interface(interface_index: u16) -> bool {
    const TUNNEL_PREFIXES: [&str; 4] = ["utun", "tun", "tap", "ppp"];

    let Ok(name) = nix::net::if_::if_indextoname(u32::from(interface_index)) else {
        return false;
    };
    let Ok(name) = name.into_string() else {
        return false;
    };

    TUNNEL_PREFIXES
        .iter()
        .any(|prefix| name.starts_with(prefix))
}

fn dump_routing_table(address_family: c_int) -> Result<Vec<u8>, Error> {
    let mut mib: [c_int; 6] = [
        libc::CTL_NET,
        libc::PF_ROUTE,
        0,
        address_family,
        NET_RT_DUMP2,
        0,
    ];

    let mut needed: libc::size_t = 0;
    // SAFETY: `mib` names a valid 6-element sysctl MIB for a routing table
    // dump. Passing a null `oldp` with a valid `oldlenp` asks the kernel to
    // report the required buffer size in `needed` without copying anything.
    let ret = unsafe {
        libc::sysctl(
            mib.as_mut_ptr(),
            mib.len() as u32,
            std::ptr::null_mut(),
            &mut needed,
            std::ptr::null_mut(),
            0,
        )
    };
    if ret < 0 {
        return Err(Error::Sysctl(std::io::Error::last_os_error()));
    }
    if needed == 0 {
        return Ok(Vec::new());
    }

    // The routing table can grow between the sizing call and the real dump,
    // so pad generously - this is the same pattern the BSD `route`/`netstat`
    // tools use around `sysctl(NET_RT_DUMP)`.
    let mut buffer = vec![0u8; needed + needed / 2];
    let mut len = buffer.len() as libc::size_t;

    // SAFETY: `buffer` has `len` allocated bytes available, and `len` is
    // updated in place to the number of bytes actually written.
    let ret = unsafe {
        libc::sysctl(
            mib.as_mut_ptr(),
            mib.len() as u32,
            buffer.as_mut_ptr().cast(),
            &mut len,
            std::ptr::null_mut(),
            0,
        )
    };
    if ret < 0 {
        return Err(Error::Sysctl(std::io::Error::last_os_error()));
    }
    buffer.truncate(len);
    Ok(buffer)
}

struct DumpedRoute {
    interface_index: u16,
    is_default: bool,
}

fn parse_dump(buffer: &[u8]) -> Result<Vec<DumpedRoute>, Error> {
    let mut routes = Vec::new();
    let mut offset = 0;

    while offset + HEADER_SIZE <= buffer.len() {
        // SAFETY: at least `HEADER_SIZE` bytes remain in `buffer` from
        // `offset` (checked by the loop condition); `rt_msghdr2` contains no
        // pointers, and `read_unaligned` doesn't require the source to be
        // aligned.
        let header: libc::rt_msghdr2 =
            unsafe { std::ptr::read_unaligned(buffer[offset..].as_ptr().cast()) };

        let msg_len = usize::from(header.rtm_msglen);
        if msg_len < HEADER_SIZE || offset + msg_len > buffer.len() {
            break;
        }

        let payload = &buffer[offset + HEADER_SIZE..offset + msg_len];
        offset += msg_len;

        let Some(address_flags) = AddressFlag::from_bits(header.rtm_addrs) else {
            return Err(Error::UnknownAddressFlag(header.rtm_addrs));
        };

        let mut destination = None;
        let mut netmask = None;

        for sockaddr in RouteSockAddrIterator::new(payload, address_flags) {
            let Ok(sockaddr) = sockaddr else {
                break;
            };
            match sockaddr {
                RouteSocketAddress::Destination(addr) => {
                    destination = addr.as_ref().and_then(sockaddr_to_ip);
                }
                RouteSocketAddress::Netmask(addr) => {
                    netmask = Some(addr.as_ref().and_then(sockaddr_to_ip));
                }
                _ => {}
            }
        }

        routes.push(DumpedRoute {
            interface_index: header.rtm_index,
            is_default: is_default_route(destination, netmask),
        });
    }

    Ok(routes)
}

/// Whether `destination`/`netmask` describe a default route - either
/// literally `0.0.0.0/0`/`::/0` (netmask absent, or present and
/// unspecified), or the `0.0.0.0/1` + `128.0.0.0/1` split some VPN clients
/// install instead of replacing the default route directly.
fn is_default_route(destination: Option<IpAddr>, netmask: Option<Option<IpAddr>>) -> bool {
    let Some(destination) = destination else {
        return false;
    };
    // `None` means the netmask attribute was absent entirely; `Some(None)`
    // means it was present but empty. Both conventionally mean "unspecified".
    let netmask = netmask.flatten();

    match destination {
        IpAddr::V4(dest) => {
            let netmask_is_unspecified =
                matches!(netmask, None | Some(IpAddr::V4(Ipv4Addr::UNSPECIFIED)));
            let is_full_default = dest == Ipv4Addr::UNSPECIFIED && netmask_is_unspecified;
            let is_split_half = (dest == Ipv4Addr::UNSPECIFIED
                || dest == Ipv4Addr::new(128, 0, 0, 0))
                && netmask == Some(IpAddr::V4(Ipv4Addr::new(128, 0, 0, 0)));
            is_full_default || is_split_half
        }
        IpAddr::V6(dest) => {
            dest == Ipv6Addr::UNSPECIFIED
                && matches!(netmask, None | Some(IpAddr::V6(Ipv6Addr::UNSPECIFIED)))
        }
    }
}

fn sockaddr_to_ip(sockaddr: &SockaddrStorage) -> Option<IpAddr> {
    if let Some(v4) = sockaddr.as_sockaddr_in() {
        return Some(IpAddr::V4(*std::net::SocketAddrV4::from(*v4).ip()));
    }
    if let Some(v6) = sockaddr.as_sockaddr_in6() {
        return Some(IpAddr::V6(*std::net::SocketAddrV6::from(*v6).ip()));
    }
    None
}
