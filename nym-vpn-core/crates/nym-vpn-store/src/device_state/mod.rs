// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::error::Error;

use bip39::Mnemonic;
use nym_credentials_interface::KeyPairUser;
use nym_validator_client::{
    DirectSecp256k1HdWallet,
    signing::{direct_wallet::DirectSecp256k1HdWalletError, signer::OfflineSigner},
};
use nym_vpn_lib_types::{StoredAccountMode, VpnAccountSummary};
use serde::{Deserialize, Serialize};
use zeroize::ZeroizeOnDrop;

pub mod ephemeral;
pub mod on_disk;

#[derive(Debug, thiserror::Error)]
pub enum DeviceAccountError {
    #[error("wallet error")]
    Wallet(#[from] DirectSecp256k1HdWalletError),

    #[error("no accounts in wallet")]
    NoAccounts,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoredAccountIdentity {
    /// Cosmos account identifier of the first derived account.
    pub id: String, // nym_validator_client::nyxd::AccountId,

    /// Base58-encoded public (secp256k1) key of the first derived account.
    pub pub_key: String,

    /// Mode of operation of this account
    pub mode: StoredAccountMode,

    /// Base64-encoded signature on the account identifier of this account.
    // Note that it is seemingly (?) only ever used once during account creation,
    // so lack of replay protection whilst sketchy is fine.
    pub signature_base64: String,
}

impl StoredAccountIdentity {
    pub fn new(mnemonic: Mnemonic, mode: StoredAccountMode) -> Result<Self, DeviceAccountError> {
        let wallet = DirectSecp256k1HdWallet::checked_from_mnemonic("n", mnemonic)?;
        Self::derive_from_wallet(wallet, mode)
    }

    fn derive_from_wallet(
        wallet: DirectSecp256k1HdWallet,
        mode: StoredAccountMode,
    ) -> Result<Self, DeviceAccountError> {
        let accounts = wallet.get_accounts();
        let Some(first) = accounts.first() else {
            return Err(DeviceAccountError::NoAccounts);
        };
        let address = first.address();
        let id = address.to_string();
        let raw_pub_key = first.public_key();
        let pub_key = bs58::encode(raw_pub_key.to_bytes()).into_string();

        let signature = wallet.sign_raw(address, &id)?;
        let signature_bytes = signature.to_bytes();
        let signature_base64 = base64_url::encode(&signature_bytes);

        Ok(Self {
            id,
            pub_key,
            mode,
            signature_base64,
        })
    }
}

pub const ECASH_SEED_LEN: usize = 32;
// pub type EcashSeed = [u8; ECASH_SEED_LEN];

#[derive(Clone, ZeroizeOnDrop, Serialize, Deserialize, Eq, PartialEq)]
pub struct EcashSeed([u8; ECASH_SEED_LEN]);

impl AsRef<[u8; ECASH_SEED_LEN]> for EcashSeed {
    fn as_ref(&self) -> &[u8; ECASH_SEED_LEN] {
        &self.0
    }
}

impl std::fmt::Debug for EcashSeed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EcashSeed").finish_non_exhaustive()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum StoredDeviceState {
    /// Device is logged out
    #[default]
    LoggedOut,

    /// Device is logged in
    LoggedIn {
        account_identity: StoredAccountIdentity,
        ecash_keypair: KeyPairUser,
        summary: Option<VpnAccountSummary>,
    },
}

impl StoredDeviceState {
    /// Returns account summary if available
    pub fn account_summary(&self) -> &Option<VpnAccountSummary> {
        match self {
            Self::LoggedIn {
                summary: account_summary,
                ..
            } => account_summary,
            Self::LoggedOut => &None,
        }
    }

    /// Returns device account when logged in
    pub fn account(&self) -> Option<&StoredAccountIdentity> {
        match self {
            Self::LoggedIn {
                account_identity: account,
                ..
            } => Some(account),
            Self::LoggedOut => None,
        }
    }
}

#[async_trait::async_trait]
pub trait DeviceStateStorage {
    type StorageError: Error + Send + Sync + 'static;

    /// Returns [`DeviceState::LoggedOut`] when no state is stored.
    async fn load_device_state(&self) -> Result<StoredDeviceState, Self::StorageError>;

    async fn store_device_state(&self, state: StoredDeviceState) -> Result<(), Self::StorageError>;

    async fn remove_device_state(&self) -> Result<(), Self::StorageError>;
}
