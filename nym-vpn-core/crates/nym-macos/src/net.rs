// Copyright 2025 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Provides functionality for managing network interfaces on macOS.
//!
//! Heavily inspired by `ifconfig` command-line tool.
//! <https://github.com/apple-oss-distributions/network_cmds/tree/main/ifconfig.tproj>

use std::{
    ffi::{CStr, c_char, c_int},
    io,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddrV4, SocketAddrV6},
    os::fd::{AsRawFd, OwnedFd},
};

use nix::{
    libc::{IFNAMSIZ, ifreq, in6_ifreq, sockaddr, sockaddr_in6, time_t},
    sys::socket::{
        AddressFamily, SockFlag, SockType, SockaddrIn, SockaddrIn6, SockaddrLike, socket,
    },
};

// usr/include/sys/sockio.h
nix::ioctl_write_ptr!(siocdifaddr, b'i', 25, ifreq);
nix::ioctl_write_ptr!(siocaifaddr, 'i', 26, ifaliasreq);
nix::ioctl_write_ptr!(siocaifaddr_in6, 'i', 26, in6_aliasreq);
nix::ioctl_write_ptr!(siocdifaddr_in6, b'i', 25, in6_ifreq);

/// Adds an IP alias to a network interface.
pub async fn add_alias(interface: &str, addr: IpAddr) -> io::Result<()> {
    match addr {
        IpAddr::V4(addr) => {
            let ctl_socket = open_ctl_socket(AddressFamily::Inet)?;
            let alias_addr = SockaddrIn::from(SocketAddrV4::new(addr, 0));
            let destination = SockaddrIn::from(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0));
            let netmask = SockaddrIn::from(SocketAddrV4::new(Ipv4Addr::new(255, 0, 0, 0), 0));

            let mut req = ifaliasreq {
                ifra_name: Default::default(),
                ifra_addr: unsafe { *alias_addr.as_ptr() },
                ifra_broadaddr: unsafe { *destination.as_ptr() },
                ifra_mask: unsafe { *netmask.as_ptr() },
            };
            copy_interface_name(interface, &mut req.ifra_name);

            unsafe { siocaifaddr(ctl_socket.as_raw_fd(), &req as _) }.inspect_err(|e| {
                tracing::error!("Failed to add alias {addr} for {interface}: {e}");
            })?;
            Ok(())
        }
        IpAddr::V6(addr) => {
            let ctl_socket = open_ctl_socket(AddressFamily::Inet6)?;
            let alias_addr = SockaddrIn6::from(SocketAddrV6::new(addr, 0, 0, 0));
            let prefix_mask = SockaddrIn6::from(SocketAddrV6::new(ipv6_netmask(64), 0, 0, 0));

            let mut req: in6_aliasreq = unsafe { std::mem::zeroed() };
            req.ifra_addr = *alias_addr.as_ref();
            req.ifra_prefixmask = *prefix_mask.as_ref();
            req.ifra_lifetime.ia6t_pltime = u32::MAX;
            req.ifra_lifetime.ia6t_vltime = u32::MAX;
            req.ifra_flags = IN6_IFF_NODAD;
            copy_interface_name(interface, &mut req.ifra_name);

            unsafe { siocaifaddr_in6(ctl_socket.as_raw_fd(), &req as _) }.inspect_err(|e| {
                tracing::error!("Failed to add alias {addr} for {interface}: {e}");
            })?;
            Ok(())
        }
    }
}

/// Removes an IP alias from a network interface.
pub async fn remove_alias(interface: &str, addr: IpAddr) -> io::Result<()> {
    match addr {
        IpAddr::V4(addr) => {
            let ctl_socket = open_ctl_socket(AddressFamily::Inet)?;
            let alias_addr = SockaddrIn::from(SocketAddrV4::new(addr, 0));

            let mut req: ifreq = unsafe { std::mem::zeroed() };
            req.ifr_ifru.ifru_addr = unsafe { *alias_addr.as_ptr() };
            copy_interface_name(interface, &mut req.ifr_name);

            unsafe { siocdifaddr(ctl_socket.as_raw_fd(), &req as _) }.inspect_err(|e| {
                tracing::error!("Failed to remove alias {addr} from {interface}: {e}");
            })?;
        }
        IpAddr::V6(addr) => {
            let ctl_socket = open_ctl_socket(AddressFamily::Inet6)?;
            let alias_addr = SockaddrIn6::from(SocketAddrV6::new(addr, 0, 0, 0));

            let mut req: in6_ifreq = unsafe { std::mem::zeroed() };
            req.ifr_ifru.ifru_addr = *alias_addr.as_ref();
            copy_interface_name(interface, &mut req.ifr_name);

            unsafe { siocdifaddr_in6(ctl_socket.as_raw_fd(), &req as _) }.inspect_err(|e| {
                tracing::error!("Failed to remove alias {addr} from {interface}: {e}");
            })?;
        }
    }

    Ok(())
}

fn open_ctl_socket(family: AddressFamily) -> nix::Result<OwnedFd> {
    socket(family, SockType::Datagram, SockFlag::empty(), None).inspect_err(|e| {
        tracing::error!("Cannot connect to control socket ({family:?}): {e}");
    })
}

fn copy_interface_name(interface_name: &str, buf: &mut [c_char; IFNAMSIZ]) {
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

#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Copy, Clone)]
struct in6_aliasreq {
    pub ifra_name: [c_char; IFNAMSIZ],
    pub ifra_addr: sockaddr_in6,
    pub ifra_dstaddr: sockaddr_in6,
    pub ifra_prefixmask: sockaddr_in6,
    pub ifra_flags: c_int,
    pub ifra_lifetime: in6_addrlifetime,
}

#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Copy, Clone)]
struct in6_addrlifetime {
    pub ia6t_expire: time_t,
    pub ia6t_preferred: time_t,
    pub ia6t_vltime: u32,
    pub ia6t_pltime: u32,
}

/// Don't perform DAD on this address (used only at first SIOC* call)
const IN6_IFF_NODAD: i32 = 0x0020;

/// Returns IPv6 netmask with the prefix length.
fn ipv6_netmask(prefix_length: u8) -> Ipv6Addr {
    // Number of bits in an IPv6 address
    const IPV6_BITS: u8 = 128;
    let bits = if prefix_length >= IPV6_BITS {
        u128::MAX
    } else {
        u128::MAX
            .checked_shl((IPV6_BITS - prefix_length) as u32)
            .unwrap_or_default()
    };
    Ipv6Addr::from(bits)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_ipv6_netmask_prefix_24() {
        assert_eq!(ipv6_netmask(24), Ipv6Addr::from_str("ffff:ff00::").unwrap());
    }

    #[test]
    fn test_ipv6_netmask_prefix_0() {
        assert_eq!(ipv6_netmask(0), Ipv6Addr::from_str("::").unwrap());
    }

    #[test]
    fn test_ipv6_netmask_prefix_128_or_greater() {
        assert_eq!(
            ipv6_netmask(128),
            Ipv6Addr::from_str("ffff:ffff:ffff:ffff:ffff:ffff:ffff:ffff").unwrap()
        );
        assert_eq!(
            ipv6_netmask(255),
            Ipv6Addr::from_str("ffff:ffff:ffff:ffff:ffff:ffff:ffff:ffff").unwrap()
        );
    }
}
