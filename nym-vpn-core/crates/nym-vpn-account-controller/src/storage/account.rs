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

    pub(crate) async fn load_mnemonic(&self) -> Result<Option<Mnemonic>, Error> {
        self.storage
            .load_mnemonic()
            .await
            .map_err(|err| Error::MnemonicStore {
                source: Box::new(err),
            })
    }

    pub(crate) async fn load_account(&self) -> Result<Option<VpnApiAccount>, Error> {
        let mnemonic = self.load_mnemonic().await?;
        mnemonic
            .map(VpnApiAccount::try_from)
            .transpose()
            .map_err(Error::internal)
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

    pub(crate) async fn load_device_keys(&self) -> Result<Option<Device>, Error> {
        let maybe_keys = self
            .storage
            .load_keys()
            .await
            .map_err(|err| Error::KeyStore {
                source: Box::new(err),
            })?;
        Ok(maybe_keys
            .map(|keys| Device::from(keys.device_keypair()))
            .inspect(|device| {
                tracing::debug!("Loading device keys: {}", device.identity_key());
            }))
    }

    pub(crate) async fn remove_device_keys(&self) -> Result<(), Error> {
        self.storage
            .remove_keys()
            .await
            .map_err(|err| Error::KeyStore {
                source: Box::new(err),
            })
    }

    async fn init_account(&self, mnemonic: Mnemonic) -> Result<Device, Error> {
        self.init_keys().await?;
        let device = self
            .load_device_keys()
            .await?
            .ok_or(Error::internal("No keys loaded right after initing them"))?;
        self.store_account(mnemonic).await?;
        Ok(device)
    }

    async fn reset_and_load_keys(&self, seed: Option<[u8; 32]>) -> Result<Device, Error> {
        self.reset_keys(seed).await?;
        self.load_device_keys()
            .await?
            .ok_or(Error::internal("No keys loaded right after resetting them"))
    }

    async fn forget_account(&self) -> Result<(), Error> {
        self.remove_account().await?;
        self.remove_device_keys().await
    }

    pub(crate) async fn handle_storage_op(&self, op: AccountStorageOp) {
        match op {
            AccountStorageOp::GetStoredMnemonic(result_tx) => {
                result_tx.send(self.load_mnemonic().await)
            }
            AccountStorageOp::StoreAccount(result_tx, mnemonic) => {
                result_tx.send(self.init_account(mnemonic).await)
            }
            AccountStorageOp::ForgetAccount(result_tx) => {
                result_tx.send(self.forget_account().await)
            }
            AccountStorageOp::ResetKeys(result_tx, seed) => {
                result_tx.send(self.reset_and_load_keys(seed).await)
            }
        }
    }
}

pub(crate) enum AccountStorageOp {
    GetStoredMnemonic(ReturnSender<Option<Mnemonic>, Error>),
    StoreAccount(ReturnSender<Device, Error>, Mnemonic),
    ForgetAccount(ReturnSender<(), Error>),
    ResetKeys(ReturnSender<Device, Error>, Option<[u8; 32]>),
}
