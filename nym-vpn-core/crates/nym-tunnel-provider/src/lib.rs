// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Facilities for interacting with:
//! - Packet tunnel provider on iOS
//! - VpnService on Android

uniffi::setup_scaffolding!();

pub mod error;
mod uniffi_custom_impls;

#[cfg(target_os = "android")]
pub mod android;
#[cfg(target_os = "ios")]
pub mod ios;
#[cfg(any(target_os = "ios", target_os = "android"))]
pub mod tunnel_settings;
