// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    os::windows::io::AsRawSocket,
};

use anyhow::{Context, Result, bail};
use tokio::net::TcpSocket;
use windows_sys::Win32::{
    NetworkManagement::IpHelper::{
        FreeMibTable, GetUnicastIpAddressTable, MIB_UNICASTIPADDRESS_TABLE,
    },
    Networking::WinSock::{AF_INET, AF_INET6, AF_UNSPEC, SOCKET_ERROR, setsockopt},
};

// These aren't defined by windows-sys
const IPPROTO_IP_LEVEL: i32 = 0; // IPPROTO_IP
const IPPROTO_IPV6_LEVEL: i32 = 41; // IPPROTO_IPV6
const IP_UNICAST_IF_OPT: i32 = 31; // IP_UNICAST_IF  — interface index in network byte order
const IPV6_UNICAST_IF_OPT: i32 = 31; // IPV6_UNICAST_IF — interface index in host byte order

/// Force the socket to send outbound traffic through the VPN tunnel interface by
/// setting `IP_UNICAST_IF` (IPv4) or `IPV6_UNICAST_IF` (IPv6).
///
/// The interface is identified by enumerating the unicast address table and matching
/// `tunnel_ip`, so no interface name or index needs to be passed via IPC.
pub fn bind_by_interface_index(
    socket: &TcpSocket,
    tunnel_ip: IpAddr,
    target: SocketAddr,
) -> Result<()> {
    let if_index = interface_index_for_ip(tunnel_ip)
        .with_context(|| format!("No adapter found with IP {tunnel_ip}"))?;

    let raw = socket.as_raw_socket() as windows_sys::Win32::Networking::WinSock::SOCKET;

    // IP_UNICAST_IF expects the index in **network byte order** (confirmed by the
    // WireGuard-NT driver source which calls RtlUlongByteSwap before setsockopt).
    // IPV6_UNICAST_IF expects the index in **host byte order** (per MSDN).
    let ret = match (tunnel_ip, target) {
        (IpAddr::V4(_), SocketAddr::V4(_)) => {
            let idx_be = if_index.to_be();
            unsafe {
                setsockopt(
                    raw,
                    IPPROTO_IP_LEVEL,
                    IP_UNICAST_IF_OPT,
                    std::ptr::addr_of!(idx_be) as *const u8,
                    std::mem::size_of::<u32>() as i32,
                )
            }
        }
        (IpAddr::V6(_), SocketAddr::V6(_)) => unsafe {
            setsockopt(
                raw,
                IPPROTO_IPV6_LEVEL,
                IPV6_UNICAST_IF_OPT,
                std::ptr::addr_of!(if_index) as *const u8,
                std::mem::size_of::<u32>() as i32,
            )
        },
        _ => {
            // Address-family mismatch (e.g. IPv4 tunnel but IPv6 target).
            // Nothing to set; the caller will fall back to bind-by-IP.
            return Ok(());
        }
    };

    if ret == SOCKET_ERROR {
        bail!(
            "setsockopt(IP_UNICAST_IF) failed: {}",
            std::io::Error::last_os_error()
        );
    }

    Ok(())
}

/// Return the `InterfaceIndex` of the adapter that owns `ip`, or `None`.
fn interface_index_for_ip(ip: IpAddr) -> Option<u32> {
    let mut table: *mut MIB_UNICASTIPADDRESS_TABLE = std::ptr::null_mut();

    // SAFETY: GetUnicastIpAddressTable heap-allocates a MIB_UNICASTIPADDRESS_TABLE
    // and sets *table to point to it.  On success the caller must free it with
    // FreeMibTable.
    let err = unsafe { GetUnicastIpAddressTable(AF_UNSPEC, &mut table) };
    if err != 0 {
        tracing::warn!("GetUnicastIpAddressTable failed with code {err}");
        return None;
    }

    // SAFETY: err == 0 guarantees table is a valid, non-null pointer.
    let num = unsafe { (*table).NumEntries } as usize;
    // Table is a flexible-array-member: the allocation holds `NumEntries` consecutive
    // MIB_UNICASTIPADDRESS_ROW values starting at Table[0].
    let first = unsafe { (*table).Table.as_ptr() };

    let mut result: Option<u32> = None;

    'rows: for i in 0..num {
        // SAFETY: indices 0..num are all within the allocation.
        let row = unsafe { &*first.add(i) };

        // si_family is the discriminant field present at offset 0 in every
        // SOCKADDR_INET variant, so reading it is always valid.
        let family = unsafe { row.Address.si_family };

        let row_ip: IpAddr = if family == AF_INET {
            // S_addr stores the IPv4 address in network byte order (big-endian).
            // to_ne_bytes() returns the raw memory bytes, which equal the four
            // IPv4 octets in network order — exactly what Ipv4Addr::from([u8;4])
            // expects.
            let s_addr = unsafe { row.Address.Ipv4.sin_addr.S_un.S_addr };
            IpAddr::V4(Ipv4Addr::from(s_addr.to_ne_bytes()))
        } else if family == AF_INET6 {
            IpAddr::V6(Ipv6Addr::from(unsafe { row.Address.Ipv6.sin6_addr.u.Byte }))
        } else {
            continue 'rows;
        };

        if row_ip == ip {
            result = Some(row.InterfaceIndex);
            break;
        }
    }

    // SAFETY: table was successfully allocated and has not been freed yet.
    unsafe { FreeMibTable(table as *const _) };

    result
}
