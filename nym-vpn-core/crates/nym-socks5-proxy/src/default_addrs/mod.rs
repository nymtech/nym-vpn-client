// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

use nym_socks5_proxy_ipc::InterfaceAddresses;
use tokio::sync::watch;

pub async fn start_monitor() -> watch::Receiver<InterfaceAddresses> {
    #[cfg(target_os = "windows")]
    return windows::start_monitor().await;

    #[cfg(target_os = "macos")]
    return macos::start_monitor().await;

    #[cfg(target_os = "linux")]
    {
        let (_, rx) = watch::channel(InterfaceAddresses::default());
        return rx;
    }
}
