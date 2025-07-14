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

#[cfg(target_os = "linux")]
pub const TUNNEL_TABLE_ID: u32 = 0x14d;
#[cfg(target_os = "linux")]
pub const TUNNEL_FWMARK: u32 = 0x14d;

pub enum RoutingConfig {
    Mixnet {
        tun_name: String,
        #[cfg(not(target_os = "linux"))]
        entry_gateway_address: IpAddr,
    },
    Wireguard {
        entry_tun_name: String,
        exit_tun_name: String,
        #[cfg(not(target_os = "linux"))]
        entry_gateway_address: IpAddr,
        exit_gateway_address: IpAddr,
    },
    WireguardNetstack {
        exit_tun_name: String,
        #[cfg(not(target_os = "linux"))]
        entry_gateway_address: IpAddr,
    },
}

#[derive(Debug, Clone)]
pub struct RouteHandler {
    route_manager: RouteManagerHandle,
}

impl RouteHandler {
    pub async fn new() -> Result<Self> {
        let route_manager = RouteManagerHandle::spawn(
            #[cfg(target_os = "linux")]
            TUNNEL_TABLE_ID,
            #[cfg(target_os = "linux")]
            TUNNEL_FWMARK,
        )
        .await?;
        Ok(Self { route_manager })
    }

    pub async fn add_routes(&mut self, routing_config: RoutingConfig) -> Result<()> {
        let routes = Self::get_routes(routing_config);

        #[cfg(target_os = "linux")]
        self.route_manager.create_routing_rules().await?;

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

    #[cfg(windows)]
    pub async fn add_default_route_listener(
        &mut self,
        event_handler: Callback,
    ) -> Result<CallbackHandle> {
        self.route_manager
            .add_default_route_change_callback(event_handler)
            .await
            .map_err(Error::from)
    }

    pub async fn stop(self) {
        self.route_manager.stop().await;
    }

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    pub fn inner_handle(&self) -> nym_routing::RouteManagerHandle {
        self.route_manager.clone()
    }

    fn get_routes(routing_config: RoutingConfig) -> HashSet<RequiredRoute> {
        let mut routes = HashSet::new();

        match routing_config {
            RoutingConfig::Mixnet {
                tun_name,
                #[cfg(not(target_os = "linux"))]
                entry_gateway_address,
            } => {
                #[cfg(not(target_os = "linux"))]
                routes.insert(RequiredRoute::new(
                    IpNetwork::from(entry_gateway_address),
                    NetNode::DefaultNode,
                ));
                routes.extend(Self::get_wildcard_routes(tun_name));
            }
            RoutingConfig::Wireguard {
                entry_tun_name,
                exit_tun_name,
                #[cfg(not(target_os = "linux"))]
                entry_gateway_address,
                exit_gateway_address,
            } => {
                #[cfg(not(target_os = "linux"))]
                routes.insert(RequiredRoute::new(
                    IpNetwork::from(entry_gateway_address),
                    NetNode::DefaultNode,
                ));
                routes.insert(Self::get_exit_route(exit_gateway_address, entry_tun_name));
                routes.extend(Self::get_wildcard_routes(exit_tun_name));
            }
            RoutingConfig::WireguardNetstack {
                exit_tun_name,
                #[cfg(not(target_os = "linux"))]
                entry_gateway_address,
            } => {
                #[cfg(not(target_os = "linux"))]
                routes.insert(RequiredRoute::new(
                    IpNetwork::from(entry_gateway_address),
                    NetNode::DefaultNode,
                ));
                routes.extend(Self::get_wildcard_routes(exit_tun_name));
            }
        }

        routes
    }

    fn get_exit_route(ip_addr: IpAddr, iface: String) -> RequiredRoute {
        let route = RequiredRoute::new(IpNetwork::from(ip_addr), Node::device(iface));

        #[cfg(target_os = "linux")]
        {
            route.use_main_table(false)
        }

        #[cfg(not(target_os = "linux"))]
        {
            route
        }
    }

    fn get_wildcard_routes(iface: String) -> Vec<RequiredRoute> {
        let ipv4_route =
            RequiredRoute::new("0.0.0.0/0".parse().unwrap(), Node::device(iface.to_owned()));
        let ipv6_route = RequiredRoute::new("::0/0".parse().unwrap(), Node::device(iface));

        let routes = vec![ipv4_route, ipv6_route];

        #[cfg(target_os = "linux")]
        {
            routes
                .into_iter()
                .map(|r| r.use_main_table(false))
                .collect()
        }

        #[cfg(not(target_os = "linux"))]
        {
            routes
        }
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
