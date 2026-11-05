// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{Error, Result, commands::ReturnSender};
use nym_vpn_api_client::types::{Device, VpnAccount, VpnAccountMode};
use nym_vpn_lib_types::StorableAccount;
use nym_vpn_store::{
    VpnStorage,
    device_state::{StoredAccountIdentity, StoredDeviceState},
    mnemonic::StoredMnemonic,
};
use tokio::sync::mpsc;

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
    pub fn from(storage: S) -> Self {
        Self { storage }
    }

    pub async fn store_account(&self, account: StorableAccount) -> Result<()> {
        let vpn_account =
            VpnAccount::try_from(account.clone()).map_err(|e| Error::InvalidMnemonic(e.into()))?;
        let ecash_keypair = vpn_account
            .create_ecash_keypair()
            .map_err(|err| Error::CreateEcashKeyPair(err.into()))?;

        let account_identity = StoredAccountIdentity::new(account.mnemonic.clone(), account.mode)
            .map_err(|err| Error::AccountStore(Box::new(err)))?;

        self.storage
            .store_mnemonic(StoredMnemonic {
                mnemonic: account.mnemonic,
            })
            .await
            .map_err(|err| Error::AccountStore(Box::new(err)))?;

        self.storage
            .store_device_state(StoredDeviceState::LoggedIn {
                account_identity,
                ecash_keypair,
                summary: None,
            })
            .await
            .map_err(|err| Error::AccountStore(Box::new(err)))
    }

    pub async fn load_mnemonic(&self) -> Result<Option<StoredMnemonic>> {
        self.storage
            .load_mnemonic()
            .await
            .map_err(|err| Error::AccountStore(Box::new(err)))
    }

    pub async fn load_device_state(&self) -> Result<StoredDeviceState> {
        self.storage
            .load_device_state()
            .await
            .map_err(|err| Error::AccountStore(Box::new(err)))
    }

    pub async fn load_vpn_account(&self) -> Result<Option<VpnAccount>> {
        let device_state = self.load_device_state().await?;

        Ok(match device_state {
            StoredDeviceState::LoggedIn {
                account_identity: account,
                ecash_keypair: _,
                summary: _,
            } => self.load_mnemonic().await.and_then(|mnemonic| {
                mnemonic
                    .map(|stored_mnemonic| {
                        VpnAccount::new(
                            stored_mnemonic.mnemonic,
                            VpnAccountMode::from(account.mode),
                        )
                        .map_err(Error::internal)
                    })
                    .transpose()
            })?,
            StoredDeviceState::LoggedOut => None,
        })
    }

    pub async fn remove_account(&self) -> Result<()> {
        self.storage
            .remove_mnemonic()
            .await
            .map_err(|err| Error::AccountStore(Box::new(err)))
    }

    pub async fn init_keys(&self) -> Result<()> {
        self.storage
            .init_keys(None)
            .await
            .map_err(|err| Error::KeyStore(Box::new(err)))
    }

    pub async fn reset_keys(&self, seed: Option<[u8; 32]>) -> Result<()> {
        self.storage
            .reset_keys(seed)
            .await
            .map_err(|err| Error::KeyStore(Box::new(err)))
    }

    pub async fn load_device_keys(&self) -> Result<Option<Device>> {
        let maybe_keys = self
            .storage
            .load_keys()
            .await
            .map_err(|err| Error::KeyStore(Box::new(err)))?;
        Ok(maybe_keys
            .map(|keys| Device::from(keys.device_keypair()))
            .inspect(|device| {
                tracing::debug!("Loading device keys: {}", device.identity_key());
            }))
    }

    pub async fn remove_device_keys(&self) -> Result<()> {
        self.storage
            .remove_keys()
            .await
            .map_err(|err| Error::KeyStore(Box::new(err)))
    }

    pub async fn store_device_state(&self, device_state: StoredDeviceState) -> Result<()> {
        self.storage
            .store_device_state(device_state)
            .await
            .map_err(|err| Error::AccountSummaryStore(Box::new(err)))
    }

    async fn init_account(&self, account: StorableAccount) -> Result<Device> {
        self.init_keys().await?;
        let device = self
            .load_device_keys()
            .await?
            .ok_or(Error::internal("No keys loaded right after initing them"))?;
        self.store_account(account).await?;

        Ok(device)
    }

    async fn reset_and_load_keys(&self, seed: Option<[u8; 32]>) -> Result<Device> {
        self.reset_keys(seed).await?;
        self.load_device_keys()
            .await?
            .ok_or(Error::internal("No keys loaded right after resetting them"))
    }

    async fn forget_account(&self) -> Result<()> {
        self.remove_account().await?;
        self.remove_device_keys().await?;
        self.storage
            .remove_device_state()
            .await
            .map_err(|err| Error::AccountSummaryStore(Box::new(err)))?;
        Ok(())
    }

    async fn load_stored_account(&self) -> Result<Option<StorableAccount>> {
        let device_state = self.load_device_state().await?;

        match device_state {
            StoredDeviceState::LoggedIn {
                account_identity: account,
                summary: _,
                ecash_keypair: _,
            } => self.load_mnemonic().await.map(|stored_mnemonic| {
                stored_mnemonic.map(|v| StorableAccount::new(v.mnemonic, account.mode))
            }),
            StoredDeviceState::LoggedOut => Ok(None),
        }
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
            AccountStorageOp::StoreDeviceState(result_tx, device_state) => {
                result_tx.send(self.store_device_state(device_state).await)
            }
        }
    }
}

pub(crate) enum AccountStorageOp {
    GetStoredAccount(ReturnSender<Option<StorableAccount>, Error>),
    StoreAccount(ReturnSender<Device, Error>, StorableAccount),
    ForgetAccount(ReturnSender<(), Error>),
    ResetKeys(ReturnSender<Device, Error>, Option<[u8; 32]>),
    StoreDeviceState(ReturnSender<(), Error>, StoredDeviceState),
}

/// Retrieves the stored account through the storage task.
///
/// This goes into storage each time to trigger the platform's step-up authentication when secure
/// storage is used.
pub(crate) async fn request_stored_account(
    storage_op_sender: &mpsc::UnboundedSender<AccountStorageOp>,
) -> Result<Option<StorableAccount>> {
    let (tx, rx) = ReturnSender::new();
    storage_op_sender
        .send(AccountStorageOp::GetStoredAccount(tx))
        .map_err(Error::internal)?;
    rx.await.map_err(Error::internal)?
}

/// Materializes the stored account as [`VpnAccount`], which requires step-up authentication.
pub(crate) async fn request_vpn_api_account(
    storage_op_sender: &mpsc::UnboundedSender<AccountStorageOp>,
) -> Result<VpnAccount> {
    request_stored_account(storage_op_sender)
        .await?
        .ok_or(Error::NoAccountStored)
        .and_then(|account| VpnAccount::try_from(account).map_err(Error::internal))
}
