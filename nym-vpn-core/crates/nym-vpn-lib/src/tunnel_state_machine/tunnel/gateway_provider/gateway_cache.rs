// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_gateway_directory::{Error, GatewayCacheHandle, GatewayClient, GatewayList, GatewayType};

#[async_trait::async_trait]
pub trait GatewayCache: Clone + Send + Sync + 'static {
    async fn lookup_gateways(&self, gw_type: GatewayType) -> Result<GatewayList, Error>;
    fn replace_gateway_client(&self, gateway_client: GatewayClient) -> Result<(), Error>;
    async fn refresh_all(&self) -> Result<(), Error>;
}

#[async_trait::async_trait]
impl GatewayCache for GatewayCacheHandle {
    async fn lookup_gateways(&self, gw_type: GatewayType) -> Result<GatewayList, Error> {
        self.lookup_gateways(gw_type).await
    }

    fn replace_gateway_client(&self, gateway_client: GatewayClient) -> Result<(), Error> {
        self.replace_gateway_client(gateway_client)
    }

    async fn refresh_all(&self) -> Result<(), Error> {
        self.refresh_all().await
    }
}

#[cfg(test)]
pub mod tests {
    use std::sync::Arc;

    use nym_gateway_directory::Gateway;
    use tokio::sync::RwLock;

    use super::*;

    #[derive(Clone)]
    pub struct MockGatewayCache {
        gateways: Arc<RwLock<Option<Vec<Gateway>>>>,
    }

    impl MockGatewayCache {
        pub fn new(gateways: Arc<RwLock<Option<Vec<Gateway>>>>) -> Self {
            Self { gateways }
        }
    }

    #[async_trait::async_trait]
    impl GatewayCache for MockGatewayCache {
        async fn lookup_gateways(&self, gw_type: GatewayType) -> Result<GatewayList, Error> {
            Ok(GatewayList::new(
                Some(gw_type),
                self.gateways.read().await.clone().unwrap_or_default(),
            ))
        }

        fn replace_gateway_client(&self, _gateway_client: GatewayClient) -> Result<(), Error> {
            Ok(())
        }

        async fn refresh_all(&self) -> Result<(), Error> {
            Ok(())
        }
    }
}
