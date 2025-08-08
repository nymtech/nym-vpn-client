// Copyright 2025 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Provides functionality for managing network interfaces on macOS.
//!
//! Heavily inspired by the `ifconfig` command-line tool.
//! <https://github.com/apple-oss-distributions/network_cmds/tree/main/ifconfig.tproj>

use std::{
    ffi::{CStr, c_char},
    io,
    net::{Ipv4Addr, SocketAddrV4},
    os::fd::{AsRawFd, OwnedFd},
};

use nix::{
    libc::{IFNAMSIZ, ifreq, sockaddr},
    sys::socket::{AddressFamily, SockFlag, SockType, SockaddrIn, SockaddrLike, socket},
};

// usr/include/sys/sockio.h
nix::ioctl_write_ptr!(siocdifaddr, b'i', 25, ifreq);
nix::ioctl_write_ptr!(siocaifaddr, 'i', 26, ifaliasreq);

/// Adds an IPv4 alias to a network interface.
pub async fn add_alias(interface: &str, addr: Ipv4Addr) -> io::Result<()> {
    let ctl_socket = open_ctl_socket(AddressFamily::Inet)?;
    let alias_addr = SockaddrIn::from(SocketAddrV4::new(addr, 0));
    let destination = SockaddrIn::from(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0));
    let netmask = SockaddrIn::from(SocketAddrV4::new(Ipv4Addr::BROADCAST, 0));

    let mut req = ifaliasreq {
        ifra_name: Default::default(),
        ifra_addr: unsafe { *alias_addr.as_ptr() },
        ifra_broadaddr: unsafe { *destination.as_ptr() },
        ifra_mask: unsafe { *netmask.as_ptr() },
    };
    copy_interface_name_into(interface, &mut req.ifra_name);

    unsafe { siocaifaddr(ctl_socket.as_raw_fd(), &req as _) }.inspect_err(|e| {
        tracing::error!("Failed to add alias {addr} for {interface}: {e}");
    })?;

    Ok(())
}

/// Removes an IPv4 alias from a network interface.
pub async fn remove_alias(interface: &str, addr: Ipv4Addr) -> io::Result<()> {
    let ctl_socket = open_ctl_socket(AddressFamily::Inet)?;
    let alias_addr = SockaddrIn::from(SocketAddrV4::new(addr, 0));

    let mut req: ifreq = unsafe { std::mem::zeroed() };
    req.ifr_ifru.ifru_addr = unsafe { *alias_addr.as_ptr() };
    copy_interface_name_into(interface, &mut req.ifr_name);

    unsafe { siocdifaddr(ctl_socket.as_raw_fd(), &req as _) }.inspect_err(|e| {
        tracing::error!("Failed to remove alias {addr} from {interface}: {e}");
    })?;

    Ok(())
}

fn open_ctl_socket(family: AddressFamily) -> nix::Result<OwnedFd> {
    socket(family, SockType::Datagram, SockFlag::empty(), None).inspect_err(|e| {
        tracing::error!("Cannot connect to control socket ({family:?}): {e}");
    })
}

fn copy_interface_name_into(interface_name: &str, buf: &mut [c_char; IFNAMSIZ]) {
    // Take IFNAMESIZ-1 bytes leaving space for nul terminator
    let mut bytes = interface_name
        .as_bytes()
        .iter()
        .copied()
        .take(IFNAMSIZ - 1)
        .collect::<Vec<u8>>();
    // Add nul terminator
    bytes.push(0);

    // Safety: skip interior nul byte checks since the copy is made to fixed array
    let name_str = unsafe { CStr::from_bytes_with_nul_unchecked(&bytes) };

    // Safety: name_str is guaranteed to not exceed IFNAMESIZ
    unsafe { std::ptr::copy_nonoverlapping(name_str.as_ptr(), buf.as_mut_ptr(), bytes.len()) };
}

// usr/include/net/if.h
// see: https://github.com/rust-lang/libc/issues/4435
#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Copy, Clone)]
struct ifaliasreq {
    pub ifra_name: [c_char; IFNAMSIZ],
    pub ifra_addr: sockaddr,
    pub ifra_broadaddr: sockaddr,
    pub ifra_mask: sockaddr,
}
