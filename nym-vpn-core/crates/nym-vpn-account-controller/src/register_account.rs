// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_store::mnemonic::Mnemonic;

#[derive(Clone, Debug)]
pub struct RegisterAccountResponse {
    pub account_token: String,
    pub mnemonic: Mnemonic,
}
