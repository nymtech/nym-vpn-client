// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_gateway_directory::{GatewayCacheHandle, GatewayList, GatewayType};

use crate::recents::error::RecentsError;

#[async_trait::async_trait]
pub trait GatewayCache: Clone + Send + Sync + 'static {
    async fn lookup_gateways(&self, gw_type: GatewayType) -> Result<GatewayList, RecentsError>;
}

#[async_trait::async_trait]
impl GatewayCache for GatewayCacheHandle {
    async fn lookup_gateways(&self, gw_type: GatewayType) -> Result<GatewayList, RecentsError> {
        self.lookup_gateways(gw_type)
            .await
            .map_err(|source| RecentsError::GetGateways { source })
    }
}
