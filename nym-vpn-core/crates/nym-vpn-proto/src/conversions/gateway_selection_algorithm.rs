// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::proto;

impl From<nym_vpn_lib_types::GatewaySelectionAlgorithmConfig>
    for proto::GatewaySelectionAlgorithmConfig
{
    fn from(value: nym_vpn_lib_types::GatewaySelectionAlgorithmConfig) -> Self {
        Self {
            enable_geo_location: value.enable_geo_location,
        }
    }
}

impl From<proto::GatewaySelectionAlgorithmConfig>
    for nym_vpn_lib_types::GatewaySelectionAlgorithmConfig
{
    fn from(value: proto::GatewaySelectionAlgorithmConfig) -> Self {
        Self::new(value.enable_geo_location)
    }
}
