// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

pub mod account;
pub mod ad_block;
pub mod device;
pub mod diagnostic;
pub mod dns;
pub mod favorites;
pub mod gateway;
pub mod geo_exclusion;
pub mod lan;
pub mod network;
pub mod network_stats;
pub mod profile;
pub mod sentry;
pub mod session;
pub mod socks5;
#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
pub mod split_tunnel;
pub mod tunnel;
