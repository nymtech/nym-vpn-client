// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{fmt::Debug, os::fd::RawFd};

use super::tunnel_settings::TunnelNetworkSettings;
use crate::platform::error::VpnError;

#[uniffi::export(with_foreign)]
pub trait AndroidTunProvider: Send + Sync + Debug {
    fn bypass(&self, socket: i32);
    fn configure_tunnel(&self, config: TunnelNetworkSettings) -> Result<RawFd, VpnError>;
}
