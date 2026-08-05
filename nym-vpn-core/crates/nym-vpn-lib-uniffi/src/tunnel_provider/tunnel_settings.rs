// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Tunnel and network settings.

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

#[cfg(target_os = "android")]
use ipnet::{Ipv4Net, Ipv6Net};
use ipnetwork::{IpNetwork, Ipv4Network, Ipv6Network};
#[cfg(target_os = "android")]
use iprange::IpRange;
use itertools::{Either, Itertools};

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

#[uniffi::export]
impl Ipv4Route {
    pub fn prefix_length(&self) -> u8 {
        match self {
            Self::Default => 0,
            Self::Specific { subnet_mask, .. } => {
                ipnetwork::ipv4_mask_to_prefix(*subnet_mask).unwrap_or(32)
            }
        }
    }

    pub fn destination(&self) -> Ipv4Addr {
        match self {
            Self::Default => Ipv4Addr::UNSPECIFIED,
            Self::Specific { destination, .. } => *destination,
        }
    }
}

#[cfg(target_os = "android")]
impl Ipv4Route {
    fn as_ipv4net(&self) -> Option<Ipv4Net> {
        let addr = self.destination();
        let prefix = self.prefix_length();

        Ipv4Net::new(self.destination(), self.prefix_length())
            .inspect_err(|err| {
                tracing::error!("Failed to create Ipv4Net from {addr}/{prefix}: {err}");
            })
            .ok()
    }
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

#[uniffi::export]
impl Ipv6Route {
    pub fn destination(&self) -> Ipv6Addr {
        match self {
            Self::Default => Ipv6Addr::UNSPECIFIED,
            Self::Specific { destination, .. } => *destination,
        }
    }

    pub fn prefix_length(&self) -> u8 {
        match self {
            Self::Default => 0,
            Self::Specific { prefix_length, .. } => *prefix_length,
        }
    }
}

#[cfg(target_os = "android")]
impl Ipv6Route {
    fn as_ipv6net(&self) -> Option<Ipv6Net> {
        let addr = self.destination();
        let prefix = self.prefix_length();
        Ipv6Net::new(addr, prefix)
            .inspect_err(|err| {
                tracing::error!("Failed to create Ipv6Net from {addr}/{prefix}: {err}");
            })
            .ok()
    }
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

#[cfg(target_os = "android")]
impl Ipv4Settings {
    /// Returns tunneled IPv4 networks based on included and excluded routes.
    pub fn tunnel_networks(&self) -> IpRange<Ipv4Net> {
        let mut include_range = IpRange::<Ipv4Net>::new();
        if let Some(included_routes) = self.included_routes.as_ref() {
            for route in included_routes {
                if let Some(ipnet) = route.as_ipv4net() {
                    include_range.add(ipnet);
                }
            }
        }

        let mut exclude_range = IpRange::<Ipv4Net>::new();
        if let Some(excluded_routes) = self.excluded_routes.as_ref() {
            for route in excluded_routes {
                if let Some(ipnet) = route.as_ipv4net() {
                    exclude_range.add(ipnet);
                }
            }
        }

        include_range.simplify();
        exclude_range.simplify();
        include_range.exclude(&exclude_range)
    }
}

#[derive(Debug, Default, uniffi::Record)]
pub struct Ipv6Settings {
    /// IPv6 addresses that will be set on tunnel interface.
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

#[cfg(target_os = "android")]
impl Ipv6Settings {
    /// Returns tunneled IPv6 networks based on included and excluded routes.
    pub fn tunnel_networks(&self) -> IpRange<Ipv6Net> {
        let mut include_range = IpRange::<Ipv6Net>::new();
        if let Some(included_routes) = self.included_routes.as_ref() {
            for route in included_routes {
                if let Some(ipnet) = route.as_ipv6net() {
                    include_range.add(ipnet);
                }
            }
        }

        let mut exclude_range = IpRange::<Ipv6Net>::new();
        if let Some(excluded_routes) = self.excluded_routes.as_ref() {
            for route in excluded_routes {
                if let Some(ipnet) = route.as_ipv6net() {
                    exclude_range.add(ipnet);
                }
            }
        }

        include_range.simplify();
        exclude_range.simplify();
        include_range.exclude(&exclude_range)
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

    /// When true on Android, exclude the VPN app from the tunnel (blocking placeholder).
    pub exclude_vpn_app: bool,
}

#[cfg(target_os = "android")]
#[uniffi::export]
impl TunnelNetworkSettings {
    /// Returns CIDRs for all tunnel networks excluding LAN networks when `allow_lan` is true.
    pub fn compute_tunnel_networks(&self, allow_lan: bool) -> Vec<String> {
        use nym_firewall_config::{ALLOWED_LAN_MULTICAST_NETS, ALLOWED_LAN_NETS};

        let mut tunnel_ipv4 = self
            .ipv4_settings
            .as_ref()
            .map(|v| v.tunnel_networks())
            .unwrap_or_default();
        let mut tunnel_ipv6 = self
            .ipv6_settings
            .as_ref()
            .map(|v| v.tunnel_networks())
            .unwrap_or_default();

        if allow_lan {
            let mut exclude_ipv4_lan = IpRange::<Ipv4Net>::new();
            let mut exclude_ipv6_lan = IpRange::<Ipv6Net>::new();

            for network in ALLOWED_LAN_NETS
                .iter()
                .chain(ALLOWED_LAN_MULTICAST_NETS.iter())
            {
                match network {
                    IpNetwork::V4(address) => match Ipv4Net::new(address.ip(), address.prefix()) {
                        Ok(ipv4_net) => {
                            exclude_ipv4_lan.add(ipv4_net);
                        }
                        Err(e) => {
                            tracing::error!("Failed to create IPv4 network for {}: {}", address, e)
                        }
                    },
                    IpNetwork::V6(address) => match Ipv6Net::new(address.ip(), address.prefix()) {
                        Ok(ipv6_net) => {
                            exclude_ipv6_lan.add(ipv6_net);
                        }
                        Err(e) => {
                            tracing::error!("Failed to create IPv6 network for {}: {}", address, e)
                        }
                    },
                }
            }

            tunnel_ipv4 = tunnel_ipv4.exclude(&exclude_ipv4_lan);
            tunnel_ipv6 = tunnel_ipv6.exclude(&exclude_ipv6_lan);
        }

        tunnel_ipv4
            .into_iter()
            .map(|ip| ip.to_string())
            .chain(tunnel_ipv6.into_iter().map(|ip| ip.to_string()))
            .collect()
    }
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

impl From<nym_vpn_lib::tunnel_provider::TunnelSettings> for TunnelNetworkSettings {
    fn from(settings: nym_vpn_lib::tunnel_provider::TunnelSettings) -> Self {
        let (interface_addrs_ipv4, interface_addrs_ipv6) =
            Self::split_ipnet_addrs(settings.interface_addresses);
        let (bypass_addrs_ipv4, bypass_addrs_ipv6) = Self::split_ipnet_addrs(
            Self::bypass_addresses(&settings.remote_addresses, &settings.dns_servers),
        );

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
                servers: settings.dns_servers,
                search_domains: None,
                // Empty string tells packet tunnel to resolve all DNS queries using tunnel's DNS first.
                // todo: this might be very ios specific knowledge.
                match_domains: Some(vec!["".to_owned()]),
            }),
            mtu: settings.mtu,
            exclude_vpn_app: settings.exclude_vpn_app,
        }
    }
}

impl TunnelNetworkSettings {
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
    fn bypass_addresses(_remote_addresses: &[IpAddr], _dns_servers: &[IpAddr]) -> Vec<IpNetwork> {
        // Do not bypass remote addresses since connections initiated within the packet tunnel
        // bypass the tunnel interface anyway.
        vec![]
    }

    #[cfg(target_os = "android")]
    fn bypass_addresses(remote_addresses: &[IpAddr], dns_servers: &[IpAddr]) -> Vec<IpNetwork> {
        // Allow local DNS servers to escape the tunnel since local DNS cannot be routed over the tunnel.
        let local_dns_servers = dns_servers
            .iter()
            .filter(|addr| nym_firewall_config::is_local_address(addr))
            .copied()
            .collect::<Vec<_>>();

        remote_addresses
            .iter()
            .copied()
            .chain(local_dns_servers)
            .map(IpNetwork::from)
            .collect()
    }

    fn split_ipnet_addrs(ipnet_addrs: Vec<IpNetwork>) -> (Vec<Ipv4Network>, Vec<Ipv6Network>) {
        ipnet_addrs.into_iter().partition_map(|addr| match addr {
            IpNetwork::V4(address) => Either::Left(address),
            IpNetwork::V6(address) => Either::Right(address),
        })
    }
}
