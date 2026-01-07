// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::service::{
    ConfigSetupError,
    config::entry_exit::legacy::{EntryPoint, ExitPoint},
};
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) struct VpnServiceConfig {
    pub(crate) entry_point: EntryPoint,
    pub(crate) exit_point: ExitPoint,
}

impl TryFrom<VpnServiceConfig> for nym_vpn_lib_types::VpnServiceConfig {
    type Error = ConfigSetupError;

    fn try_from(value: VpnServiceConfig) -> Result<Self, Self::Error> {
        Ok(Self {
            entry_point: value.entry_point.try_into()?,
            exit_point: value.exit_point.try_into()?,
            ..Default::default()
        })
    }
}
