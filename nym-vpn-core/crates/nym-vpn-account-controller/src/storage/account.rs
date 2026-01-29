// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{commands::ReturnSender, error::Error};
use nym_vpn_api_client::types::{Device, VpnAccount};
use nym_vpn_store::{VpnStorage, account::StorableAccount, types::StoredAccountMode};

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

    pub(crate) async fn store_account(&self, account: StorableAccount) -> Result<(), Error> {
        self.storage
            .store_account(account)
            .await
            .map_err(|err| Error::AccountStore {
                source: Box::new(err),
            })
    }

    pub(crate) async fn load_stored_accounts(&self) -> Result<Vec<StorableAccount>, Error> {
        self.storage
            .load_accounts()
            .await
            .map_err(|err| Error::AccountStore {
                source: Box::new(err),
            })
    }

    pub(crate) async fn load_vpn_accounts(&self) -> Result<Vec<VpnAccount>, Error> {
        let stored_accounts = self.load_stored_accounts().await?;
        stored_accounts
            .into_iter()
            .map(|account| account.try_into())
            .collect::<Result<Vec<_>, _>>()
            .map_err(Error::internal)
    }

    pub(crate) async fn remove_account(
        &self,
        stored_account_mode: Option<StoredAccountMode>,
    ) -> Result<(), Error> {
        self.storage
            .remove_account(stored_account_mode)
            .await
            .map_err(|err| Error::AccountStore {
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

    async fn init_account(&self, account: StorableAccount) -> Result<Device, Error> {
        self.init_keys().await?;
        let device = self
            .load_device_keys()
            .await?
            .ok_or(Error::internal("No keys loaded right after initing them"))?;
        self.store_account(account).await?;
        Ok(device)
    }

    async fn reset_and_load_keys(&self, seed: Option<[u8; 32]>) -> Result<Device, Error> {
        self.reset_keys(seed).await?;
        self.load_device_keys()
            .await?
            .ok_or(Error::internal("No keys loaded right after resetting them"))
    }

    async fn forget_account(
        &self,
        stored_account_mode: Option<StoredAccountMode>,
    ) -> Result<(), Error> {
        self.remove_account(stored_account_mode).await?;
        self.remove_device_keys().await
    }

    pub(crate) async fn handle_storage_op(&self, op: AccountStorageOp) {
        match op {
            AccountStorageOp::GetStoredAccounts(result_tx) => {
                result_tx.send(self.load_stored_accounts().await)
            }
            AccountStorageOp::StoreAccount(result_tx, account) => {
                result_tx.send(self.init_account(account).await)
            }
            AccountStorageOp::ForgetAccount(result_tx, stored_account_mode) => {
                result_tx.send(self.forget_account(stored_account_mode).await)
            }
            AccountStorageOp::ResetKeys(result_tx, seed) => {
                result_tx.send(self.reset_and_load_keys(seed).await)
            }
        }
    }
}

pub(crate) enum AccountStorageOp {
    GetStoredAccounts(ReturnSender<Vec<StorableAccount>, Error>),
    StoreAccount(ReturnSender<Device, Error>, StorableAccount),
    ForgetAccount(ReturnSender<(), Error>, Option<StoredAccountMode>),
    ResetKeys(ReturnSender<Device, Error>, Option<[u8; 32]>),
}
