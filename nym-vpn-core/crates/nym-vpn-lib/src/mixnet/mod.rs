// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

mod backpressure;
mod error;
mod mixnet_listener;
mod processor;
mod topology_service;

pub use processor::start_processor;

pub use error::MixnetError;
pub use topology_service::{
    VpnTopologyProvider, VpnTopologyService, VpnTopologyServiceError, VpnTopologyServiceHandle,
};

pub const DEFAULT_MIN_MIXNODE_PERFORMANCE: u8 = 50;
pub const DEFAULT_MIN_GATEWAY_PERFORMANCE: u8 = 50;
