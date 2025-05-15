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
    Address, AddressEndpoint, BonjourServiceEndpoint, Endpoint, HostEndpoint, UnknownEndpoint,
    UrlEndpoint, nw_endpoint_type_t,
};
pub use interface::{Interface, InterfaceType, nw_interface_type_t};
pub use path::{Path, PathStatus, nw_path_status_t};
pub use path_monitor::PathMonitor;
