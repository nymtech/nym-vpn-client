// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::service::{ConfigSetupError, config::VpnServiceConfigExt, error::Result};
use nym_vpn_lib_types::{EntryPoint, ExitPoint, NodeIdentity, Recipient, VpnServiceConfig};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VpnServiceConfigExtV1 {
    entry_point: EntryPointExtV1,
    exit_point: ExitPointExtV1,
}

impl From<VpnServiceConfigExtV1> for VpnServiceConfigExt {
    fn from(v1: VpnServiceConfigExtV1) -> Self {
        VpnServiceConfigExt::V1(v1)
    }
}

impl TryFrom<VpnServiceConfigExtV1> for VpnServiceConfig {
    type Error = ConfigSetupError;

    fn try_from(value: VpnServiceConfigExtV1) -> Result<Self, Self::Error> {
        let config = VpnServiceConfig {
            entry_point: EntryPoint::try_from(value.entry_point)?,
            exit_point: ExitPoint::try_from(value.exit_point)?,
            ..Default::default()
        };
        Ok(config)
    }
}

//
// EntryPointExtV1
//

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntryPointExtV1 {
    Gateway { identity: String },
    Location { location: String },
    Random,
}

impl TryFrom<EntryPointExtV1> for EntryPoint {
    type Error = ConfigSetupError;

    fn try_from(value: EntryPointExtV1) -> Result<Self, Self::Error> {
        match value {
            EntryPointExtV1::Gateway { ref identity } => EntryPoint::from_base58_string(identity)
                .map_err(|e| ConfigSetupError::EntryPoint(e.to_string())),
            EntryPointExtV1::Location { location } => Ok(EntryPoint::Country {
                two_letter_iso_country_code: location,
            }),
            EntryPointExtV1::Random => Ok(EntryPoint::Random),
        }
    }
}

//
// ExitPointExtV1
//

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ExitPointExtV1 {
    Address { address: String },
    Gateway { identity: String },
    Location { location: String },
    Random,
}

impl TryFrom<ExitPointExtV1> for ExitPoint {
    type Error = ConfigSetupError;

    fn try_from(value: ExitPointExtV1) -> Result<Self, Self::Error> {
        match value {
            ExitPointExtV1::Address { address } => {
                let recipient = Recipient::try_from_base58_string(&address)
                    .map_err(|e| ConfigSetupError::ExitPoint(e.to_string()))?;
                Ok(ExitPoint::Address {
                    address: Box::new(recipient),
                })
            }
            ExitPointExtV1::Gateway { identity } => {
                let node_identity = NodeIdentity::from_str(&identity)
                    .map_err(|e| ConfigSetupError::ExitPoint(e.to_string()))?;
                Ok(ExitPoint::Gateway {
                    identity: node_identity,
                })
            }
            ExitPointExtV1::Location { location } => Ok(ExitPoint::Country {
                two_letter_iso_country_code: location,
            }),
            ExitPointExtV1::Random => Ok(ExitPoint::Random),
        }
    }
}
