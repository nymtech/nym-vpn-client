// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

uniffi::setup_scaffolding!();

use std::path::PathBuf;
use time::OffsetDateTime;

uniffi::use_remote_type!(nym_vpn_lib_types_uniffi::PathBuf);
uniffi::use_remote_type!(nym_vpn_lib_types_uniffi::OffsetDateTime);

pub mod gateway;
pub mod log_path;
pub mod nym_vpn_api;
pub mod service;
