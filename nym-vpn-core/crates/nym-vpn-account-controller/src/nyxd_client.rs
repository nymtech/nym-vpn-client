// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_validator_client::DirectSigningHttpRpcNyxdClient;
use nym_vpn_lib_types::AccountControllerError;
use nym_vpn_network_config::Network;

pub struct NyxdClient(DirectSigningHttpRpcNyxdClient);

impl NyxdClient {
    pub fn new(network: &Network) -> Result<Self, AccountControllerError> {
        todo!()
    }
}
