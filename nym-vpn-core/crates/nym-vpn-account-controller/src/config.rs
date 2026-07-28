// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::PathBuf;

use nym_sdk::mixnet::StoragePaths;
use nym_vpn_network_config::Network;

pub struct AccountControllerConfig {
    // The data directory where we store the account and device keys.
    pub data_dir: PathBuf,

    // Storage paths derived from the data directory, used to locate the on-disk databases.
    pub storage_paths: StoragePaths,

    // The network environment that the controller is running in.
    pub network_env: Network,
}
