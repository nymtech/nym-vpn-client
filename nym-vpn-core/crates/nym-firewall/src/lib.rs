use std::{
    borrow::Cow,
    fmt,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    sync::LazyLock,
};

use ipnetwork::{IpNetwork, Ipv4Network, Ipv6Network};
#[cfg(not(target_os = "android"))]
use nym_dns::ResolvedDnsConfig;

#[cfg(target_os = "macos")]
#[path = "macos.rs"]
mod imp;
#[cfg(target_os = "macos")]
pub use imp::LOCAL_DNS_RESOLVER;

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
mod split_tunnel;
use net::ALLOWED_LAN_NETS;
pub use net::{
    AllowedClients, AllowedEndpoint, AllowedTunnelTraffic, Endpoint, TransportProtocol,
    TunnelInterface, TunnelMetadata,
};

pub use self::imp::Error;

#[cfg(any(target_os = "linux", target_os = "macos"))]
static IPV6_LINK_LOCAL: LazyLock<Ipv6Network> =
    LazyLock::new(|| Ipv6Network::new(Ipv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, 0), 10).unwrap());
/// The allowed target addresses of outbound DHCPv6 requests
#[cfg(any(target_os = "linux", target_os = "macos"))]
static DHCPV6_SERVER_ADDRS: LazyLock<[Ipv6Addr; 2]> = LazyLock::new(|| {
    [
        Ipv6Addr::new(0xff02, 0, 0, 0, 0, 0, 1, 2),
        Ipv6Addr::new(0xff05, 0, 0, 0, 0, 0, 1, 3),
    ]
});
#[cfg(any(target_os = "linux", target_os = "macos"))]
static ROUTER_SOLICITATION_OUT_DST_ADDR: LazyLock<Ipv6Addr> =
    LazyLock::new(|| Ipv6Addr::new(0xff02, 0, 0, 0, 0, 0, 0, 2));
#[cfg(any(target_os = "linux", target_os = "macos"))]
static SOLICITED_NODE_MULTICAST: LazyLock<Ipv6Network> = LazyLock::new(|| {
    Ipv6Network::new(Ipv6Addr::new(0xff02, 0, 0, 0, 0, 1, 0xFF00, 0), 104).unwrap()
});
static LOOPBACK_NETS: LazyLock<[IpNetwork; 2]> = LazyLock::new(|| {
    [
        IpNetwork::V4(Ipv4Network::new(Ipv4Addr::new(127, 0, 0, 0), 8).unwrap()),
        IpNetwork::V6(Ipv6Network::new(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1), 128).unwrap()),
    ]
});

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

/// Allowed TCP ports to DNS servers when connecting.
#[cfg(any(target_os = "linux", target_os = "macos"))]
const DNS_TCP_PORTS: [u16; 3] = [53, 443, 853];

/// Allowed UDP ports to DNS servers when connecting.
#[cfg(any(target_os = "linux", target_os = "macos"))]
const DNS_UDP_PORTS: [u16; 1] = [53];

/// Returns whether an address belongs to a private subnet.
pub fn is_local_address(address: &IpAddr) -> bool {
    let address = *address;
    (*ALLOWED_LAN_NETS)
        .iter()
        .chain(&*LOOPBACK_NETS)
        .any(|net| net.contains(address))
}

/// A enum that describes network security strategy
///
/// # Firewall block/allow specification.
///
/// See the [security](../../../docs/security.md) document for the specification on how to
/// implement these policies and what should and should not be allowed to flow.
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
        dns_config: ResolvedDnsConfig,
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
        /// Destination port for DNS traffic redirection. Traffic destined to `127.0.0.1:53` will
        /// be redirected to `127.0.0.1:$dns_redirect_port`.
        #[cfg(target_os = "macos")]
        dns_redirect_port: u16,
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
        dns_config: ResolvedDnsConfig,
        /// Hosts that should be reachable outside of tunnel when connected.
        allowed_endpoints: Vec<AllowedEndpoint>,
        /// Interface to redirect (VPN tunnel) traffic to
        #[cfg(target_os = "macos")]
        redirect_interface: Option<String>,
        /// Destination port for DNS traffic redirection. Traffic destined to `127.0.0.1:53` will
        /// be redirected to `127.0.0.1:$dns_redirect_port`.
        #[cfg(target_os = "macos")]
        dns_redirect_port: u16,
    },

    /// Block all network traffic in and out from the computer.
    Blocked {
        /// Flag setting if communication with LAN networks should be possible.
        allow_lan: bool,
        /// Hosts that should be reachable while in the blocked state.
        allowed_endpoints: Vec<AllowedEndpoint>,
        /// Destination port for DNS traffic redirection. Traffic destined to `127.0.0.1:53` will
        /// be redirected to `127.0.0.1:$dns_redirect_port`.
        #[cfg(target_os = "macos")]
        dns_redirect_port: u16,
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
                ..
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
                    )
                } else {
                    write!(
                        f,
                        "Connecting to {}, {} LAN, interface: none. Allowing endpoints: {}. Allowing non-tunnel DNS: {}",
                        display_peer_endpoints(peer_endpoints),
                        if *allow_lan { "Allowing" } else { "Blocking" },
                        display_allowed_endpoints(allowed_endpoints),
                        dns_str
                    )
                }
            }
            FirewallPolicy::Connected {
                peer_endpoints,
                tunnel,
                allow_lan,
                #[cfg(not(target_os = "android"))]
                dns_config,
                allowed_endpoints,
                ..
            } => {
                #[cfg(not(target_os = "android"))]
                let dns_str = display_allowed_non_tunnel_dns(dns_config);
                #[cfg(target_os = "android")]
                let dns_str = "none".to_owned();

                write!(
                f,
                "Connected to {} over {}, {} LAN. Allowing endpoints: {}. Allowing non-tunnel DNS: {}",
                display_peer_endpoints(peer_endpoints),
                display_tunnel_interface(tunnel),
                if *allow_lan { "Allowing" } else { "Blocking" },
                display_allowed_endpoints(allowed_endpoints),
                dns_str
            )
            }
            FirewallPolicy::Blocked {
                allow_lan,
                allowed_endpoints,
                ..
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
fn display_allowed_non_tunnel_dns(dns_config: &ResolvedDnsConfig) -> String {
    if dns_config.non_tunnel_config().is_empty() {
        "none".to_owned()
    } else {
        dns_config
            .non_tunnel_config()
            .iter()
            .map(|ip| ip.to_string())
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
    /// Specifies the firewall mark used to identify traffic that is allowed to be excluded from
    /// the tunnel and _leaked_ during blocked states.
    #[cfg(target_os = "linux")]
    pub fwmark: u32,
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

    /// Creates a new firewall instance.
    pub fn new(#[cfg(target_os = "linux")] fwmark: u32) -> Result<Self, Error> {
        Ok(Firewall {
            inner: imp::Firewall::new(
                #[cfg(target_os = "linux")]
                fwmark,
            )?,
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
    #[error("Failed to set firewall policy")]
    Generic,
    /// An application prevented the firewall policy from being set
    #[cfg(windows)]
    #[error("An application prevented the firewall policy from being set")]
    Locked(Option<BlockingApplication>),
}
