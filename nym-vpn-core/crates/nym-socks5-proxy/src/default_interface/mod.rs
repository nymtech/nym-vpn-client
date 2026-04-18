// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

use std::net::{Ipv4Addr, Ipv6Addr};
use tokio::sync::watch;

#[derive(Clone, Debug, Default)]
pub struct DefaultInterface {
    #[cfg(target_os = "windows")]
    index: Option<u32>,
    pub v4_addr: Option<Ipv4Addr>,
    pub v6_addr: Option<Ipv6Addr>,
}

pub async fn start_monitor() -> watch::Receiver<DefaultInterface> {
    #[cfg(target_os = "windows")]
    return windows::start_monitor().await;

    #[cfg(target_os = "macos")]
    return macos::start_monitor().await;

    #[cfg(target_os = "linux")]
    {
        let (_, rx) = watch::channel(DefaultInterface::default());
        rx
    }
}
