// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Facilities for interacting with:
//! - Packet tunnel provider on iOS
//! - VpnService on Android

#[cfg(target_os = "android")]
pub mod android;
#[cfg(target_os = "ios")]
pub mod ios;
pub mod tunnel_settings;
