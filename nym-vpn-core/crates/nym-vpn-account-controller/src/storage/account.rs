// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{commands::ReturnSender, error::Error};
use nym_vpn_api_client::types::{Device, VpnAccount, VpnAccountMode};
use nym_vpn_lib_types::StorableAccount;
use nym_vpn_store::{
    VpnStorage,
    device_state::{StoredAccountIdentity, StoredDeviceState},
    mnemonic::StoredMnemonic,
};

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

    pub async fn store_account(&self, account: StorableAccount) -> Result<(), Error> {
        let device_account = StoredAccountIdentity::new(account.mnemonic.clone(), account.mode)
            .map_err(|err| Error::AccountStore {
                source: Box::new(err),
            })?;

        self.storage
            .store_mnemonic(StoredMnemonic {
                mnemonic: account.mnemonic,
            })
            .await
            .map_err(|err| Error::AccountStore {
                source: Box::new(err),
            })?;

        self.storage
            .store_device_state(StoredDeviceState::LoggedIn {
                account_identity: device_account,
                summary: None,
            })
            .await
            .map_err(|err| Error::AccountStore {
                source: Box::new(err),
            })
    }

    pub async fn load_mnemonic(&self) -> Result<Option<StoredMnemonic>, Error> {
        self.storage
            .load_mnemonic()
            .await
            .map_err(|err| Error::AccountStore {
                source: Box::new(err),
            })
    }

    pub async fn load_device_state(&self) -> Result<StoredDeviceState, Error> {
        self.storage
            .load_device_state()
            .await
            .map_err(|err| Error::AccountStore {
                source: Box::new(err),
            })
    }

    pub async fn load_vpn_account(&self) -> Result<Option<VpnAccount>, Error> {
        let device_state = self.load_device_state().await?;

        Ok(match device_state {
            StoredDeviceState::LoggedIn {
                account_identity: account,
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

    pub async fn remove_account(&self) -> Result<(), Error> {
        self.storage
            .remove_mnemonic()
            .await
            .map_err(|err| Error::AccountStore {
                source: Box::new(err),
            })
    }

    pub async fn init_keys(&self) -> Result<(), Error> {
        self.storage
            .init_keys(None)
            .await
            .map_err(|err| Error::KeyStore {
                source: Box::new(err),
            })
    }

    pub async fn reset_keys(&self, seed: Option<[u8; 32]>) -> Result<(), Error> {
        self.storage
            .reset_keys(seed)
            .await
            .map_err(|err| Error::KeyStore {
                source: Box::new(err),
            })
    }

    pub async fn load_device_keys(&self) -> Result<Option<Device>, Error> {
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

    pub async fn remove_device_keys(&self) -> Result<(), Error> {
        self.storage
            .remove_keys()
            .await
            .map_err(|err| Error::KeyStore {
                source: Box::new(err),
            })
    }

    pub async fn store_device_state(&self, device_state: StoredDeviceState) -> Result<(), Error> {
        self.storage
            .store_device_state(device_state)
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
        self.storage
            .remove_device_state()
            .await
            .map_err(|err| Error::AccountSummaryStore {
                source: Box::new(err),
            })?;
        Ok(())
    }

    async fn load_stored_account(&self) -> Result<Option<StorableAccount>, Error> {
        let device_state = self.load_device_state().await?;

        match device_state {
            StoredDeviceState::LoggedIn {
                account_identity: account,
                summary: _,
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
