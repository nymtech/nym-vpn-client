use std::{collections::HashSet, fmt, net::IpAddr};

use talpid_routing::{NetNode, Node, RequiredRoute, RouteManager};

#[cfg(target_os = "linux")]
const TUNNEL_TABLE_ID: u32 = 0x14d;
#[cfg(target_os = "linux")]
const TUNNEL_FWMARK: u32 = 0x14d;

pub struct MixnetRouteHandler {
    route_manager: RouteManager,
}

impl MixnetRouteHandler {
    pub async fn new() -> Result<Self> {
        let route_manager = RouteManager::new(
            HashSet::new(),
            #[cfg(target_os = "linux")]
            TUNNEL_TABLE_ID,
            #[cfg(target_os = "linux")]
            TUNNEL_FWMARK,
        )
        .await?;
        Ok(Self { route_manager })
    }

    pub async fn add_routes(
        &mut self,
        tun_name: String,
        #[cfg(not(target_os = "linux"))] entry_gateway_address: IpAddr,
    ) -> Result<()> {
        let mut routes = HashSet::new();

        #[cfg(not(target_os = "linux"))]
        routes.insert(RequiredRoute::new(
            ipnetwork::IpNetwork::from(entry_gateway_address),
            NetNode::DefaultNode,
        ));

        routes.insert(RequiredRoute::new(
            "0.0.0.0/0".parse().unwrap(),
            Node::device(tun_name.clone()),
        ));

        routes.insert(RequiredRoute::new(
            "::0/0".parse().unwrap(),
            Node::device(tun_name),
        ));

        #[cfg(target_os = "linux")]
        {
            routes = routes
                .into_iter()
                .map(|r| r.use_main_table(false))
                .collect();
        }

        #[cfg(target_os = "linux")]
        self.route_manager.create_routing_rules(true).await?;

        self.route_manager.add_routes(routes).await?;

        Ok(())
    }

    pub async fn remove_routes(&mut self) {
        if let Err(e) = self.route_manager.clear_routes() {
            tracing::error!("Failed to remove rules: {}", e);
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
    }
}

#[derive(Debug)]
pub struct Error {
    inner: talpid_routing::Error,
}

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
        write!(f, "Failed to setup routing")
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
