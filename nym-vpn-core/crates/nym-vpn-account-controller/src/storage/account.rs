// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_api_client::types::{Device, VpnApiAccount};
use nym_vpn_store::{VpnStorage, mnemonic::Mnemonic};

use crate::{commands::ReturnSender, error::Error};

#[derive(Debug)]
pub(crate) struct AccountStorage<S>
where
    S: VpnStorage,
{
    storage: S,
}

// SW we might not need that mutex in the end
impl<S> AccountStorage<S>
where
    S: VpnStorage,
{
    pub(crate) fn from(storage: S) -> Self {
        Self { storage }
    }

    pub(crate) async fn store_account(&self, mnemonic: Mnemonic) -> Result<(), Error> {
        self.storage
            .store_mnemonic(mnemonic)
            .await
            .map_err(|err| Error::MnemonicStore {
                source: Box::new(err),
            })
    }

    pub(crate) async fn load_mnemonic(&self) -> Result<Mnemonic, Error> {
        self.storage
            .load_mnemonic()
            .await
            .map_err(|err| Error::MnemonicStore {
                source: Box::new(err),
            })
    }

    pub(crate) async fn load_account(&self) -> Result<VpnApiAccount, Error> {
        let mnemonic = self.load_mnemonic().await?;
        VpnApiAccount::try_from(mnemonic).map_err(Error::internal)
    }

    pub(crate) async fn remove_account(&self) -> Result<(), Error> {
        self.storage
            .remove_mnemonic()
            .await
            .map_err(|err| Error::MnemonicStore {
                source: Box::new(err),
            })
    }

    pub(crate) async fn init_keys(&self) -> Result<(), Error> {
        self.storage
            .init_keys(None)
            .await
            .map_err(|err| Error::KeyStore {
                source: Box::new(err),
            })
    }

    pub(crate) async fn reset_keys(&self, seed: Option<[u8; 32]>) -> Result<(), Error> {
        self.storage
            .reset_keys(seed)
            .await
            .map_err(|err| Error::KeyStore {
                source: Box::new(err),
            })
    }

    pub(crate) async fn load_device_keys(&self) -> Result<Device, Error> {
        self.storage
            .load_keys()
            .await
            .map(|keys| Device::from(keys.device_keypair()))
            .inspect(|device| {
                tracing::debug!("Loading device keys: {}", device.identity_key());
            })
            .map_err(|err| Error::KeyStore {
                source: Box::new(err),
            })
    }

    pub(crate) async fn remove_device_keys(&self) -> Result<(), Error> {
        self.storage
            .remove_keys()
            .await
            .map_err(|err| Error::KeyStore {
                source: Box::new(err),
            })
    }

    pub(crate) async fn handle_storage_op(&self, op: AccountStorageOp) {
        match op {
            AccountStorageOp::GetStoredMnemonic(result_tx) => {
                result_tx.send(self.load_mnemonic().await)
            }
            AccountStorageOp::StoreAccount(result_tx, mnemonic) => {
                if let Ok(_) = self.init_keys().await
                    && let Ok(device) = self.load_device_keys().await
                    && let Ok(_) = self.store_account(mnemonic).await
                {
                    result_tx.send(Ok(device));
                } else {
                    result_tx.send(Err(Error::internal(""))); // SW better error
                }
            }
            AccountStorageOp::ForgetAccount(result_tx) => {
                let account_result = self.remove_account().await;
                if account_result.is_err() {
                    result_tx.send(account_result)
                } else {
                    result_tx.send(self.remove_device_keys().await)
                }
            }
            AccountStorageOp::ResetKeys(result_tx, seed) => {
                if let Ok(_) = self.reset_keys(seed).await
                    && let Ok(device) = self.load_device_keys().await
                {
                    result_tx.send(Ok(device));
                } else {
                    result_tx.send(Err(Error::internal(""))); // SW better error
                }
            }
        }
    }
}

pub(crate) enum AccountStorageOp {
    GetStoredMnemonic(ReturnSender<Mnemonic, Error>), //SW Better error handling here
    StoreAccount(ReturnSender<Device, Error>, Mnemonic), //SW Better error handling here
    ForgetAccount(ReturnSender<(), Error>),           //SW Better error handling here
    ResetKeys(ReturnSender<Device, Error>, Option<[u8; 32]>), //SW Better error handling here
}
