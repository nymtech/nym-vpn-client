// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "linux")]
pub use linux::set_socket_tunnel_fwmark;

#[cfg(target_os = "windows")]
pub use windows::set_socket_interface_index;

use std::{
    fmt::{Debug, Formatter},
    net::{Ipv4Addr, Ipv6Addr},
};
use tokio::sync::watch;
use tokio_util::sync::CancellationToken;

#[cfg(target_os = "windows")]
use ::windows::Win32::NetworkManagement::Ndis::NET_LUID_LH;

#[derive(Clone, Default)]
pub struct DefaultInterface {
    #[cfg(target_os = "windows")]
    pub v4_luid: Option<NET_LUID_LH>,

    pub v4_addr: Option<Ipv4Addr>,

    #[cfg(target_os = "windows")]
    pub v6_luid: Option<NET_LUID_LH>,

    pub v6_addr: Option<Ipv6Addr>,
}

impl Debug for DefaultInterface {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut s = f.debug_struct("DefaultInterface");
        #[cfg(target_os = "windows")]
        s.field("v4_luid", &self.v4_luid.map(|l| unsafe { l.Value }));
        s.field("v4_addr", &self.v4_addr);
        #[cfg(target_os = "windows")]
        s.field("v6_luid", &self.v6_luid.map(|l| unsafe { l.Value }));
        s.field("v6_addr", &self.v6_addr);
        s.finish()
    }
}

pub async fn start_monitor(shutdown_token: CancellationToken) -> watch::Receiver<DefaultInterface> {
    #[cfg(target_os = "windows")]
    return windows::start_monitor(shutdown_token).await;

    #[cfg(target_os = "macos")]
    return macos::start_monitor(shutdown_token).await;

    #[cfg(any(target_os = "linux", target_os = "android"))]
    {
        let _ = shutdown_token;
        let (_, rx) = watch::channel(DefaultInterface::default());
        rx
    }
}
