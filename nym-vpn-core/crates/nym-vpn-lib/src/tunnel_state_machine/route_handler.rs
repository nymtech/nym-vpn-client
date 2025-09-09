// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{collections::HashSet, fmt, net::IpAddr};

use ipnetwork::IpNetwork;

use nym_common::trace_err_chain;
#[cfg(not(target_os = "linux"))]
use nym_routing::NetNode;
#[cfg(windows)]
pub use nym_routing::{Callback, CallbackHandle};
use nym_routing::{Node, RequiredRoute, RouteManagerHandle};

pub enum RoutingConfig {
    Mixnet {
        tun_name: String,
        tun_mtu: u16,
        #[cfg(not(target_os = "linux"))]
        entry_gateway_address: IpAddr,
    },
    Wireguard {
        /// Entry tunnel name
        entry_tun_name: String,

        /// Exit tunnel name
        exit_tun_name: String,

        /// Entry tunnel MTU
        entry_tun_mtu: u16,

        /// Exit tunnel MTU
        exit_tun_mtu: u16,

        /// Private (in-tunnel) gateway IP
        private_entry_gateway_address: IpAddr,

        /// Public entry gateway IP
        #[cfg(not(target_os = "linux"))]
        entry_gateway_address: IpAddr,

        /// Public exit gateway IP
        exit_gateway_address: IpAddr,
    },
    WireguardNetstack {
        exit_tun_name: String,
        exit_tun_mtu: u16,
        #[cfg(not(target_os = "linux"))]
        entry_gateway_address: IpAddr,
    },
}

#[cfg(target_os = "linux")]
#[derive(Debug, Clone)]
pub struct RoutingParameters {
    /// Routing table id used for routing all traffic through the tunnel.
    pub table_id: u32,

    /// Firewall mark used for marking traffic that should bypass the tunnel.
    pub fwmark: u32,
}

#[cfg(target_os = "linux")]
impl Default for RoutingParameters {
    fn default() -> Self {
        Self {
            table_id: crate::TUNNEL_TABLE_ID,
            fwmark: crate::TUNNEL_FWMARK,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RouteHandler {
    route_manager: RouteManagerHandle,
}

impl RouteHandler {
    pub async fn new(
        #[cfg(target_os = "linux")] routing_parameters: RoutingParameters,
    ) -> Result<Self> {
        let route_manager = RouteManagerHandle::spawn(
            #[cfg(target_os = "linux")]
            routing_parameters.table_id,
            #[cfg(target_os = "linux")]
            routing_parameters.fwmark,
        )
        .await?;
        Ok(Self { route_manager })
    }

    pub async fn add_routes(
        &mut self,
        routing_config: RoutingConfig,
        enable_ipv6: bool,
    ) -> Result<()> {
        let routes = Self::get_routes(routing_config, enable_ipv6);

        #[cfg(target_os = "linux")]
        self.route_manager.create_routing_rules(enable_ipv6).await?;

        self.route_manager.add_routes(routes).await?;

        Ok(())
    }

    pub async fn remove_routes(&mut self) {
        if let Err(e) = self.route_manager.clear_routes() {
            trace_err_chain!(e, "Failed to remove routes");
        }

        #[cfg(target_os = "linux")]
        if let Err(e) = self.route_manager.clear_routing_rules().await {
            trace_err_chain!(e, "Failed to remove routing rules");
        }
    }

    #[cfg(target_os = "macos")]
    pub async fn refresh_routes(&mut self) {
        if let Err(e) = self.route_manager.refresh_routes() {
            trace_err_chain!(e, "Failed to refresh routes");
        }
    }

    #[cfg(any(target_os = "linux", target_os = "windows"))]
    pub async fn get_mtu_for_route(&mut self, ip_addr: IpAddr) -> Result<u16> {
        Ok(self.route_manager.get_mtu_for_route(ip_addr).await?)
    }

    #[cfg(windows)]
    pub async fn add_default_route_listener(
        &mut self,
        event_handler: Callback,
    ) -> Result<CallbackHandle> {
        Ok(self
            .route_manager
            .add_default_route_change_callback(event_handler)
            .await?)
    }

    pub async fn stop(self) {
        self.route_manager.stop().await;
    }

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    pub fn inner_handle(&self) -> nym_routing::RouteManagerHandle {
        self.route_manager.clone()
    }

    fn get_routes(routing_config: RoutingConfig, enable_ipv6: bool) -> HashSet<RequiredRoute> {
        let mut routes = HashSet::new();

        match routing_config {
            RoutingConfig::Mixnet {
                tun_name,
                tun_mtu,
                #[cfg(not(target_os = "linux"))]
                entry_gateway_address,
            } => {
                #[cfg(not(target_os = "linux"))]
                routes.insert(RequiredRoute::new(
                    IpNetwork::from(entry_gateway_address),
                    NetNode::DefaultNode,
                ));
                routes.extend(Self::get_default_routes(tun_name, tun_mtu, enable_ipv6));
            }
            RoutingConfig::Wireguard {
                entry_tun_name,
                exit_tun_name,
                entry_tun_mtu,
                exit_tun_mtu,
                private_entry_gateway_address: in_tunnel_entry_gateway_address,
                #[cfg(not(target_os = "linux"))]
                entry_gateway_address,
                exit_gateway_address,
            } => {
                #[cfg(not(target_os = "linux"))]
                routes.insert(RequiredRoute::new(
                    IpNetwork::from(entry_gateway_address),
                    NetNode::DefaultNode,
                ));

                routes.insert(Self::get_in_tunnel_gateway_entry_route(
                    in_tunnel_entry_gateway_address,
                    entry_tun_name.clone(),
                    entry_tun_mtu,
                ));
                routes.insert(Self::get_multihop_exit_route(
                    exit_gateway_address,
                    entry_tun_name,
                    entry_tun_mtu,
                ));
                routes.extend(Self::get_default_routes(
                    exit_tun_name,
                    exit_tun_mtu,
                    enable_ipv6,
                ));
            }
            RoutingConfig::WireguardNetstack {
                exit_tun_name,
                exit_tun_mtu,
                #[cfg(not(target_os = "linux"))]
                entry_gateway_address,
            } => {
                #[cfg(not(target_os = "linux"))]
                routes.insert(RequiredRoute::new(
                    IpNetwork::from(entry_gateway_address),
                    NetNode::DefaultNode,
                ));
                routes.extend(Self::get_default_routes(
                    exit_tun_name,
                    exit_tun_mtu,
                    enable_ipv6,
                ));
            }
        }

        routes
    }

    fn get_in_tunnel_gateway_entry_route(
        in_tunnel_gateway_address: IpAddr,
        iface: String,
        _mtu: u16,
    ) -> RequiredRoute {
        #[allow(unused_mut)]
        let mut route = RequiredRoute::new(
            IpNetwork::from(in_tunnel_gateway_address),
            Node::device(iface),
        );

        #[cfg(target_os = "linux")]
        {
            route = route.use_main_table(false);
        }

        #[cfg(any(target_os = "linux", target_os = "macos"))]
        {
            route = route.mtu(_mtu);
        }

        route
    }

    fn get_multihop_exit_route(ip_addr: IpAddr, iface: String, _mtu: u16) -> RequiredRoute {
        #[allow(unused_mut)]
        let mut route = RequiredRoute::new(IpNetwork::from(ip_addr), Node::device(iface));

        #[cfg(target_os = "linux")]
        {
            route = route.use_main_table(false);
        }

        #[cfg(any(target_os = "linux", target_os = "macos"))]
        {
            route = route.mtu(_mtu);
        }

        route
    }

    fn get_default_routes(iface: String, _mtu: u16, enable_ipv6: bool) -> Vec<RequiredRoute> {
        let mut routes = Vec::new();

        routes.push(RequiredRoute::new(
            "0.0.0.0/0".parse().unwrap(),
            Node::device(iface.to_owned()),
        ));

        if enable_ipv6 {
            routes.push(RequiredRoute::new(
                "::0/0".parse().unwrap(),
                Node::device(iface),
            ));
        }

        #[cfg(target_os = "linux")]
        {
            routes = routes
                .into_iter()
                .map(|r| r.use_main_table(false))
                .collect();
        }

        #[cfg(any(target_os = "linux", target_os = "macos"))]
        {
            routes = routes.into_iter().map(|r| r.mtu(_mtu)).collect();
        }

        routes
    }
}

#[derive(Debug)]
pub struct Error {
    inner: nym_routing::Error,
}

unsafe impl Send for Error {}
unsafe impl Sync for Error {}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.inner)
    }
}

impl From<nym_routing::Error> for Error {
    fn from(value: nym_routing::Error) -> Self {
        Self { inner: value }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "routing error: {}", self.inner)
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
