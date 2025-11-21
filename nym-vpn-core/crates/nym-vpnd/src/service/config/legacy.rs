// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::service::{ConfigSetupError, error::Result};
use nym_vpn_lib_types::{EntryPoint, ExitPoint, NodeIdentity, Recipient, VpnServiceConfig};
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct VpnServiceConfigExtLegacy {
    entry_point: EntryPointLegacy,
    exit_point: ExitPointLegacy,
}

impl TryFrom<VpnServiceConfigExtLegacy> for VpnServiceConfig {
    type Error = ConfigSetupError;

    fn try_from(value: VpnServiceConfigExtLegacy) -> Result<Self, Self::Error> {
        Ok(Self {
            entry_point: value.entry_point.try_into()?,
            exit_point: value.exit_point.try_into()?,
            ..Default::default()
        })
    }
}

#[derive(Clone, Debug, Deserialize)]
enum EntryPointLegacy {
    Gateway { identity: Vec<u8> },
    Location { location: String },
    Random,
}

impl TryFrom<EntryPointLegacy> for EntryPoint {
    type Error = ConfigSetupError;

    fn try_from(value: EntryPointLegacy) -> Result<Self, Self::Error> {
        match value {
            EntryPointLegacy::Gateway { identity } => Ok(EntryPoint::Gateway {
                identity: NodeIdentity::from_bytes(&identity)
                    .map_err(|e| ConfigSetupError::EntryPoint(e.to_string()))?,
            }),
            EntryPointLegacy::Location { location } => Ok(EntryPoint::Country {
                two_letter_iso_country_code: location,
            }),
            EntryPointLegacy::Random => Ok(EntryPoint::Random),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
enum ExitPointLegacy {
    Address { address: String },
    Gateway { identity: Vec<u8> },
    Location { location: String },
    Random,
}

impl TryFrom<ExitPointLegacy> for ExitPoint {
    type Error = ConfigSetupError;

    fn try_from(value: ExitPointLegacy) -> Result<Self, Self::Error> {
        match value {
            ExitPointLegacy::Address { address } => {
                let recipient = Recipient::try_from_base58_string(&address)
                    .map_err(|e| ConfigSetupError::ExitPoint(e.to_string()))?;
                Ok(ExitPoint::Address {
                    address: Box::new(recipient),
                })
            }
            ExitPointLegacy::Gateway { identity } => Ok(ExitPoint::Gateway {
                identity: NodeIdentity::from_bytes(&identity)
                    .map_err(|e| ConfigSetupError::ExitPoint(e.to_string()))?,
            }),
            ExitPointLegacy::Location { location } => Ok(ExitPoint::Country {
                two_letter_iso_country_code: location,
            }),
            ExitPointLegacy::Random => Ok(ExitPoint::Random),
        }
    }
}
