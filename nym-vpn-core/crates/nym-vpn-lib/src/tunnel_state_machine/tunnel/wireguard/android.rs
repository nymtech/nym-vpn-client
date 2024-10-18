use std::{fmt::Debug, os::fd::RawFd};

use super::tunnel_settings::TunnelSettings;
use crate::platform::error::VpnError;

#[uniffi::export(with_foreign)]
pub trait AndroidTunProvider: Send + Sync + Debug {
    fn bypass(&self, socket: i32);
    fn configure_tunnel(&self, config: TunnelNetworkSettings) -> Result<RawFd, VpnError>;
}
