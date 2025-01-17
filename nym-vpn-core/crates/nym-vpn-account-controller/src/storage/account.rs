// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use nym_vpn_api_client::types::{Device, VpnApiAccount};
use nym_vpn_store::{mnemonic::Mnemonic, VpnStorage};

use crate::error::Error;

#[derive(Debug, Clone)]
pub(crate) struct AccountStorage<S>
where
    S: VpnStorage,
{
    storage: Arc<tokio::sync::Mutex<S>>,
}

impl<S> AccountStorage<S>
where
    S: VpnStorage,
{
    pub(crate) fn from(storage: Arc<tokio::sync::Mutex<S>>) -> Self {
        Self { storage }
    }

    pub(crate) async fn store_account(&self, mnemonic: Mnemonic) -> Result<(), Error> {
        self.storage
            .lock()
            .await
            .store_mnemonic(mnemonic)
            .await
            .map_err(|err| Error::MnemonicStore {
                source: Box::new(err),
            })
    }

    pub(crate) async fn load_account(&self) -> Result<VpnApiAccount, Error> {
        self.storage
            .lock()
            .await
            .load_mnemonic()
            .await
            .map(VpnApiAccount::from)
            .map_err(|err| Error::MnemonicStore {
                source: Box::new(err),
            })
    }

    pub(crate) async fn remove_account(&self) -> Result<(), Error> {
        self.storage
            .lock()
            .await
            .remove_mnemonic()
            .await
            .map_err(|err| Error::MnemonicStore {
                source: Box::new(err),
            })
    }

    pub(crate) async fn load_account_id(&self) -> Result<String, Error> {
        self.load_account().await.map(|account| account.id())
    }

    pub(crate) async fn init_keys(&self) -> Result<(), Error> {
        self.storage
            .lock()
            .await
            .init_keys(None)
            .await
            .map_err(|err| Error::KeyStore {
                source: Box::new(err),
            })
    }

    pub(crate) async fn load_device_keys(&self) -> Result<Device, Error> {
        self.storage
            .lock()
            .await
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

    pub(crate) async fn load_device_id(&self) -> Result<String, Error> {
        self.load_device_keys()
            .await
            .map(|device| device.identity_key().to_string())
    }

    pub(crate) async fn remove_device_keys(&self) -> Result<(), Error> {
        self.storage
            .lock()
            .await
            .remove_keys()
            .await
            .map_err(|err| Error::KeyStore {
                source: Box::new(err),
            })
    }
}
