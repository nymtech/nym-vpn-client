// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::ffi::{c_char, c_int};

use nix::libc::{IFNAMSIZ, ifreq, in6_addrlifetime, in6_ifreq, sockaddr, sockaddr_in6};

use crate::copy_into::CopyInto;

// bsd/sys/sockio.h
nix::ioctl_write_ptr!(siocsifflags, 'i', 16, ifreq);
nix::ioctl_readwrite!(siocgifflags, 'i', 17, ifreq);
nix::ioctl_write_ptr!(siocdifaddr, b'i', 25, ifreq);
nix::ioctl_write_ptr!(siocsifmtu, 'i', 52, ifreq);
nix::ioctl_readwrite!(siocgifmtu, 'i', 51, ifreq);
nix::ioctl_write_ptr!(siocaifaddr, 'i', 26, ifaliasreq);
nix::ioctl_write_ptr!(siocifddestroy, 'i', 121, ifreq);
nix::ioctl_readwrite!(siocifcreate2, 'i', 122, ifreq);

// bsd/netinet6/in6_var.h
nix::ioctl_write_ptr!(siocdifaddr_in6, b'i', 25, in6_ifreq);
nix::ioctl_write_ptr!(siocaifaddr_in6, 'i', 26, in6_aliasreq);
nix::ioctl_readwrite!(siocgifinfo_in6, 'i', 76, in6_ondireq);
nix::ioctl_readwrite!(siocsifinfo_flags, 'i', 87, in6_ndireq);

// usr/include/net/if.h
// see: https://github.com/rust-lang/libc/issues/4435
#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Copy, Clone)]
pub struct ifaliasreq {
    pub ifra_name: [c_char; IFNAMSIZ],
    pub ifra_addr: sockaddr,
    pub ifra_broadaddr: sockaddr,
    pub ifra_mask: sockaddr,
}

// usr/include/netinet6/in6_var.h
#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Copy, Clone)]
pub struct in6_aliasreq {
    pub ifra_name: [c_char; IFNAMSIZ],
    pub ifra_addr: sockaddr_in6,
    pub ifra_dstaddr: sockaddr_in6,
    pub ifra_prefixmask: sockaddr_in6,
    pub ifra_flags: c_int,
    pub ifra_lifetime: in6_addrlifetime,
}

// usr/include/netinet6/nd6.h
#[allow(non_camel_case_types)]
#[repr(C)]
pub struct in6_ondireq {
    pub ifname: [c_char; IFNAMSIZ],
    pub ndi: in6_ondireq_ndi,
}

// extracted from in6_ondireq.ndi
#[allow(non_camel_case_types)]
#[repr(C)]
pub struct in6_ondireq_ndi {
    pub linkmtu: u32,
    pub maxmtu: u32,
    pub basereachable: u32,
    pub reachable: u32,
    pub retrans: u32,
    pub flags: u32,
    pub recalctm: c_int,
    pub chlim: u8,
    pub receivedra: u8,
    pub collision_count: u8,
}

// usr/include/netinet6/nd6.h
#[allow(non_camel_case_types)]
#[repr(C)]
pub struct in6_ndireq {
    pub ifname: [c_char; IFNAMSIZ],
    pub ndi: nd_ifinfo,
}

// usr/include/netinet6/nd6.h
#[allow(non_camel_case_types)]
#[repr(C)]
pub struct nd_ifinfo {
    pub linkmtu: u32,
    pub maxmtu: u32,
    pub basereachable: u32,
    pub reachable: u32,
    pub retrans: u32,
    pub flags: u32,
    pub recalctm: c_int,
    pub chlim: u8,
    pub receivedra: u8,
    pub randomseed0: [u8; 8],
    pub randomseed1: [u8; 8],
    pub randomid: [u8; 8],
}

pub trait DefaultWithInterfaceExt {
    fn default_with_interface(interface: &str) -> Self;
}

impl DefaultWithInterfaceExt for ifreq {
    fn default_with_interface(interface: &str) -> ifreq {
        let mut req: ifreq = unsafe { std::mem::zeroed() };
        interface.copy_into(&mut req.ifr_name);
        req
    }
}

impl DefaultWithInterfaceExt for in6_ifreq {
    fn default_with_interface(interface: &str) -> in6_ifreq {
        let mut req: in6_ifreq = unsafe { std::mem::zeroed() };
        interface.copy_into(&mut req.ifr_name);
        req
    }
}

impl DefaultWithInterfaceExt for ifaliasreq {
    fn default_with_interface(interface: &str) -> ifaliasreq {
        let mut req: ifaliasreq = unsafe { std::mem::zeroed() };
        interface.copy_into(&mut req.ifra_name);
        req
    }
}

impl DefaultWithInterfaceExt for in6_aliasreq {
    fn default_with_interface(interface: &str) -> in6_aliasreq {
        let mut req: in6_aliasreq = unsafe { std::mem::zeroed() };
        interface.copy_into(&mut req.ifra_name);
        req
    }
}

impl DefaultWithInterfaceExt for in6_ondireq {
    fn default_with_interface(interface: &str) -> in6_ondireq {
        let mut req: in6_ondireq = unsafe { std::mem::zeroed() };
        interface.copy_into(&mut req.ifname);
        req
    }
}

impl DefaultWithInterfaceExt for in6_ndireq {
    fn default_with_interface(interface: &str) -> in6_ndireq {
        let mut req: in6_ndireq = unsafe { std::mem::zeroed() };
        interface.copy_into(&mut req.ifname);
        req
    }
}
