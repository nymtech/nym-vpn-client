// Copyright 2016-2025 Mullvad VPN AB. All Rights Reserved.
// Copyright 2025 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(windows)]
use std::path::PathBuf;
use std::{
    fmt,
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    str::FromStr,
    sync::LazyLock,
};

use ipnetwork::{IpNetwork, Ipv4Network, Ipv6Network};

/// When "allow local network" is enabled the app will allow traffic to and from these networks.
pub static ALLOWED_LAN_NETS: LazyLock<[IpNetwork; 6]> = LazyLock::new(|| {
    [
        IpNetwork::V4(Ipv4Network::new(Ipv4Addr::new(10, 0, 0, 0), 8).unwrap()),
        IpNetwork::V4(Ipv4Network::new(Ipv4Addr::new(172, 16, 0, 0), 12).unwrap()),
        IpNetwork::V4(Ipv4Network::new(Ipv4Addr::new(192, 168, 0, 0), 16).unwrap()),
        IpNetwork::V4(Ipv4Network::new(Ipv4Addr::new(169, 254, 0, 0), 16).unwrap()),
        IpNetwork::V6(Ipv6Network::new(Ipv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, 0), 10).unwrap()),
        IpNetwork::V6(Ipv6Network::new(Ipv6Addr::new(0xfc00, 0, 0, 0, 0, 0, 0, 0), 7).unwrap()),
    ]
});
/// When "allow local network" is enabled the app will allow traffic to these networks.
#[cfg_attr(target_os = "windows", allow(unused))]
pub static ALLOWED_LAN_MULTICAST_NETS: LazyLock<[IpNetwork; 8]> = LazyLock::new(|| {
    [
        // Local network broadcast. Not routable
        IpNetwork::V4(Ipv4Network::new(Ipv4Addr::new(255, 255, 255, 255), 32).unwrap()),
        // Local subnetwork multicast. Not routable
        IpNetwork::V4(Ipv4Network::new(Ipv4Addr::new(224, 0, 0, 0), 24).unwrap()),
        // Admin-local IPv4 multicast.
        IpNetwork::V4(Ipv4Network::new(Ipv4Addr::new(239, 0, 0, 0), 8).unwrap()),
        // Interface-local IPv6 multicast.
        IpNetwork::V6(Ipv6Network::new(Ipv6Addr::new(0xff01, 0, 0, 0, 0, 0, 0, 0), 16).unwrap()),
        // Link-local IPv6 multicast. IPv6 equivalent of 224.0.0.0/24
        IpNetwork::V6(Ipv6Network::new(Ipv6Addr::new(0xff02, 0, 0, 0, 0, 0, 0, 0), 16).unwrap()),
        // Realm-local IPv6 multicast.
        IpNetwork::V6(Ipv6Network::new(Ipv6Addr::new(0xff03, 0, 0, 0, 0, 0, 0, 0), 16).unwrap()),
        // Admin-local IPv6 multicast.
        IpNetwork::V6(Ipv6Network::new(Ipv6Addr::new(0xff04, 0, 0, 0, 0, 0, 0, 0), 16).unwrap()),
        // Site-local IPv6 multicast.
        IpNetwork::V6(Ipv6Network::new(Ipv6Addr::new(0xff05, 0, 0, 0, 0, 0, 0, 0), 16).unwrap()),
    ]
});

/// What [`Endpoint`]s to allow the client to send traffic to and receive from.
///
/// In some cases we want to restrict what IP addresses the client may communicate with even
/// inside of the tunnel, for example while negotiating a PQ-safe PSK with an ephemeral peer.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum AllowedTunnelTraffic {
    /// Block all traffic inside the tunnel.
    None,
    /// Allow all traffic inside the tunnel. This is the normal mode of operation.
    All,
    /// Only allow communication with this specific endpoint. This will usually be a relay during a
    /// short amount of time.
    One(Endpoint),
    /// Only allow communication with these two specific endpoints. The intended use case for this
    /// is while negotiating for example a PSK with both the entry & exit relays in a multihop setup.
    Two(Endpoint, Endpoint),
}

impl AllowedTunnelTraffic {
    /// Do we currently allow traffic to all endpoints?
    pub fn all(&self) -> bool {
        matches!(self, AllowedTunnelTraffic::All)
    }
}

impl fmt::Display for AllowedTunnelTraffic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            AllowedTunnelTraffic::None => "None".fmt(f),
            AllowedTunnelTraffic::All => "All".fmt(f),
            AllowedTunnelTraffic::One(endpoint) => endpoint.fmt(f),
            AllowedTunnelTraffic::Two(endpoint1, endpoint2) => {
                endpoint1.fmt(f)?;
                f.write_str(", ")?;
                endpoint2.fmt(f)
            }
        }
    }
}

/// Represents a network layer IP address together with the transport layer protocol and port.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Endpoint {
    /// The socket address for the endpoint
    pub address: SocketAddr,
    /// The protocol part of this endpoint.
    pub protocol: TransportProtocol,
}

impl Endpoint {
    /// Constructs a new `Endpoint` from the given parameters.
    pub fn new(address: impl Into<IpAddr>, port: u16, protocol: TransportProtocol) -> Self {
        Endpoint {
            address: SocketAddr::new(address.into(), port),
            protocol,
        }
    }

    pub const fn from_socket_address(address: SocketAddr, protocol: TransportProtocol) -> Self {
        Endpoint { address, protocol }
    }
}

impl fmt::Display for Endpoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(f, "{}/{}", self.address, self.protocol)
    }
}

/// Host that should be reachable in any tunnel state.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct AllowedEndpoint {
    /// How to connect to a certain `endpoint`.
    pub endpoint: Endpoint,
    /// Clients that should be allowed to communicate with `endpoint`.
    pub clients: AllowedClients,
}

impl AllowedEndpoint {
    pub fn new(endpoint: Endpoint, clients: AllowedClients) -> Self {
        Self { endpoint, clients }
    }
}

impl fmt::Display for AllowedEndpoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        #[cfg(not(windows))]
        write!(f, "{}", self.endpoint)?;
        #[cfg(windows)]
        {
            let clients = if self.clients.allow_all() {
                "any executable".to_string()
            } else {
                self.clients
                    .iter()
                    .map(|client| {
                        client
                            .file_name()
                            .map(|s| s.to_string_lossy())
                            .unwrap_or(std::borrow::Cow::Borrowed("<UNKNOWN>"))
                    })
                    .collect::<Vec<_>>()
                    .join(" ")
            };
            write!(
                f,
                "{endpoint} for {clients}",
                endpoint = self.endpoint,
                clients = clients
            )?;
        }
        Ok(())
    }
}

/// Clients which should be able to reach an allowed host in any tunnel state.
#[cfg(unix)]
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum AllowedClients {
    /// Allow only clients running as `root` to leak traffic to an allowed [`Endpoint`].
    ///
    /// # Note
    /// The most secure client(s) is our own, which runs as root.
    Root,
    /// Allow *all* clients to leak traffic to an allowed [`Endpoint`].
    ///
    /// This is necessary on platforms which does not have proper support for
    /// split-tunneling, but which wants to support running local proxies which
    /// may not run as root.
    All,
}

#[cfg(unix)]
impl AllowedClients {
    pub fn allow_all(&self) -> bool {
        matches!(self, AllowedClients::All)
    }
}

/// Clients which should be able to reach an allowed host in any tunnel state.
///
/// # Note
/// On Windows, there is no predetermined binary which should be allowed to leak
/// traffic outside of the tunnel. Thus, [`std::default::Default`] is not
/// implemented for [`AllowedClients`].
#[cfg(windows)]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct AllowedClients(std::sync::Arc<[PathBuf]>);

#[cfg(windows)]
impl std::ops::Deref for AllowedClients {
    type Target = [PathBuf];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(windows)]
impl From<Vec<PathBuf>> for AllowedClients {
    fn from(value: Vec<PathBuf>) -> Self {
        Self(value.into())
    }
}

#[cfg(windows)]
impl AllowedClients {
    /// Allow all clients to leak traffic to an allowed [`Endpoint`].
    pub fn all() -> Self {
        vec![].into()
    }

    /// Allow current executable to leak traffic to an allowed [`Endpoint`]
    pub fn current_exe() -> Self {
        let current_exe_path = std::env::current_exe().expect("failed to obtain current_exe");
        Self::from(vec![current_exe_path])
    }

    pub fn allow_all(&self) -> bool {
        self.is_empty()
    }
}

/// Representation of a transport protocol, either UDP or TCP.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum TransportProtocol {
    /// Represents the UDP transport protocol.
    Udp,
    /// Represents the TCP transport protocol.
    Tcp,
}

impl FromStr for TransportProtocol {
    type Err = TransportProtocolParseError;

    fn from_str(s: &str) -> std::result::Result<TransportProtocol, Self::Err> {
        if s.eq_ignore_ascii_case("udp") {
            return Ok(TransportProtocol::Udp);
        }
        if s.eq_ignore_ascii_case("tcp") {
            return Ok(TransportProtocol::Tcp);
        }
        Err(TransportProtocolParseError)
    }
}

impl fmt::Display for TransportProtocol {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            TransportProtocol::Udp => "UDP".fmt(fmt),
            TransportProtocol::Tcp => "TCP".fmt(fmt),
        }
    }
}

/// Returned when `TransportProtocol::from_str` fails to convert a string into a
/// [`TransportProtocol`] object.
#[derive(thiserror::Error, Debug, Clone, PartialEq, Eq)]
#[error("Not a valid transport protocol")]
pub struct TransportProtocolParseError;

/// Information about a VPN tunnel.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct TunnelMetadata {
    /// The name of the device which the tunnel is running on.
    pub interface: String,
    /// The local IPs on the tunnel interface.
    pub ips: Vec<IpAddr>,
    /// The IP to the default gateway on the tunnel interface.
    pub ipv4_gateway: Option<Ipv4Addr>,
    /// The IP to the IPv6 default gateway on the tunnel interface.
    pub ipv6_gateway: Option<Ipv6Addr>,
}

/// Describes the interface(s) that a tunnel is running on.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum TunnelInterface {
    // Single tunnel interface
    One(TunnelMetadata),
    // Two tunnel interfaces
    Two {
        entry: TunnelMetadata,
        exit: TunnelMetadata,
    },
}

impl TunnelInterface {
    pub fn exit_metadata(&self) -> &TunnelMetadata {
        match self {
            TunnelInterface::One(tunnel_metadata) => tunnel_metadata,
            TunnelInterface::Two { entry: _, exit } => exit,
        }
    }

    pub fn inner_metadatas(&self) -> Vec<&TunnelMetadata> {
        match self {
            TunnelInterface::One(tunnel_metadata) => vec![tunnel_metadata],
            TunnelInterface::Two { entry, exit } => vec![entry, exit],
        }
    }
}
