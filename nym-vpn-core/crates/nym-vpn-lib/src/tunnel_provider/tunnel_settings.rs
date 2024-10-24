// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Tunnel and network settings.

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use ipnetwork::{IpNetwork, Ipv4Network, Ipv6Network};
use itertools::{Either, Itertools};

#[derive(Debug)]
pub struct TunnelSettings {
    /// Tunnel interface addresses.
    pub interface_addresses: Vec<IpNetwork>,

    /// DNS servers to set on tunnel interface.
    pub dns_servers: Vec<IpAddr>,

    /// Tunnel remote addresses that will be excluded from being routed over the tunnel
    /// to prevent the network loop.
    pub remote_addresses: Vec<IpAddr>,

    /// Tunnel device MTU.
    pub mtu: u16,
}

impl TunnelSettings {
    /// Create tunnel network settings holding both networ and tunnel data necessary
    /// for tunnel provider/vpn service configuration on mobile.
    pub fn into_tunnel_network_settings(self) -> TunnelNetworkSettings {
        let (interface_addrs_ipv4, interface_addrs_ipv6) =
            Self::split_ipnet_addrs(self.interface_addresses);
        let (bypass_addrs_ipv4, bypass_addrs_ipv6) =
            Self::split_ipnet_addrs(Self::bypass_addresses(self.remote_addresses));

        let ipv4_settings = if interface_addrs_ipv4.is_empty() {
            None
        } else {
            Some(Self::ipv4_settings(interface_addrs_ipv4, bypass_addrs_ipv4))
        };

        let ipv6_settings = if interface_addrs_ipv6.is_empty() {
            None
        } else {
            Some(Self::ipv6_settings(interface_addrs_ipv6, bypass_addrs_ipv6))
        };

        TunnelNetworkSettings {
            tunnel_remote_address: "127.0.0.1".to_owned(),
            ipv4_settings,
            ipv6_settings,
            dns_settings: Some(DnsSettings {
                servers: self.dns_servers,
                search_domains: None,
                // Empty string tells packet tunnel to resolve all DNS queries using tunnel's DNS first.
                // todo: this might be very ios specific knowledge.
                match_domains: Some(vec!["".to_owned()]),
            }),
            mtu: self.mtu,
        }
    }

    fn ipv4_settings(
        interface_addresses: Vec<Ipv4Network>,
        bypass_addresses: Vec<Ipv4Network>,
    ) -> Ipv4Settings {
        let mut ipv4_settings = Ipv4Settings::new(interface_addresses.clone());

        ipv4_settings.included_routes = Some(
            interface_addresses
                .into_iter()
                .map(Ipv4Route::from)
                .chain([Ipv4Route::Specific {
                    // todo: consider using Ipv4Route::Default
                    destination: Ipv4Addr::UNSPECIFIED,
                    subnet_mask: Ipv4Addr::UNSPECIFIED,
                    gateway: None,
                }])
                .collect(),
        );

        if !bypass_addresses.is_empty() {
            ipv4_settings.excluded_routes =
                Some(bypass_addresses.into_iter().map(Ipv4Route::from).collect())
        }

        ipv4_settings
    }

    fn ipv6_settings(
        interface_addresses: Vec<Ipv6Network>,
        bypass_addresses: Vec<Ipv6Network>,
    ) -> Ipv6Settings {
        let mut ipv6_settings = Ipv6Settings::new(interface_addresses.clone());

        ipv6_settings.included_routes = Some(
            interface_addresses
                .into_iter()
                .map(Ipv6Route::from)
                .chain([Ipv6Route::Specific {
                    // todo: consider using Ipv6Route::Default
                    destination: Ipv6Addr::UNSPECIFIED,
                    prefix_length: 0,
                    gateway: None,
                }])
                .collect(),
        );

        if !bypass_addresses.is_empty() {
            ipv6_settings.excluded_routes =
                Some(bypass_addresses.into_iter().map(Ipv6Route::from).collect())
        }

        ipv6_settings
    }

    #[cfg(target_os = "ios")]
    fn bypass_addresses(_remote_addresses: Vec<IpAddr>) -> Vec<IpNetwork> {
        // Do not bypass remote addresses since connections initiated within the packet tunnel
        // bypass the tunnel interface anyway.
        vec![]
    }

    #[cfg(target_os = "android")]
    fn bypass_addresses(remote_addresses: Vec<IpAddr>) -> Vec<IpNetwork> {
        remote_addresses
            .into_iter()
            .map(|ip_addr| match ip_addr {
                IpAddr::V4(addr) => {
                    IpNetwork::V4(Ipv4Network::new(addr, 32).expect("remote_addr/32"))
                }
                IpAddr::V6(addr) => {
                    IpNetwork::V6(Ipv6Network::new(addr, 128).expect("remote_addr/128"))
                }
            })
            .collect()
    }

    fn split_ipnet_addrs(ipnet_addrs: Vec<IpNetwork>) -> (Vec<Ipv4Network>, Vec<Ipv6Network>) {
        ipnet_addrs.into_iter().partition_map(|addr| match addr {
            IpNetwork::V4(address) => Either::Left(
                Ipv4Network::new(address.ip(), address.prefix())
                    .expect("failed to create ipv4 addr with /32 prefix"),
            ),
            IpNetwork::V6(address) => Either::Right(
                Ipv6Network::new(address.ip(), address.prefix())
                    .expect("failed to create ipv6 addr with /128 prefix"),
            ),
        })
    }
}

#[derive(Debug, uniffi::Enum)]
pub enum Ipv4Route {
    /// Default IPv4 route (0.0.0.0/0)
    Default,
    /// Individual IPv4 route
    Specific {
        destination: Ipv4Addr,
        subnet_mask: Ipv4Addr,
        gateway: Option<Ipv4Addr>,
    },
}

impl From<Ipv4Network> for Ipv4Route {
    fn from(value: Ipv4Network) -> Self {
        Ipv4Route::Specific {
            destination: value.network(),
            subnet_mask: value.mask(),
            gateway: Some(value.ip()),
        }
    }
}

#[derive(Debug, uniffi::Enum)]
pub enum Ipv6Route {
    /// Default IPv6 route (::/0)
    Default,
    /// Individual IPv6 route
    Specific {
        destination: Ipv6Addr,
        prefix_length: u8,
        gateway: Option<Ipv6Addr>,
    },
}

impl From<Ipv6Network> for Ipv6Route {
    fn from(value: Ipv6Network) -> Self {
        Ipv6Route::Specific {
            destination: value.network(),
            prefix_length: value.prefix(),
            gateway: Some(value.ip()),
        }
    }
}

#[derive(Debug, Default, uniffi::Record)]
pub struct Ipv4Settings {
    /// IPv4 addresses that will be set on tunnel interface.
    pub addresses: Vec<Ipv4Network>,

    /// Traffic matching these routes will be routed over the tun interface.
    pub included_routes: Option<Vec<Ipv4Route>>,

    /// Traffic matching these routes will be routed over the primary physical interface.
    pub excluded_routes: Option<Vec<Ipv4Route>>,
}

impl Ipv4Settings {
    pub fn new(addresses: Vec<Ipv4Network>) -> Self {
        Self {
            addresses,
            ..Default::default()
        }
    }
}

#[derive(Debug, Default, uniffi::Record)]
pub struct Ipv6Settings {
    /// IPv4 addresses that will be set on tunnel interface.
    pub addresses: Vec<Ipv6Network>,

    /// Traffic matching these routes will be routed over the tun interface.
    pub included_routes: Option<Vec<Ipv6Route>>,

    /// Traffic matching these routes will be routed over the primary physical interface.
    pub excluded_routes: Option<Vec<Ipv6Route>>,
}

impl Ipv6Settings {
    pub fn new(addresses: Vec<Ipv6Network>) -> Self {
        Self {
            addresses,
            ..Default::default()
        }
    }
}

/// Tunnel + network settings
#[derive(Debug, uniffi::Record)]
pub struct TunnelNetworkSettings {
    /// Tunnel remote address, which is mostly of decorative value.
    pub tunnel_remote_address: String,

    /// IPv4 interface settings.
    pub ipv4_settings: Option<Ipv4Settings>,

    /// IPv6 interface settings.
    pub ipv6_settings: Option<Ipv6Settings>,

    /// DNS settings.
    pub dns_settings: Option<DnsSettings>,

    /// Tunnel device MTU.
    pub mtu: u16,
}

#[derive(Debug, uniffi::Record)]
pub struct DnsSettings {
    /// DNS IP addresses.
    pub servers: Vec<IpAddr>,

    /// DNS server search domains.
    pub search_domains: Option<Vec<String>>,

    /// Which domains to resolve using these DNS settings.
    pub match_domains: Option<Vec<String>>,
}
