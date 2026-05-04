// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

pub mod default_interface;
pub mod proxy;
pub mod routing;

pub use nym_socks5_proxy_ipc::{InterfaceAddresses, ProxyConfig};
pub use proxy::run;

#[cfg(target_os = "android")]
pub type SocketProtector = std::sync::Arc<dyn Fn(i32) + Send + Sync>;
