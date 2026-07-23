// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_gateway_directory::{Error, GatewayCacheHandle, GatewayList, GatewayType};

#[async_trait::async_trait]
pub trait RecentGatewayCache: Send + Sync {
    async fn lookup_gateways(&self, gw_type: GatewayType) -> Result<GatewayList, Error>;
}

#[async_trait::async_trait]
impl RecentGatewayCache for GatewayCacheHandle {
    async fn lookup_gateways(&self, gw_type: GatewayType) -> Result<GatewayList, Error> {
        self.lookup_gateways(gw_type).await
    }
}
