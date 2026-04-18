// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    io::Error,
    mem::size_of,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    os::windows::io::AsRawSocket,
    ptr::addr_of,
};

use anyhow::{Context, Result, bail};
use tokio::net::TcpSocket;
use windows_sys::Win32::{
    NetworkManagement::IpHelper::{
        FreeMibTable, GetUnicastIpAddressTable, MIB_UNICASTIPADDRESS_TABLE,
    },
    Networking::WinSock::{AF_INET, AF_INET6, AF_UNSPEC, SOCKET, SOCKET_ERROR, setsockopt},
};

// These aren't defined by windows-sys
const IPPROTO_IP_LEVEL: i32 = 0; // IPPROTO_IP
const IPPROTO_IPV6_LEVEL: i32 = 41; // IPPROTO_IPV6
const IP_UNICAST_IF_OPT: i32 = 31; // IP_UNICAST_IF  — interface index in network byte order
const IPV6_UNICAST_IF_OPT: i32 = 31; // IPV6_UNICAST_IF — interface index in host byte order

pub fn bind_by_interface_index(
    socket: &TcpSocket,
    if_index: Option<u32>,
    bind_addr: IpAddr,
    target_addr: SocketAddr,
) -> Result<()> {
    let Some(if_index) = if_index else {
        tracing::warn!(
            "Cannot bind socket by interface index: no default interface index available (no default route?)"
        );
        return Ok(());
    };

    let raw = socket.as_raw_socket() as SOCKET;

    // IP_UNICAST_IF expects the index in **network byte order** (confirmed by the
    // WireGuard-NT driver source which calls RtlUlongByteSwap before setsockopt).
    // IPV6_UNICAST_IF expects the index in **host byte order** (per MSDN).
    let ret = match (bind_addr, target_addr) {
        (IpAddr::V4(_), SocketAddr::V4(_)) => {
            let idx_be = if_index.to_be() as i32;
            unsafe {
                setsockopt(
                    raw,
                    IPPROTO_IP_LEVEL,
                    IP_UNICAST_IF_OPT,
                    addr_of!(idx_be) as *const u8,
                    size_of::<u32>() as i32,
                )
            }
        }
        (IpAddr::V6(_), SocketAddr::V6(_)) => unsafe {
            setsockopt(
                raw,
                IPPROTO_IPV6_LEVEL,
                IPV6_UNICAST_IF_OPT,
                addr_of!(if_index) as *const u8,
                size_of::<u32>() as i32,
            )
        },
        _ => {
            tracing::warn!(
                "Address family mismatch between bind {bind_addr} and target {target_addr} address types; \
                 cannot bind by interface index"
            );
            return Ok(());
        }
    };

    if ret == SOCKET_ERROR {
        bail!(
            "setsockopt(IP_UNICAST_IF) failed: {}",
            Error::last_os_error()
        );
    }

    Ok(())
}
