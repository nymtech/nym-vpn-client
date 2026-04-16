// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

use nym_socks5_proxy_ipc::InterfaceAddresses;
use tokio::sync::watch;

pub async fn start_monitor() -> watch::Receiver<InterfaceAddresses> {
    #[cfg(target_os = "windows")]
    return windows::start().await;

    #[cfg(target_os = "macos")]
    return macos::start().await;

    #[cfg(target_os = "linux")]
    return linux::start().await;
}
