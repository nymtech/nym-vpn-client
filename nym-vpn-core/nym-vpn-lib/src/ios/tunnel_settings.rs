//! Packet tunnel network settings generator for iOS

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use ipnetwork::{Ipv4Network, Ipv6Network};
use itertools::{Either, Itertools};

/// Create tunnel settings for iOS packet tunnel.
///
/// * `interface_addresses` - a list of IP addresses to assign on tunnel interface
/// * `remote_endpoint` - remote endpoint that will be excluded from the tunnel
/// * `mtu` - tunnel device MTU.
pub fn create(
    interface_addresses: Vec<IpAddr>,
    remote_endpoint: IpAddr,
    dns_servers: Vec<IpAddr>,
    mtu: u16,
) -> TunnelNetworkSettings {
    let mut ipv4_settings: Option<Ipv4Settings> = None;
    let mut ipv6_settings: Option<Ipv6Settings> = None;

    // Assign tun interface IP and create IP settings.
    let (ipv4_interface_addrs, ipv6_interface_addrs): (Vec<_>, Vec<_>) = interface_addresses
        .into_iter()
        .partition_map(|addr| match addr {
            IpAddr::V4(address) => Either::Left(
                Ipv4Network::new(address, 32).expect("failed to create ipv4 addr with /32 prefix"),
            ),
            IpAddr::V6(address) => Either::Right(
                Ipv6Network::new(address, 128)
                    .expect("failed to create ipv6 addr with /128 prefix"),
            ),
        });

    if !ipv4_interface_addrs.is_empty() {
        ipv4_settings = Some(Ipv4Settings::new(ipv4_interface_addrs.clone()));
    }

    if !ipv6_interface_addrs.is_empty() {
        ipv6_settings = Some(Ipv6Settings::new(ipv6_interface_addrs.clone()));
    }

    // Add routes:
    //
    // - Add default route to pass all traffic over the tunnel.
    // - Exclude entry server IP to pass entry traffic outside of tunnel.
    match remote_endpoint {
        IpAddr::V4(entry_server_ipv4) => {
            if let Some(ipv4_settings) = ipv4_settings.as_mut() {
                ipv4_settings.included_routes = Some(
                    ipv4_interface_addrs
                        .into_iter()
                        .map(|addr_range| Ipv4Route::Specific {
                            destination: addr_range.network(),
                            subnet_mask: addr_range.mask(),
                            gateway: None,
                        })
                        .chain([Ipv4Route::Specific {
                            destination: Ipv4Addr::UNSPECIFIED,
                            subnet_mask: Ipv4Addr::UNSPECIFIED,
                            gateway: None,
                        }])
                        .collect(),
                );
                // ipv4_settings.excluded_routes = Some(vec![Ipv4Route::Specific {
                //     destination: entry_server_ipv4,
                //     subnet_mask: Ipv4Addr::BROADCAST,
                //     gateway: None,
                // }]);
            }
        }
        IpAddr::V6(entry_server_ipv6) => {
            if let Some(ipv6_settings) = ipv6_settings.as_mut() {
                ipv6_settings.included_routes = Some(
                    ipv6_interface_addrs
                        .into_iter()
                        .map(|addr_range| Ipv6Route::Specific {
                            destination: addr_range.network(),
                            prefix_length: addr_range.prefix(),
                            gateway: None,
                        })
                        .chain([Ipv6Route::Specific {
                            destination: Ipv6Addr::UNSPECIFIED,
                            prefix_length: 0,
                            gateway: None,
                        }])
                        .collect(),
                );
                // ipv6_settings.excluded_routes = Some(vec![Ipv6Route::Specific {
                //     destination: entry_server_ipv6,
                //     prefix_length: 128,
                //     gateway: None,
                // }]);
            }
        }
    }

    TunnelNetworkSettings {
        tunnel_remote_address: "127.0.0.1".to_owned(),
        ipv4_settings,
        ipv6_settings,
        dns_settings: Some(DnsSettings {
            servers: dns_servers,
            search_domains: None,
            // Empty string tells packet tunnel to resolve all DNS queries using tunne's DNS first.
            match_domains: Some(vec!["".to_owned()]),
        }),
        mtu,
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
