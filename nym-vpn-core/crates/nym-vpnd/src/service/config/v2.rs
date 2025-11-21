// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::service::{ConfigSetupError, config::VpnServiceConfigExt, error::Result};
use nym_vpn_lib_types::{EntryPoint, ExitPoint, NodeIdentity, Recipient, VpnServiceConfig};
use serde::{Deserialize, Serialize};
use std::{net::IpAddr, str::FromStr};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VpnServiceConfigExtV2 {
    entry_point: EntryPointExtV2,
    exit_point: ExitPointExtV2,
    dns: Option<String>,
    allow_lan: bool,
    disable_ipv6: bool,
    enable_two_hop: bool,
    enable_bridges: bool,
    netstack: bool,
    disable_poisson_rate: bool,
    disable_background_cover_traffic: bool,
    min_mixnode_performance: Option<u8>,
    min_gateway_mixnet_performance: Option<u8>,
    min_gateway_vpn_performance: Option<u8>,
    residential_exit: bool,
}

impl From<VpnServiceConfigExtV2> for VpnServiceConfigExt {
    fn from(v2: VpnServiceConfigExtV2) -> Self {
        VpnServiceConfigExt::V2(v2)
    }
}

impl TryFrom<VpnServiceConfigExtV2> for VpnServiceConfig {
    type Error = ConfigSetupError;

    fn try_from(value: VpnServiceConfigExtV2) -> Result<Self, Self::Error> {
        let dns = value
            .dns
            .map(|addr| {
                IpAddr::from_str(&addr)
                    .map_err(|e| ConfigSetupError::IpAddress { error: Box::new(e) })
            })
            .transpose()?;

        let config = VpnServiceConfig {
            entry_point: EntryPoint::try_from(value.entry_point)?,
            exit_point: ExitPoint::try_from(value.exit_point)?,
            allow_lan: value.allow_lan,
            disable_ipv6: value.disable_ipv6,
            enable_two_hop: value.enable_two_hop,
            enable_bridges: value.enable_bridges,
            netstack: value.netstack,
            disable_poisson_rate: value.disable_poisson_rate,
            disable_background_cover_traffic: value.disable_background_cover_traffic,
            min_mixnode_performance: value.min_mixnode_performance,
            min_gateway_mixnet_performance: value.min_gateway_mixnet_performance,
            min_gateway_vpn_performance: value.min_gateway_vpn_performance,
            residential_exit: value.residential_exit,
            custom_dns: dns.map(|addr| vec![addr]),
        };
        Ok(config)
    }
}

//
// EntryPointExtV2
//

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntryPointExtV2 {
    Gateway { identity: String },
    Country { two_letter_iso_country_code: String },
    Region { region: String },
    Random,
}

impl TryFrom<EntryPointExtV2> for EntryPoint {
    type Error = ConfigSetupError;

    fn try_from(value: EntryPointExtV2) -> Result<Self, Self::Error> {
        match value {
            EntryPointExtV2::Gateway { ref identity } => EntryPoint::from_base58_string(identity)
                .map_err(|e| ConfigSetupError::EntryPoint(e.to_string())),
            EntryPointExtV2::Country {
                two_letter_iso_country_code,
            } => Ok(EntryPoint::Country {
                two_letter_iso_country_code,
            }),
            EntryPointExtV2::Region { region } => Ok(EntryPoint::Region { region }),
            EntryPointExtV2::Random => Ok(EntryPoint::Random),
        }
    }
}

impl TryFrom<&EntryPoint> for EntryPointExtV2 {
    type Error = ConfigSetupError;

    fn try_from(value: &EntryPoint) -> Result<Self, Self::Error> {
        match value {
            EntryPoint::Gateway { identity } => Ok(EntryPointExtV2::Gateway {
                identity: identity.to_base58_string(),
            }),
            EntryPoint::Country {
                two_letter_iso_country_code,
            } => Ok(EntryPointExtV2::Country {
                two_letter_iso_country_code: two_letter_iso_country_code.clone(),
            }),
            EntryPoint::Region { region } => Ok(EntryPointExtV2::Region {
                region: region.clone(),
            }),
            EntryPoint::Random => Ok(EntryPointExtV2::Random),
        }
    }
}

//
// ExitPointExtV2
//

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ExitPointExtV2 {
    Address { address: String },
    Gateway { identity: String },
    Country { two_letter_iso_country_code: String },
    Region { region: String },
    Random,
}

impl TryFrom<ExitPointExtV2> for ExitPoint {
    type Error = ConfigSetupError;

    fn try_from(value: ExitPointExtV2) -> Result<Self, Self::Error> {
        match value {
            ExitPointExtV2::Address { address } => {
                let recipient = Recipient::try_from_base58_string(&address)
                    .map_err(|e| ConfigSetupError::ExitPoint(e.to_string()))?;
                Ok(ExitPoint::Address {
                    address: Box::new(recipient),
                })
            }
            ExitPointExtV2::Gateway { identity } => {
                let node_identity = NodeIdentity::from_str(&identity)
                    .map_err(|e| ConfigSetupError::ExitPoint(e.to_string()))?;
                Ok(ExitPoint::Gateway {
                    identity: node_identity,
                })
            }
            ExitPointExtV2::Country {
                two_letter_iso_country_code,
            } => Ok(ExitPoint::Country {
                two_letter_iso_country_code,
            }),
            ExitPointExtV2::Region { region } => Ok(ExitPoint::Region { region }),
            ExitPointExtV2::Random => Ok(ExitPoint::Random),
        }
    }
}

impl TryFrom<&ExitPoint> for ExitPointExtV2 {
    type Error = ConfigSetupError;

    fn try_from(value: &ExitPoint) -> Result<Self, Self::Error> {
        match value {
            ExitPoint::Address { address } => Ok(ExitPointExtV2::Address {
                address: address.to_string(),
            }),
            ExitPoint::Gateway { identity } => Ok(ExitPointExtV2::Gateway {
                identity: identity.to_string(),
            }),
            ExitPoint::Country {
                two_letter_iso_country_code,
            } => Ok(ExitPointExtV2::Country {
                two_letter_iso_country_code: two_letter_iso_country_code.clone(),
            }),
            ExitPoint::Region { region } => Ok(ExitPointExtV2::Region {
                region: region.clone(),
            }),
            ExitPoint::Random => Ok(ExitPointExtV2::Random),
        }
    }
}
