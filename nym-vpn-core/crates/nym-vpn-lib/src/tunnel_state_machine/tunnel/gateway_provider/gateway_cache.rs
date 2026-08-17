// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_gateway_directory::{Error, GatewayCacheHandle, GatewayClient, GatewayList, GatewayType};

#[async_trait::async_trait]
pub trait GatewayCache: Clone + Send + Sync + 'static {
    async fn lookup_gateways(&self, gw_type: GatewayType) -> Result<GatewayList, Error>;
    fn replace_gateway_client(&self, gateway_client: GatewayClient) -> Result<(), Error>;
    fn set_paused(&self, paused: bool) -> Result<(), Error>;
}

#[async_trait::async_trait]
impl GatewayCache for GatewayCacheHandle {
    async fn lookup_gateways(&self, gw_type: GatewayType) -> Result<GatewayList, Error> {
        self.lookup_gateways(gw_type).await
    }

    fn replace_gateway_client(&self, gateway_client: GatewayClient) -> Result<(), Error> {
        self.replace_gateway_client(gateway_client)
    }

    fn set_paused(&self, paused: bool) -> Result<(), Error> {
        self.set_paused(paused)
    }
}

#[cfg(test)]
pub mod tests {
    use std::{sync::Arc, time::Duration};

    use nym_gateway_directory::Gateway;
    use tokio::sync::RwLock;

    use super::*;

    #[derive(Clone)]
    pub struct MockGatewayCache {
        gateways: Arc<RwLock<Option<Vec<Gateway>>>>,
        /// Artificial delay applied to every `lookup_gateways` call, used to
        /// model a fresh selection that hasn't landed a value in the stream yet.
        lookup_delay: Duration,
    }

    impl MockGatewayCache {
        pub fn new(gateways: Arc<RwLock<Option<Vec<Gateway>>>>) -> Self {
            Self {
                gateways,
                lookup_delay: Duration::ZERO,
            }
        }

        pub fn new_with_lookup_delay(
            gateways: Arc<RwLock<Option<Vec<Gateway>>>>,
            lookup_delay: Duration,
        ) -> Self {
            Self {
                gateways,
                lookup_delay,
            }
        }
    }

    #[async_trait::async_trait]
    impl GatewayCache for MockGatewayCache {
        async fn lookup_gateways(&self, gw_type: GatewayType) -> Result<GatewayList, Error> {
            if !self.lookup_delay.is_zero() {
                tokio::time::sleep(self.lookup_delay).await;
            }
            Ok(GatewayList::new(
                Some(gw_type),
                self.gateways.read().await.clone().unwrap_or_default(),
            ))
        }

        fn replace_gateway_client(&self, _gateway_client: GatewayClient) -> Result<(), Error> {
            Ok(())
        }

        fn set_paused(&self, _paused: bool) -> Result<(), Error> {
            Ok(())
        }
    }
}
