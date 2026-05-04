// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

mod manager;

#[cfg(not(target_os = "android"))]
mod process;

#[cfg(target_os = "android")]
mod task;

pub use manager::Socks5ProxyManager;

#[cfg(any(target_os = "macos", target_os = "windows"))]
pub use process::find_proxy_binary;

#[derive(Debug)]
enum Socks5ProxyEvent {
    Ready,
    Error { message: String },
    Exited { success: bool },
}
