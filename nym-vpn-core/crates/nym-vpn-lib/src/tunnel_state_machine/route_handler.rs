use std::{collections::HashSet, fmt, net::IpAddr};

use ipnetwork::IpNetwork;
#[cfg(not(target_os = "linux"))]
use talpid_routing::NetNode;
use talpid_routing::{Node, RequiredRoute, RouteManager};

#[cfg(target_os = "linux")]
pub const TUNNEL_TABLE_ID: u32 = 0x14d;
#[cfg(target_os = "linux")]
pub const TUNNEL_FWMARK: u32 = 0x14d;

pub enum RoutingConfig {
    Mixnet {
        enable_ipv6: bool,
        tun_name: String,
        #[cfg(not(target_os = "linux"))]
        entry_gateway_address: IpAddr,
    },
    Wireguard {
        enable_ipv6: bool,
        entry_tun_name: String,
        exit_tun_name: String,
        #[cfg(not(target_os = "linux"))]
        entry_gateway_address: IpAddr,
        exit_gateway_address: IpAddr,
    },
}

impl RoutingConfig {
    #[cfg(target_os = "linux")]
    pub fn enable_ipv6(&self) -> bool {
        match self {
            Self::Mixnet { enable_ipv6, .. } => *enable_ipv6,
            Self::Wireguard { enable_ipv6, .. } => *enable_ipv6,
        }
    }
}

pub struct RouteHandler {
    route_manager: RouteManager,
}

impl RouteHandler {
    pub async fn new() -> Result<Self> {
        let route_manager = RouteManager::new(
            #[cfg(target_os = "linux")]
            TUNNEL_TABLE_ID,
            #[cfg(target_os = "linux")]
            TUNNEL_FWMARK,
        )
        .await?;
        Ok(Self { route_manager })
    }

    pub async fn add_routes(&mut self, routing_config: RoutingConfig) -> Result<()> {
        #[cfg(target_os = "linux")]
        let enable_ipv6 = routing_config.enable_ipv6();
        let routes = Self::get_routes(routing_config);

        #[cfg(target_os = "linux")]
        self.route_manager.create_routing_rules(enable_ipv6).await?;

        self.route_manager.add_routes(routes).await?;

        Ok(())
    }

    pub async fn remove_routes(&mut self) {
        if let Err(e) = self.route_manager.clear_routes() {
            tracing::error!("Failed to remove routes: {}", e);
        }

        #[cfg(target_os = "linux")]
        if let Err(e) = self.route_manager.clear_routing_rules().await {
            tracing::error!("Failed to remove routing rules: {}", e);
        }
    }

    pub async fn stop(mut self) {
        #[cfg(windows)]
        self.route_manager.stop();

        #[cfg(not(windows))]
        self.route_manager.stop().await;

        _ = tokio::task::spawn_blocking(|| drop(self.route_manager)).await;
    }

    #[cfg(target_os = "linux")]
    pub(super) fn inner_handle(&self) -> Result<talpid_routing::RouteManagerHandle> {
        Ok(self.route_manager.handle()?)
    }

    fn get_routes(routing_config: RoutingConfig) -> HashSet<RequiredRoute> {
        let mut routes = HashSet::new();

        match routing_config {
            RoutingConfig::Mixnet {
                enable_ipv6,
                tun_name,
                #[cfg(not(target_os = "linux"))]
                entry_gateway_address,
            } => {
                #[cfg(not(target_os = "linux"))]
                routes.insert(RequiredRoute::new(
                    IpNetwork::from(entry_gateway_address),
                    NetNode::DefaultNode,
                ));

                routes.insert(RequiredRoute::new(
                    "0.0.0.0/0".parse().unwrap(),
                    Node::device(tun_name.to_owned()),
                ));

                if enable_ipv6 {
                    routes.insert(RequiredRoute::new(
                        "::0/0".parse().unwrap(),
                        Node::device(tun_name.to_owned()),
                    ));
                }
            }
            RoutingConfig::Wireguard {
                enable_ipv6,
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

                routes.insert(RequiredRoute::new(
                    IpNetwork::from(exit_gateway_address),
                    Node::device(entry_tun_name.to_owned()),
                ));

                routes.insert(RequiredRoute::new(
                    "0.0.0.0/0".parse().unwrap(),
                    Node::device(exit_tun_name.to_owned()),
                ));

                if enable_ipv6 {
                    routes.insert(RequiredRoute::new(
                        "::0/0".parse().unwrap(),
                        Node::device(exit_tun_name.to_owned()),
                    ));
                }
            }
        }

        #[cfg(target_os = "linux")]
        {
            routes = routes
                .into_iter()
                .map(|r| r.use_main_table(false))
                .collect();
        }

        routes
    }
}

#[derive(Debug)]
pub struct Error {
    inner: talpid_routing::Error,
}

unsafe impl Send for Error {}
unsafe impl Sync for Error {}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.inner)
    }
}

impl From<talpid_routing::Error> for Error {
    fn from(value: talpid_routing::Error) -> Self {
        Self { inner: value }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "routing error: {}", self.inner)
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
