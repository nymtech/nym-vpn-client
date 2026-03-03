// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    ffi::CStr,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6},
    os::fd::AsRawFd,
};

use ipnetwork::{IpNetwork, Ipv4Network, Ipv6Network};
use nix::{
    libc::{ifreq, in6_addrlifetime, in6_ifreq},
    net::if_::InterfaceFlags,
    sys::socket::{AddressFamily, SockaddrIn, SockaddrIn6, SockaddrLike, SockaddrStorage},
};

use super::{ctl_sockets::CtlSockets, sys::*};
use crate::{Error, ErrorKind, Result};

/// Network interface configuration session
#[derive(Debug, Default)]
pub struct Session {
    ctl_sockets: CtlSockets,
}

impl Session {
    /// Create new network interface
    ///
    /// Returns the name of newly created interface
    pub fn create_interface(&mut self, name: &str) -> Result<String> {
        let ctl_socket = self.ctl_sockets.ctl_socket_v4()?;
        let mut req = ifreq::default_with_interface(name);
        unsafe { siocifcreate2(ctl_socket.as_raw_fd(), &mut req)? };

        let bytes = req
            .ifr_name
            .into_iter()
            .map(|c| c as u8)
            .collect::<Vec<_>>();

        let name = CStr::from_bytes_until_nul(&bytes)
            .map_err(|err| Error::new(ErrorKind::ConvertInterfaceNameIntoString, Box::new(err)))?
            .to_str()
            .map_err(|err| Error::new(ErrorKind::ConvertInterfaceNameIntoString, Box::new(err)))
            .map(ToOwned::to_owned)?;

        Ok(name)
    }

    /// Destroy interface by name
    pub fn destroy_interface(&mut self, name: &str) -> Result<()> {
        let ctl_socket = self.ctl_sockets.ctl_socket_v4()?;
        let req = ifreq::default_with_interface(name);
        unsafe { siocifddestroy(ctl_socket.as_raw_fd(), &req)? };
        Ok(())
    }

    /// Add or change an IP address on network interface.
    ///
    /// ## Platform behavior
    ///
    /// As of macOS 14.7:
    ///
    /// - Calling this method with the IPv4 address that is already assigned on the interface
    ///   will properly update destination and netmask.
    ///
    /// - Calling this method with the IPv6 address that is already assigned on the interface is a no-op.
    ///   Prefix length, when differs, will not be applied. The IPv6 address needs to be removed first.
    ///
    pub fn add_address(
        &mut self,
        interface: &str,
        add_request: impl Into<AddAddressRequest>,
    ) -> Result<()> {
        match add_request.into() {
            AddAddressRequest::V4(add_request) => {
                let ctl_socket = self.ctl_sockets.ctl_socket_v4()?;
                let alias_addr = SockaddrIn::from(SocketAddrV4::new(add_request.address.ip(), 0));
                let destination = add_request
                    .destination
                    .map(|ip| SockaddrIn::from(SocketAddrV4::new(ip, 0)));
                let netmask = SockaddrIn::from(SocketAddrV4::new(add_request.address.mask(), 0));

                let mut req = ifaliasreq::default_with_interface(interface);
                req.ifra_addr = unsafe { *alias_addr.as_ptr() };
                if let Some(destination) = destination {
                    req.ifra_broadaddr = unsafe { *destination.as_ptr() };
                }
                req.ifra_mask = unsafe { *netmask.as_ptr() };

                unsafe { siocaifaddr(ctl_socket.as_raw_fd(), &req as _) }?;
                Ok(())
            }
            AddAddressRequest::V6(add_request) => {
                let ctl_socket = self.ctl_sockets.ctl_socket_v6()?;
                let alias_addr =
                    SockaddrIn6::from(SocketAddrV6::new(add_request.address.ip(), 0, 0, 0));
                let destination = add_request
                    .destination
                    .map(|ip| SockaddrIn6::from(SocketAddrV6::new(ip, 0, 0, 0)));
                let netmask = add_request.address.mask();
                let prefix_mask = SockaddrIn6::from(SocketAddrV6::new(netmask, 0, 0, 0));

                let mut req = in6_aliasreq::default_with_interface(interface);
                req.ifra_addr = *alias_addr.as_ref();
                req.ifra_prefixmask = *prefix_mask.as_ref();
                if let Some(destination) = destination {
                    req.ifra_dstaddr = *destination.as_ref();
                }
                req.ifra_lifetime = add_request.lifetime.0;
                req.ifra_flags = add_request.flags.bits();

                unsafe { siocaifaddr_in6(ctl_socket.as_raw_fd(), &req as _) }?;
                Ok(())
            }
        }
    }

    /// Remove an IP address from a network interface.
    pub fn remove_address(&mut self, interface: &str, addr: IpAddr) -> Result<()> {
        match addr {
            IpAddr::V4(addr) => {
                let ctl_socket = self.ctl_sockets.ctl_socket_v4()?;
                let alias_addr = SockaddrIn::from(SocketAddrV4::new(addr, 0));

                let mut req = ifreq::default_with_interface(interface);
                req.ifr_ifru.ifru_addr = unsafe { *alias_addr.as_ptr() };

                unsafe { siocdifaddr(ctl_socket.as_raw_fd(), &req as _) }?;
            }
            IpAddr::V6(addr) => {
                let ctl_socket = self.ctl_sockets.ctl_socket_v6()?;
                let alias_addr = SockaddrIn6::from(SocketAddrV6::new(addr, 0, 0, 0));

                let mut req = in6_ifreq::default_with_interface(interface);
                req.ifr_ifru.ifru_addr = *alias_addr.as_ref();

                unsafe { siocdifaddr_in6(ctl_socket.as_raw_fd(), &req as _) }?;
            }
        }

        Ok(())
    }

    /// Get IP addresses assigned on interface.
    pub fn addresses(&self, interface: &str) -> Result<Vec<InterfaceIpAddrEntry>> {
        let mut entries = vec![];
        for ifaddr in nix::ifaddrs::getifaddrs()? {
            if ifaddr.interface_name == interface {
                let Some(address) = ifaddr.address.and_then(socket_addr_from_sockaddr_storage)
                else {
                    continue;
                };
                let Some(netmask) = ifaddr.netmask.and_then(socket_addr_from_sockaddr_storage)
                else {
                    continue;
                };

                let address = IpNetwork::with_netmask(address.ip(), netmask.ip())
                    .map_err(|err| Error::new(ErrorKind::InvalidAddress, Box::new(err)))?;

                let entry = InterfaceIpAddrEntry {
                    address,
                    broadcast: ifaddr
                        .broadcast
                        .and_then(socket_addr_from_sockaddr_storage)
                        .map(|v| v.ip()),
                    destination: ifaddr
                        .destination
                        .and_then(socket_addr_from_sockaddr_storage)
                        .map(|v| v.ip()),
                };
                entries.push(entry);
            }
        }

        Ok(entries)
    }

    /// Get device MTU
    pub fn mtu(&mut self, interface: &str) -> Result<i32> {
        let ctl_socket = self.ctl_sockets.ctl_socket_v4()?;
        let mut req = ifreq::default_with_interface(interface);
        Ok(unsafe {
            siocgifmtu(ctl_socket.as_raw_fd(), &mut req)?;
            req.ifr_ifru.ifru_mtu
        })
    }

    /// Set device MTU
    pub fn set_mtu(&mut self, interface: &str, mtu: i32) -> Result<()> {
        let ctl_socket = self.ctl_sockets.ctl_socket_v4()?;
        let mut req = ifreq::default_with_interface(interface);
        req.ifr_ifru.ifru_mtu = mtu;
        unsafe {
            siocsifmtu(ctl_socket.as_raw_fd(), &req)?;
        }
        Ok(())
    }

    /// Get interface flags
    pub fn interface_flags(&mut self, interface: &str) -> Result<InterfaceFlags> {
        let ctl_socket = self.ctl_sockets.ctl_socket_v4()?;
        let mut req = ifreq::default_with_interface(interface);

        Ok(unsafe {
            siocgifflags(ctl_socket.as_raw_fd(), &mut req)?;
            InterfaceFlags::from_bits_retain(req.ifr_ifru.ifru_flags as _)
        })
    }

    /// Set interface flags
    pub fn set_interface_flags(
        &mut self,
        interface: &str,
        interface_flags: InterfaceFlags,
    ) -> Result<()> {
        let ctl_socket = self.ctl_sockets.ctl_socket_v4()?;
        let mut req = ifreq::default_with_interface(interface);
        req.ifr_ifru.ifru_flags = interface_flags.bits() as _;
        unsafe { siocsifflags(ctl_socket.as_raw_fd(), &req)? };
        Ok(())
    }

    /// Get neighbour discovery info
    pub fn nd6_info(&mut self, interface: &str) -> Result<Nd6Info> {
        let ctl_socket = self.ctl_sockets.ctl_socket_v6()?;
        let mut req = in6_ondireq::default_with_interface(interface);

        unsafe { siocgifinfo_in6(ctl_socket.as_raw_fd(), &mut req as _) }?;

        Ok(Nd6Info::from(req.ndi))
    }

    /// Set neighbour discovery flags
    pub fn set_nd6_flags(&mut self, interface: &str, nd6_flags: Nd6Flags) -> Result<()> {
        let ctl_socket = self.ctl_sockets.ctl_socket_v6()?;
        let mut req = in6_ndireq::default_with_interface(interface);
        req.ndi.flags = nd6_flags.bits();

        unsafe { siocsifinfo_flags(ctl_socket.as_raw_fd(), &mut req as _) }?;

        Ok(())
    }
}

/// Request to add IPv4 or IPv6 address on the interface
#[derive(Debug, Clone, Copy)]
pub enum AddAddressRequest {
    V4(AddAddressRequestV4),
    V6(AddAddressRequestV6),
}

impl From<AddAddressRequestV4> for AddAddressRequest {
    fn from(value: AddAddressRequestV4) -> Self {
        Self::V4(value)
    }
}

impl From<AddAddressRequestV6> for AddAddressRequest {
    fn from(value: AddAddressRequestV6) -> Self {
        Self::V6(value)
    }
}

/// Request to add IPv4 address on the interface
#[derive(Debug, Clone, Copy)]
pub struct AddAddressRequestV4 {
    /// Interface IP address
    pub address: Ipv4Network,

    /// Destination address for point-to-point interfaces
    pub destination: Option<Ipv4Addr>,
}

/// Request to add IPv6 address on the interface
#[derive(Debug, Clone, Copy)]
pub struct AddAddressRequestV6 {
    /// Interface IP address
    pub address: Ipv6Network,

    /// Destination address for point-to-point interfaces (prefix must be set to 128)
    pub destination: Option<Ipv6Addr>,

    /// Address lifetime
    pub lifetime: Ipv6AddrLifetime,

    /// IPv6 address flags
    /// <https://github.com/apple-oss-distributions/xnu/blob/main/bsd/netinet6/in6_var.h#L814-L841>
    pub flags: Ipv6AddrFlags,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct InterfaceIpAddrEntry {
    /// Interface IP address with prefix
    pub address: IpNetwork,
    pub broadcast: Option<IpAddr>,
    pub destination: Option<IpAddr>,
}

bitflags::bitflags! {
    /// IPv6 address flags
    #[derive(Debug, Clone, Copy, Eq, PartialEq)]
    pub struct Ipv6AddrFlags: i32 {
        /// Anycast address
        const IN6_IFF_ANYCAST = 0x0001;
        /// Tentative address
        const IN6_IFF_TENTATIVE = 0x0002;
        /// DAD detected duplicate
        const IN6_IFF_DUPLICATED = 0x0004;
        const IN6_IFF_DETACHED = 0x0008;
        /// Don't perform DAD on this address
        const IN6_IFF_NODAD = 0x0020;
        /// Autoconfigurable address
        const IN6_IFF_AUTOCONF = 0x0040;
        /// Temporary (anonymous) address
        const IN6_IFF_TEMPORARY = 0x0080;
        /// Assigned by DHCPv6 service
        const IN6_IFF_DYNAMIC = 0x0100;
        /// Optimistic DAD, i.e. RFC 4429
        const IN6_IFF_OPTIMISTIC = 0x0200;
        /// Cryptographically generated
        const IN6_IFF_SECURED = 0x0400;
        /// Address reserved for CLAT46
        const IN6_IFF_CLAT46 = 0x1000;
    }
}

bitflags::bitflags! {
    /// Neighbor discovery flags
    #[derive(Debug, Clone, Copy, Eq, PartialEq)]
    pub struct Nd6Flags: u32 {
        const ND6_IFF_PERFORMNUD = 0x1;
        const ND6_IFF_PROXY_PREFIXES = 0x20;
        const ND6_IFF_IGNORE_NA = 0x40;
        const ND6_IFF_REPLICATED = 0x100;
        /// Perform DAD on the interface
        const ND6_IFF_DAD = 0x200;
    }
}

pub const ND6_INFINITE_LIFETIME: u32 = 0xffffffff;
pub const ND6_MAX_LIFETIME: u32 = 0x7fffffff;

/// Neighbor discovery information
#[derive(Debug, Clone, Copy)]
pub struct Nd6Info {
    pub linkmtu: u32,
    pub maxmtu: u32,
    pub basereachable: u32,
    pub reachable: u32,
    pub retrans: u32,
    pub flags: Nd6Flags,
    pub recalctm: i32,
    pub chlim: u8,
    pub receivedra: u8,
    pub collision_count: u8,
}

impl From<in6_ondireq_ndi> for Nd6Info {
    fn from(value: in6_ondireq_ndi) -> Self {
        Self {
            linkmtu: value.linkmtu,
            maxmtu: value.maxmtu,
            basereachable: value.basereachable,
            reachable: value.reachable,
            retrans: value.retrans,
            flags: Nd6Flags::from_bits_retain(value.flags),
            recalctm: value.recalctm,
            chlim: value.chlim,
            receivedra: value.receivedra,
            collision_count: value.collision_count,
        }
    }
}

/// IPv6 address lifetime
#[repr(transparent)]
#[derive(Debug, Clone, Copy)]
pub struct Ipv6AddrLifetime(pub in6_addrlifetime);

impl Default for Ipv6AddrLifetime {
    fn default() -> Self {
        Self(in6_addrlifetime {
            ia6t_expire: 0,
            ia6t_preferred: 0,
            ia6t_vltime: ND6_INFINITE_LIFETIME,
            ia6t_pltime: ND6_INFINITE_LIFETIME,
        })
    }
}

// Converts `SockaddrStorage` to SocketAddr` if possible, otherwise returns `None`
fn socket_addr_from_sockaddr_storage(sin: SockaddrStorage) -> Option<SocketAddr> {
    match sin.family()? {
        AddressFamily::Inet => Some(SocketAddr::from(*sin.as_sockaddr_in()?)),
        AddressFamily::Inet6 => Some(SocketAddr::from(*sin.as_sockaddr_in6()?)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use std::{io::Write, net::IpAddr};

    use super::{super::utun::Utun, *};

    #[test]
    #[serial_test::serial]
    fn test_create_fake_ethernet() {
        let mut sess = Session::default();

        let interface = sess.create_interface("feth").unwrap();
        assert!(interface.starts_with("feth"));
        sess.destroy_interface(&interface).unwrap();
    }

    #[test]
    #[serial_test::serial]
    fn test_add_p2p_ipv4_address() {
        let tun = Utun::new().unwrap();
        let interface = tun.name().unwrap();

        let ipv4_addr: Ipv4Network = "10.2.0.10/32".parse().unwrap();
        let ipv4_destination: Ipv4Addr = "10.2.0.1".parse().unwrap();
        let req = AddAddressRequest::V4(AddAddressRequestV4 {
            address: ipv4_addr,
            destination: Some(ipv4_destination),
        });

        let mut sess = Session::default();
        sess.add_address(&interface, req).unwrap();
        assert!(
            sess.addresses(&interface)
                .unwrap()
                .iter()
                .any(|addr| addr.address == IpNetwork::from(ipv4_addr)
                    && addr.destination == Some(ipv4_destination.into()))
        );
        ifconfig(&interface);

        sess.remove_address(&interface, ipv4_addr.ip().into())
            .unwrap();
        ifconfig(&interface);
        assert!(
            sess.addresses(&interface)
                .unwrap()
                .iter()
                .all(|addr| addr.address != IpNetwork::from(ipv4_addr))
        );
    }

    #[test]
    #[serial_test::serial]
    fn test_add_p2p_ipv6_address() {
        let tun = Utun::new().unwrap();
        let interface = tun.name().unwrap();

        let ipv6_addr: Ipv6Network = "fdc0:9d3b:6a16::/128".parse().unwrap();
        let req = AddAddressRequest::V6(AddAddressRequestV6 {
            address: ipv6_addr,
            destination: None,
            lifetime: Ipv6AddrLifetime::default(),
            flags: Ipv6AddrFlags::IN6_IFF_NODAD,
        });

        let mut sess = Session::default();
        sess.add_address(&interface, req).unwrap();
        assert!(
            sess.addresses(&interface)
                .unwrap()
                .iter()
                .any(|addr| addr.address == IpNetwork::from(ipv6_addr))
        );
        ifconfig(&interface);

        sess.remove_address(&interface, IpAddr::from(ipv6_addr.ip()))
            .unwrap();
        assert!(
            sess.addresses(&interface)
                .unwrap()
                .iter()
                .all(|addr| addr.address.ip() != ipv6_addr.ip())
        );
        ifconfig(&interface);
    }

    #[test]
    #[serial_test::serial]
    fn test_add_feth_ipv4_address() {
        let mut sess = Session::default();

        let interface = sess.create_interface("feth").unwrap();
        let _guard = DropGuard::new(|| Session::default().destroy_interface(&interface).unwrap());

        let addr: Ipv4Network = "10.2.0.10/32".parse().unwrap();
        let req = AddAddressRequest::V4(AddAddressRequestV4 {
            address: addr,
            destination: None,
        });
        sess.add_address(&interface, req)
            .expect("failed to add the first IPv4");
        assert!(
            sess.addresses(&interface)
                .unwrap()
                .iter()
                .any(|vaddr| vaddr.address == IpNetwork::from(addr))
        );
        ifconfig(&interface);
    }

    #[test]
    #[serial_test::serial]
    fn test_change_feth_ipv4_prefix() {
        let mut sess = Session::default();
        let interface = sess.create_interface("feth").unwrap();
        let _guard = DropGuard::new(|| {
            Session::default()
                .destroy_interface(&interface)
                .expect("failed to destroy interface")
        });

        let addr: Ipv4Network = "10.2.0.10/32".parse().unwrap();
        let req = AddAddressRequest::V4(AddAddressRequestV4 {
            address: addr,
            destination: None,
        });

        sess.add_address(&interface, req)
            .expect("failed to add the first IPv4");
        assert!(
            sess.addresses(&interface)
                .unwrap()
                .iter()
                .any(|vaddr| vaddr.address == IpNetwork::from(addr))
        );
        ifconfig(&interface);

        let addr: Ipv4Network = "10.2.0.10/24".parse().unwrap();
        let req = AddAddressRequest::V4(AddAddressRequestV4 {
            address: addr,
            destination: None,
        });
        sess.add_address(&interface, req).unwrap();
        ifconfig(&interface);
        assert!(
            sess.addresses(&interface)
                .unwrap()
                .iter()
                .any(|vaddr| vaddr.address == IpNetwork::from(addr))
        );
    }

    #[test]
    #[serial_test::serial]
    fn test_add_feth_ipv6_address() {
        let mut sess = Session::default();
        let interface = sess.create_interface("feth").unwrap();
        let _guard = DropGuard::new(|| Session::default().destroy_interface(&interface).unwrap());
        ifconfig(&interface);

        let addr: Ipv6Network = "fdc0:9d3b:6a16::/64".parse().unwrap();
        let req = AddAddressRequest::V6(AddAddressRequestV6 {
            address: addr,
            destination: None,
            lifetime: Ipv6AddrLifetime::default(),
            flags: Ipv6AddrFlags::IN6_IFF_ANYCAST | Ipv6AddrFlags::IN6_IFF_NODAD,
        });
        sess.add_address(&interface, req).unwrap();
        assert!(
            sess.addresses(&interface)
                .unwrap()
                .iter()
                .any(|vaddr| vaddr.address == IpNetwork::from(addr))
        );
        ifconfig(&interface);
    }

    #[test]
    #[serial_test::serial]
    fn test_macos_does_not_change_feth_ipv6_prefix() {
        let mut sess = Session::default();
        let interface = sess.create_interface("feth").unwrap();
        let _guard = DropGuard::new(|| Session::default().destroy_interface(&interface).unwrap());

        let addr1: Ipv6Network = "fdc0:9d3b:6a16::/128".parse().unwrap();
        let req = AddAddressRequest::V6(AddAddressRequestV6 {
            address: addr1,
            destination: None,
            lifetime: Ipv6AddrLifetime::default(),
            flags: Ipv6AddrFlags::IN6_IFF_NODAD,
        });
        sess.add_address(&interface, req).unwrap();
        assert!(
            sess.addresses(&interface)
                .unwrap()
                .iter()
                .any(|vaddr| vaddr.address == IpNetwork::from(addr1))
        );
        ifconfig(&interface);

        let addr2: Ipv6Network = "fdc0:9d3b:6a16::/64".parse().unwrap();
        let req = AddAddressRequest::V6(AddAddressRequestV6 {
            address: addr2,
            destination: None,
            lifetime: Ipv6AddrLifetime::default(),
            flags: Ipv6AddrFlags::IN6_IFF_NODAD,
        });
        sess.add_address(&interface, req).unwrap();

        // macOS 14.7+ does not update IPv6 prefix!
        assert!(
            sess.addresses(&interface)
                .unwrap()
                .iter()
                .any(|vaddr| vaddr.address.ip() == IpAddr::from(addr2.ip())
                    && vaddr.address.prefix() == addr1.prefix())
        );
        ifconfig(&interface);
    }

    #[test]
    #[serial_test::serial]
    fn test_add_loopback_ipv4_alias() {
        let interface = "lo0";
        let ipv4_addr: Ipv4Network = "127.24.32.254/16".parse().unwrap();
        let mut sess = Session::default();
        sess.add_address(
            interface,
            AddAddressRequest::V4(AddAddressRequestV4 {
                address: ipv4_addr,
                destination: None,
            }),
        )
        .unwrap();
        assert!(
            sess.addresses(interface)
                .unwrap()
                .iter()
                .any(|addr| addr.address.ip() != ipv4_addr.ip())
        );
        ifconfig(interface);
        sess.remove_address(interface, IpAddr::from(ipv4_addr.ip()))
            .unwrap();
        ifconfig(interface);
        assert!(
            sess.addresses(interface)
                .unwrap()
                .iter()
                .all(|addr| addr.address.ip() != ipv4_addr.ip())
        );
    }

    #[test]
    #[serial_test::serial]
    fn test_set_nd6_flags() {
        let tun = Utun::new().unwrap();
        let interface = tun.name().unwrap();

        let mut sess = Session::default();
        ifconfig(&interface);

        let ndi = sess.nd6_info(&interface).unwrap();
        let new_flags = if ndi.flags.contains(Nd6Flags::ND6_IFF_PERFORMNUD) {
            ndi.flags - Nd6Flags::ND6_IFF_PERFORMNUD
        } else {
            ndi.flags | Nd6Flags::ND6_IFF_PERFORMNUD
        };

        sess.set_nd6_flags(&interface, new_flags).unwrap();
        ifconfig(&interface);

        let ndi = sess.nd6_info(&interface).unwrap();
        assert_eq!(ndi.flags, new_flags);
    }

    #[test]
    #[serial_test::serial]
    fn test_set_mtu() {
        use rand::Rng;

        let tun = Utun::new().unwrap();
        let interface = tun.name().unwrap();
        let mut sess = Session::default();

        let mtu = rand::thread_rng().gen_range(1280..=1480);
        sess.set_mtu(&interface, mtu).unwrap();

        let current_mtu = sess.mtu(&interface).unwrap();
        assert_eq!(mtu, current_mtu);

        ifconfig(&interface);
    }

    fn ifconfig(name: &str) {
        let output = std::process::Command::new("ifconfig")
            .arg(name)
            .output()
            .unwrap();

        std::io::stdout().write_all(&output.stdout).unwrap();
        std::io::stderr().write_all(&output.stderr).unwrap();
    }

    struct DropGuard<F: FnOnce()> {
        on_drop: Option<F>,
    }

    impl<F: FnOnce()> DropGuard<F> {
        fn new(on_drop: F) -> Self {
            Self {
                on_drop: Some(on_drop),
            }
        }
    }

    impl<F: FnOnce()> Drop for DropGuard<F> {
        fn drop(&mut self) {
            if let Some(on_drop) = self.on_drop.take() {
                on_drop();
            }
        }
    }
}
