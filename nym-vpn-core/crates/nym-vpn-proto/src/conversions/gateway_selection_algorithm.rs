// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{conversions::ConversionError, proto};

impl From<nym_vpn_lib_types::GatewaySelectionAlgorithm> for proto::GatewaySelectionAlgorithm {
    fn from(value: nym_vpn_lib_types::GatewaySelectionAlgorithm) -> Self {
        let inner = match value {
            nym_vpn_lib_types::GatewaySelectionAlgorithm::Explicit => {
                proto::gateway_selection_algorithm::GatewaySelectionAlgorithm::Explicit(
                    proto::gateway_selection_algorithm::Explicit {},
                )
            }
            nym_vpn_lib_types::GatewaySelectionAlgorithm::AutoEntryExplicitExit => {
                proto::gateway_selection_algorithm::GatewaySelectionAlgorithm::AutoEntryExplicitExit(
                    proto::gateway_selection_algorithm::AutoEntryExplicitExit {},
                )
            }
            nym_vpn_lib_types::GatewaySelectionAlgorithm::Auto => {
                proto::gateway_selection_algorithm::GatewaySelectionAlgorithm::Auto(
                    proto::gateway_selection_algorithm::Auto {},
                )
            }
        };
        Self {
            gateway_selection_algorithm: Some(inner),
        }
    }
}

impl TryFrom<proto::GatewaySelectionAlgorithm> for nym_vpn_lib_types::GatewaySelectionAlgorithm {
    type Error = ConversionError;

    fn try_from(value: proto::GatewaySelectionAlgorithm) -> Result<Self, Self::Error> {
        let gateway_selection_algorithm =
            value
                .gateway_selection_algorithm
                .ok_or(ConversionError::NoValueSet(
                    "GatewaySelectionAlgorithm.gateway_selection_algorithm",
                ))?;

        let ret =match gateway_selection_algorithm {
            proto::gateway_selection_algorithm::GatewaySelectionAlgorithm::Explicit(_) => Self::Explicit,
            proto::gateway_selection_algorithm::GatewaySelectionAlgorithm::AutoEntryExplicitExit(_) => Self::AutoEntryExplicitExit,
            proto::gateway_selection_algorithm::GatewaySelectionAlgorithm::Auto(_) => Self::Auto,
        };
        Ok(ret)
    }
}

impl From<nym_vpn_lib_types::GatewaySelectionAlgorithmConfig>
    for proto::GatewaySelectionAlgorithmConfig
{
    fn from(value: nym_vpn_lib_types::GatewaySelectionAlgorithmConfig) -> Self {
        Self {
            enable_geo_location: value.enable_geo_location,
            gateway_selection_algorithm: Some(proto::GatewaySelectionAlgorithm::from(
                value.gateway_selection_algorithm,
            )),
        }
    }
}

impl TryFrom<proto::GatewaySelectionAlgorithmConfig>
    for nym_vpn_lib_types::GatewaySelectionAlgorithmConfig
{
    type Error = ConversionError;

    fn try_from(value: proto::GatewaySelectionAlgorithmConfig) -> Result<Self, Self::Error> {
        let gateway_selection_algorithm = value
            .gateway_selection_algorithm
            .ok_or(ConversionError::NoValueSet(
                "GatewaySelectionAlgorithmConfig.gateway_selection_algorithm",
            ))?
            .try_into()?;
        Ok(Self {
            enable_geo_location: value.enable_geo_location,
            gateway_selection_algorithm,
        })
    }
}
