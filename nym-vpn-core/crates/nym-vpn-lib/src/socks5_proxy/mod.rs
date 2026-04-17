// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

mod manager;
mod process;

pub use manager::Socks5ProxyManager;

// This doesn't need re-exporting on Linux as it's only used in this crate.
#[cfg(any(target_os = "macos", target_os = "windows"))]
pub use process::find_proxy_binary;
