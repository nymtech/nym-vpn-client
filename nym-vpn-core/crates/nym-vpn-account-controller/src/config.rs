// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::path::PathBuf;

use nym_sdk::UserAgent;
use nym_vpn_network_config::Network;

pub struct AccountControllerConfig {
    // The data directory where we store the account and device keys.
    pub data_dir: PathBuf,

    // User agent used by api client.
    pub user_agent: UserAgent,

    // Credentials mode is a feature flag that determines if we should automatically request
    // zk-nyms.
    pub credentials_mode: Option<bool>,

    // The network environment that the controller is running in.
    pub network_env: Network,
}

impl AccountControllerConfig {
    // Determine if the credentials mode is enabled. This is determined by the credentials_mode
    // field in the config, if it is set. Else the network environment feature flag is used.
    pub fn background_zk_nym_refresh(&self) -> bool {
        self.credentials_mode.unwrap_or_else(|| {
            self.network_env
                .get_feature_flag_credential_mode()
                .unwrap_or(false)
        })
    }
}
