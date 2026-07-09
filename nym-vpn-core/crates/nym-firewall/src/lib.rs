// Copyright 2025 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(any(target_os = "linux", target_os = "macos"))]
use std::net::Ipv6Addr;
use std::{borrow::Cow, fmt, net::IpAddr};

#[cfg(any(target_os = "linux", target_os = "macos"))]
use ipnetwork::Ipv6Network;

#[cfg(target_os = "linux")]
use nym_cgroup::v2::CGroup2;

#[cfg(target_os = "macos")]
#[path = "macos.rs"]
mod imp;

#[cfg(target_os = "linux")]
#[path = "linux.rs"]
mod imp;

#[cfg(windows)]
#[path = "windows/mod.rs"]
mod imp;

#[cfg(target_os = "android")]
#[path = "android.rs"]
mod imp;

#[cfg(target_os = "ios")]
#[path = "ios.rs"]
mod imp;

mod net;
#[cfg(not(target_os = "android"))]
pub use net::AllowedDns;
pub use net::{
    AllowedClients, AllowedEndpoint, AllowedTunnelTraffic, Endpoint, TransportProtocol,
    TunnelInterface, TunnelMetadata,
};

pub use self::imp::Error;

#[cfg(any(target_os = "linux", target_os = "macos"))]
const IPV6_LINK_LOCAL: Ipv6Network =
    Ipv6Network::new_checked(Ipv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, 0), 10).unwrap();
/// The allowed target addresses of outbound DHCPv6 requests
#[cfg(any(target_os = "linux", target_os = "macos"))]
const DHCPV6_SERVER_ADDRS: [Ipv6Addr; 2] = [
    Ipv6Addr::new(0xff02, 0, 0, 0, 0, 0, 1, 2),
    Ipv6Addr::new(0xff05, 0, 0, 0, 0, 0, 1, 3),
];
#[cfg(any(target_os = "linux", target_os = "macos"))]
const ROUTER_SOLICITATION_OUT_DST_ADDR: Ipv6Addr = Ipv6Addr::new(0xff02, 0, 0, 0, 0, 0, 0, 2);
#[cfg(any(target_os = "linux", target_os = "macos"))]
const SOLICITED_NODE_MULTICAST: Ipv6Network =
    Ipv6Network::new_checked(Ipv6Addr::new(0xff02, 0, 0, 0, 0, 1, 0xFF00, 0), 104).unwrap();

#[cfg(all(unix, not(any(target_os = "android", target_os = "ios"))))]
const DHCPV4_SERVER_PORT: u16 = 67;

#[cfg(all(unix, not(any(target_os = "android", target_os = "ios"))))]
const DHCPV4_CLIENT_PORT: u16 = 68;

#[cfg(all(unix, not(any(target_os = "android", target_os = "ios"))))]
const DHCPV6_SERVER_PORT: u16 = 547;

#[cfg(all(unix, not(any(target_os = "android", target_os = "ios"))))]
const DHCPV6_CLIENT_PORT: u16 = 546;

#[cfg(all(unix, not(any(target_os = "android", target_os = "ios"))))]
const ROOT_UID: u32 = 0;

/// A enum that describes network security strategy
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum FirewallPolicy {
    /// Allow traffic only to server
    Connecting {
        /// The tunnel peer endpoints that should be allowed.
        peer_endpoints: Vec<AllowedEndpoint>,
        /// Metadata about the tunnels and tunnel interfaces.
        tunnel: Option<TunnelInterface>,
        /// Flag setting if communication with LAN networks should be possible.
        allow_lan: bool,
        /// Servers that are allowed to respond to DNS requests.
        #[cfg(not(target_os = "android"))]
        dns_config: AllowedDns,
        /// Hosts that should be reachable while connecting.
        allowed_endpoints: Vec<AllowedEndpoint>,
        /// Networks for which to permit entry in-tunnel traffic.
        allowed_entry_tunnel_traffic: AllowedTunnelTraffic,
        /// Networks for which to permit exit in-tunnel traffic.
        /// Used when only one tunnel interface is utilized.
        allowed_exit_tunnel_traffic: AllowedTunnelTraffic,
        /// Interface to redirect (VPN tunnel) traffic to
        #[cfg(target_os = "macos")]
        redirect_interface: Option<String>,
    },

    /// Allow traffic only to server and over tunnel interface
    Connected {
        /// The tunnel peer endpoints that should be allowed.
        peer_endpoints: Vec<AllowedEndpoint>,
        /// Metadata about the tunnels and tunnel interfaces.
        tunnel: TunnelInterface,
        /// Flag setting if communication with LAN networks should be possible.
        allow_lan: bool,
        /// Servers that are allowed to respond to DNS requests.
        #[cfg(not(target_os = "android"))]
        dns_config: AllowedDns,
        /// Interface to redirect (VPN tunnel) traffic to
        #[cfg(target_os = "macos")]
        redirect_interface: Option<String>,
    },

    /// Block all network traffic in and out from the computer.
    Blocked {
        /// Flag setting if communication with LAN networks should be possible.
        allow_lan: bool,
        /// Hosts that should be reachable while in the blocked state.
        allowed_endpoints: Vec<AllowedEndpoint>,
    },
}

impl FirewallPolicy {
    /// Return the tunnel peer endpoints
    pub fn peer_endpoints(&self) -> &[AllowedEndpoint] {
        match self {
            FirewallPolicy::Connecting { peer_endpoints, .. }
            | FirewallPolicy::Connected { peer_endpoints, .. } => peer_endpoints.as_ref(),
            _ => &[],
        }
    }

    /// Return the allowed endpoint, if available
    pub fn allowed_endpoints(&self) -> &[AllowedEndpoint] {
        match self {
            FirewallPolicy::Connecting {
                allowed_endpoints, ..
            }
            | FirewallPolicy::Blocked {
                allowed_endpoints, ..
            } => allowed_endpoints,
            _ => &[],
        }
    }

    /// Return tunnel metadata, if available
    pub fn tunnel(&self) -> Option<&TunnelInterface> {
        match self {
            FirewallPolicy::Connecting {
                tunnel: Some(tunnel),
                ..
            }
            | FirewallPolicy::Connected { tunnel, .. } => Some(tunnel),
            _ => None,
        }
    }

    /// Return allowed in-tunnel traffic for entry tunnel
    pub fn allowed_entry_tunnel_traffic(&self) -> &AllowedTunnelTraffic {
        match self {
            FirewallPolicy::Connecting {
                allowed_entry_tunnel_traffic,
                ..
            } => allowed_entry_tunnel_traffic,
            FirewallPolicy::Connected { .. } => &AllowedTunnelTraffic::All,
            _ => &AllowedTunnelTraffic::None,
        }
    }

    /// Return allowed in-tunnel traffic for exit tunnel
    pub fn allowed_exit_tunnel_traffic(&self) -> &AllowedTunnelTraffic {
        match self {
            FirewallPolicy::Connecting {
                allowed_exit_tunnel_traffic,
                ..
            } => allowed_exit_tunnel_traffic,
            FirewallPolicy::Connected { .. } => &AllowedTunnelTraffic::All,
            _ => &AllowedTunnelTraffic::None,
        }
    }

    /// Return whether LAN traffic is allowed
    pub fn allow_lan(&self) -> bool {
        match self {
            FirewallPolicy::Connecting { allow_lan, .. }
            | FirewallPolicy::Connected { allow_lan, .. }
            | FirewallPolicy::Blocked { allow_lan, .. } => *allow_lan,
        }
    }

    /// Return the interface to redirect (VPN tunnel) traffic to, if any.
    #[cfg(target_os = "macos")]
    pub fn redirect_interface(&self) -> Option<&str> {
        match self {
            FirewallPolicy::Connecting {
                redirect_interface, ..
            } => redirect_interface.as_deref(),
            FirewallPolicy::Connected {
                redirect_interface, ..
            } => redirect_interface.as_deref(),
            FirewallPolicy::Blocked { .. } => None,
        }
    }

    #[cfg(not(target_os = "android"))]
    pub fn dns_config(&self) -> Option<&AllowedDns> {
        match self {
            FirewallPolicy::Connecting { dns_config, .. }
            | FirewallPolicy::Connected { dns_config, .. } => Some(dns_config),
            FirewallPolicy::Blocked { .. } => None,
        }
    }
}

impl fmt::Display for FirewallPolicy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FirewallPolicy::Connecting {
                peer_endpoints,
                tunnel,
                allow_lan,
                #[cfg(not(target_os = "android"))]
                dns_config,
                allowed_endpoints,
                allowed_entry_tunnel_traffic,
                allowed_exit_tunnel_traffic,
                #[cfg(target_os = "macos")]
                redirect_interface,
            } => {
                #[cfg(not(target_os = "android"))]
                let dns_str = display_allowed_non_tunnel_dns(dns_config);
                #[cfg(target_os = "android")]
                let dns_str = "none".to_owned();

                if let Some(tunnel) = tunnel {
                    write!(
                        f,
                        "Connecting to {} over {}, allowed entry in-tunnel traffic: {}, allowed exit in-tunnel traffic: {}), {} LAN. Allowing endpoints: {}. Allowing non-tunnel DNS: {}",
                        display_peer_endpoints(peer_endpoints),
                        display_tunnel_interface(tunnel),
                        allowed_entry_tunnel_traffic,
                        allowed_exit_tunnel_traffic,
                        if *allow_lan { "Allowing" } else { "Blocking" },
                        display_allowed_endpoints(allowed_endpoints),
                        dns_str
                    )?;
                } else {
                    write!(
                        f,
                        "Connecting to {}, {} LAN, interface: none. Allowing endpoints: {}. Allowing non-tunnel DNS: {}",
                        display_peer_endpoints(peer_endpoints),
                        if *allow_lan { "Allowing" } else { "Blocking" },
                        display_allowed_endpoints(allowed_endpoints),
                        dns_str
                    )?;
                }

                #[cfg(target_os = "macos")]
                write!(f, ". Redirect interface: {:?}", redirect_interface)?;

                Ok(())
            }
            FirewallPolicy::Connected {
                peer_endpoints,
                tunnel,
                allow_lan,
                #[cfg(not(target_os = "android"))]
                dns_config,
                #[cfg(target_os = "macos")]
                redirect_interface,
            } => {
                #[cfg(not(target_os = "android"))]
                let dns_str = display_allowed_non_tunnel_dns(dns_config);
                #[cfg(target_os = "android")]
                let dns_str = "none".to_owned();

                write!(
                    f,
                    "Connected to {} over {}, {} LAN. Allowing non-tunnel DNS: {}",
                    display_peer_endpoints(peer_endpoints),
                    display_tunnel_interface(tunnel),
                    if *allow_lan { "Allowing" } else { "Blocking" },
                    dns_str
                )?;

                #[cfg(target_os = "macos")]
                write!(f, ". Redirect interface: {:?}", redirect_interface)?;

                Ok(())
            }
            FirewallPolicy::Blocked {
                allow_lan,
                allowed_endpoints,
            } => write!(
                f,
                "Blocked. {} LAN. Allowing endpoints: {}",
                if *allow_lan { "Allowing" } else { "Blocking" },
                display_allowed_endpoints(allowed_endpoints),
            ),
        }
    }
}

#[cfg(not(target_os = "android"))]
fn display_allowed_non_tunnel_dns(dns_config: &AllowedDns) -> String {
    if dns_config.non_tunnel_dns().is_empty() {
        "none".to_owned()
    } else {
        dns_config
            .non_tunnel_dns()
            .iter()
            .map(|ep| ep.to_string())
            .collect::<Vec<_>>()
            .join(",")
    }
}

fn display_tunnel_interface(tunnel: &TunnelInterface) -> String {
    match tunnel {
        TunnelInterface::One(metadata) => display_tunnel_metadata(metadata),
        TunnelInterface::Two { entry, exit } => {
            format!(
                "entry {}, exit {}",
                display_tunnel_metadata(entry),
                display_tunnel_metadata(exit)
            )
        }
    }
}

fn display_tunnel_metadata(metadata: &TunnelMetadata) -> String {
    format!(
        "interface: {}, ips: {}, v4 gw: {:?}, v6 gw: {:?}",
        metadata.interface,
        display_ips(&metadata.ips),
        metadata.ipv4_gateway,
        metadata.ipv6_gateway
    )
}

fn display_peer_endpoints(peer_endpoints: &[AllowedEndpoint]) -> String {
    if peer_endpoints.is_empty() {
        "peers: none".to_owned()
    } else {
        format!(
            "peers: {}",
            peer_endpoints
                .iter()
                .map(|ep| ep.to_string())
                .collect::<Vec<_>>()
                .join(",")
        )
    }
}

fn display_allowed_endpoints(allowed_endpoints: &[AllowedEndpoint]) -> Cow<'_, str> {
    if allowed_endpoints.is_empty() {
        Cow::from("none")
    } else {
        Cow::from(
            allowed_endpoints
                .iter()
                .map(|ep| ep.to_string())
                .collect::<Vec<_>>()
                .join(","),
        )
    }
}

fn display_ips(ips: &[IpAddr]) -> String {
    ips.iter()
        .map(|ip| ip.to_string())
        .collect::<Vec<_>>()
        .join(",")
}

/// Manages network security of the computer/device. Can apply and enforce firewall policies
/// by manipulating the OS firewall and DNS settings.
pub struct Firewall {
    inner: imp::Firewall,
}

/// Arguments required when first initializing the firewall.
pub struct FirewallArguments {
    /// Initial firewall state to enter during init.
    pub initial_state: InitialFirewallState,
    /// This argument is required for the blocked state to configure the firewall correctly.
    pub allow_lan: bool,
    /// Firewall mark is used to mark traffic which should be able to bypass the tunnel
    #[cfg(target_os = "linux")]
    pub fwmark: u32,
    /// The table ID will be used for the routing table that will route all traffic through the
    /// tunnel interface.
    #[cfg(target_os = "linux")]
    pub table_id: u32,
    /// The cgroup2 used for split tunneling.
    /// Traffic from processes in this cgroup2 should be allowed outside the tunnel.
    #[cfg(target_os = "linux")]
    pub excluded_cgroup2: Option<CGroup2>,
    /// The net_cls id of the v1 cgroup used for split tunneling.
    /// This is used as a fallback to [`Self::excluded_cgroup2`] since old kernels don't support cgroups v2.
    #[cfg(target_os = "linux")]
    pub net_cls: Option<u32>,
}

/// State to enter during firewall init.
pub enum InitialFirewallState {
    /// Do not set any policy.
    None,
    /// Atomically enter the blocked state.
    Blocked(Vec<AllowedEndpoint>),
}

impl Firewall {
    /// Creates a firewall instance with the given arguments.
    pub fn from_args(args: FirewallArguments) -> Result<Self, Error> {
        Ok(Firewall {
            inner: imp::Firewall::from_args(args)?,
        })
    }

    /// Applies and starts enforcing the given `FirewallPolicy` Makes sure it is being kept in place
    /// until this method is called again with another policy, or until `reset_policy` is called.
    pub fn apply_policy(&mut self, policy: FirewallPolicy) -> Result<(), Error> {
        tracing::info!("Applying firewall policy: {}", policy);
        self.inner.apply_policy(policy)
    }

    /// Resets/removes any currently enforced `FirewallPolicy`. Returns the system to the same state
    /// it had before any policy was applied through this `Firewall` instance.
    pub fn reset_policy(&mut self) -> Result<(), Error> {
        tracing::info!("Resetting firewall policy");
        self.inner.reset_policy()
    }

    // fixme: exposed as a poor man solution to support nym-split-tunnel
    #[cfg(target_os = "linux")]
    pub fn send_and_process(batch: &nftnl::FinalizedBatch) -> Result<(), Error> {
        imp::Firewall::send_and_process(batch)
    }
}

/// Application that prevents setting the firewall policy.
#[cfg(windows)]
#[derive(Debug, Clone)]
pub struct BlockingApplication {
    pub name: String,
    pub pid: u32,
}

/// Errors that can occur when setting the firewall policy.
#[derive(thiserror::Error, Debug, Clone)]
pub enum FirewallPolicyError {
    /// General firewall failure
    #[error("failed to set firewall policy")]
    Generic,
    /// An application prevented the firewall policy from being set
    #[cfg(windows)]
    #[error("an application prevented the firewall policy from being set")]
    Locked(Option<BlockingApplication>),
}
