// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! The Uniffi generated bindings for the Nym VPN library. The API is designed to be used by
//! frontends to interact with the Nym VPN library. The API is designed to be platform-agnostic and
//! should work on any platform that supports the Uniffi FFI bindings.

#![cfg(any(target_os = "android", target_os = "ios", target_os = "macos"))]
uniffi::setup_scaffolding!();

use std::{net::IpAddr, path::PathBuf};

uniffi::use_remote_type!(nym_vpn_lib_types::IpAddr);
uniffi::use_remote_type!(nym_vpn_lib_types::PathBuf);

mod environment;
pub(crate) mod error;
mod favorites;
mod gateway_cache;
mod offline_monitor;

#[cfg(target_os = "ios")]
mod account;
#[cfg(target_os = "android")]
mod android_connectivity_monitor;
#[cfg(target_os = "macos")]
mod authorization_token;
#[cfg(target_os = "ios")]
mod deeplink;
#[cfg(target_os = "ios")]
mod local_authentication;
#[cfg(any(target_os = "android", target_os = "ios"))]
mod logging;
#[cfg(any(target_os = "android", target_os = "ios"))]
mod mobile;
#[cfg(target_os = "macos")]
mod rpc;
#[cfg(any(target_os = "android", target_os = "ios"))]
mod tunnel_provider;
#[cfg(target_os = "ios")]
mod vpn_account_storage;
#[cfg(any(target_os = "android", target_os = "ios"))]
mod vpn_service;
#[cfg(any(target_os = "android", target_os = "ios"))]
mod vpn_service_command_sender;
#[cfg(any(target_os = "android", target_os = "ios"))]
mod wake_signal;

#[cfg(any(target_os = "android", target_os = "ios"))]
pub use mobile::*;
