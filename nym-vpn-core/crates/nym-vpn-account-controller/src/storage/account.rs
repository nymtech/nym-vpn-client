// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{commands::ReturnSender, error::Error};
use nym_vpn_api_client::types::{Device, VpnAccount};
use nym_vpn_lib_types::{StorableAccount, VpnAccountSummary};
use nym_vpn_store::VpnStorage;

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

    pub(crate) async fn load_stored_account(&self) -> Result<Option<StorableAccount>, Error> {
        self.storage
            .load_account()
            .await
            .map_err(|err| Error::AccountStore {
                source: Box::new(err),
            })
    }

    pub(crate) async fn load_vpn_account(&self) -> Result<Option<VpnAccount>, Error> {
        let account = self.load_stored_account().await?;
        account
            .map(VpnAccount::try_from)
            .transpose()
            .map_err(Error::internal)
    }

    pub(crate) async fn remove_account(&self) -> Result<(), Error> {
        self.storage
            .remove_account()
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

    pub(crate) async fn load_account_summary(&self) -> Result<Option<VpnAccountSummary>, Error> {
        self.storage
            .load_summary()
            .await
            .map_err(|err| Error::AccountSummaryStore {
                source: Box::new(err),
            })
    }

    async fn store_account_summary(&self, summary: VpnAccountSummary) -> Result<(), Error> {
        self.storage
            .store_summary(summary)
            .await
            .map_err(|err| Error::AccountSummaryStore {
                source: Box::new(err),
            })
    }

    async fn remove_account_summary(&self) -> Result<(), Error> {
        self.storage
            .remove_summary()
            .await
            .map_err(|err| Error::AccountSummaryStore {
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

    async fn forget_account(&self) -> Result<(), Error> {
        self.remove_account().await?;
        self.remove_device_keys().await?;
        // The cached summary is non-critical; don't fail the whole forget if its removal fails.
        if let Err(err) = self.remove_account_summary().await {
            tracing::warn!("Failed to remove cached account summary on forget: {err}");
        }
        Ok(())
    }

    pub(crate) async fn handle_storage_op(&self, op: AccountStorageOp) {
        match op {
            AccountStorageOp::GetStoredAccount(result_tx) => {
                result_tx.send(self.load_stored_account().await)
            }
            AccountStorageOp::StoreAccount(result_tx, account) => {
                result_tx.send(self.init_account(account).await)
            }
            AccountStorageOp::ForgetAccount(result_tx) => {
                result_tx.send(self.forget_account().await)
            }
            AccountStorageOp::ResetKeys(result_tx, seed) => {
                result_tx.send(self.reset_and_load_keys(seed).await)
            }
            // Summary persistence is a best-effort cache: log failures rather than surfacing them.
            AccountStorageOp::StoreAccountSummary(summary) => {
                if let Err(err) = self.store_account_summary(*summary).await {
                    tracing::warn!("Failed to persist account summary: {err}");
                }
            }
            AccountStorageOp::RemoveAccountSummary => {
                if let Err(err) = self.remove_account_summary().await {
                    tracing::warn!("Failed to remove cached account summary: {err}");
                }
            }
        }
    }
}

pub(crate) enum AccountStorageOp {
    GetStoredAccount(ReturnSender<Option<StorableAccount>, Error>),
    StoreAccount(ReturnSender<Device, Error>, StorableAccount),
    ForgetAccount(ReturnSender<(), Error>),
    ResetKeys(ReturnSender<Device, Error>, Option<[u8; 32]>),
    /// Persist the latest account summary to the cache (best-effort, fire-and-forget).
    StoreAccountSummary(Box<VpnAccountSummary>),
    /// Drop the cached account summary (best-effort, fire-and-forget).
    RemoveAccountSummary,
}
