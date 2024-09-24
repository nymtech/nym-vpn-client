use std::{collections::HashSet, fmt, net::IpAddr};

use talpid_routing::{NetNode, Node, RequiredRoute, RouteManager};

#[cfg(target_os = "linux")]
pub const TUNNEL_TABLE_ID: u32 = 0x14d;
#[cfg(target_os = "linux")]
pub const TUNNEL_FWMARK: u32 = 0x14d;

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

    pub async fn add_routes(
        &mut self,
        tun_name: &str,
        #[cfg(not(target_os = "linux"))] entry_gateway_address: IpAddr,
        enable_ipv6: bool,
    ) -> Result<()> {
        let mut routes = HashSet::new();

        #[cfg(not(target_os = "linux"))]
        routes.insert(RequiredRoute::new(
            ipnetwork::IpNetwork::from(entry_gateway_address),
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

        #[cfg(target_os = "linux")]
        {
            routes = routes
                .into_iter()
                .map(|r| r.use_main_table(false))
                .collect();
        }

        #[cfg(target_os = "linux")]
        self.route_manager.create_routing_rules(enable_ipv6).await?;

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

        _ = tokio::task::spawn_blocking(|| drop(self.route_manager)).await;
    }

    #[cfg(target_os = "linux")]
    pub(super) fn inner_handle(&self) -> Result<talpid_routing::RouteManagerHandle> {
        Ok(self.route_manager.handle()?)
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
        write!(f, "Routing error")
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
