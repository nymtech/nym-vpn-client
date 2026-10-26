// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::error::Error;

use nym_vpn_lib_types::{StoredAccountMode, VpnAccountSummary};
use serde::{Deserialize, Serialize};

pub mod ephemeral;
pub mod on_disk;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct DeviceAccount {
    /// Cosmos account identifier of the first derived account.
    id: AccountId,

    /// Base58-encoded public (secp256k1) key of the first derived account.
    pub_key: String,

    /// Mode of operation of this account
    mode: StoredAccountMode,

    /// Base64-encoded signature on the account identifier of this account.
    // Note that it is seemingly (?) only ever used once during account creation,
    // so lack of replay protection whilst sketchy is fine.
    signature_base64: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DeviceState {
    /// Device is logged out
    #[default]
    LoggedOut,

    /// Device is logged in
    LoggedIn {
        // mnemonic: stored separately
        account: DeviceAccount,
        account_summary: Option<VpnAccountSummary>,
    },
}

impl DeviceState {
    pub fn account_summary(&self) -> &Option<VpnAccountSummary> {
        match self {
            Self::LoggedIn {
                account_summary, ..
            } => account_summary,
            Self::LoggedOut => &None,
        }
    }
}

#[async_trait::async_trait]
pub trait DeviceStateStorage {
    type StorageError: Error + Send + Sync + 'static;

    /// Returns [`DeviceState::LoggedOut`] when no state is stored.
    async fn load_device_state(&self) -> Result<DeviceState, Self::StorageError>;

    async fn store_device_state(&self, state: DeviceState) -> Result<(), Self::StorageError>;

    async fn remove_device_state(&self) -> Result<(), Self::StorageError>;
}
