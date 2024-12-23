// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Minimalistic wrapper for Apple Network framework.
//! Documentation: <https://developer.apple.com/documentation/network?language=objc>

#![cfg(any(target_os = "macos", target_os = "ios"))]

mod endpoint;
mod interface;
mod path;
mod path_monitor;
mod rc;
mod sys;

pub use endpoint::{
    nw_endpoint_type_t, Address, AddressEndpoint, BonjourServiceEndpoint, Endpoint, HostEndpoint,
    UnknownEndpoint, UrlEndpoint,
};
pub use interface::{nw_interface_type_t, Interface, InterfaceType};
pub use path::{nw_path_status_t, Path, PathStatus};
pub use path_monitor::PathMonitor;
