// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{conversions::ConversionError, proto};

impl From<nym_vpn_lib_types::GetRecentGatewaysParams> for proto::GetRecentGatewaysParams {
    fn from(value: nym_vpn_lib_types::GetRecentGatewaysParams) -> Self {
        Self {
            tunnel_type: proto::TunnelType::from(value.tunnel_type) as i32,
        }
    }
}

impl TryFrom<proto::GetRecentGatewaysParams> for nym_vpn_lib_types::GetRecentGatewaysParams {
    type Error = ConversionError;

    fn try_from(value: proto::GetRecentGatewaysParams) -> Result<Self, Self::Error> {
        let tunnel_type = proto::TunnelType::try_from(value.tunnel_type)
            .map_err(|e| ConversionError::Decode("TunnelType", e))
            .map(nym_vpn_lib_types::TunnelType::from)?;

        Ok(Self { tunnel_type })
    }
}
