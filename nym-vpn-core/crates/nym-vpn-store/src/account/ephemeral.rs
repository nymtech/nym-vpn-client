// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::convert::Infallible;

use tokio::sync::Mutex;

use super::{AccountInformationStorage, StoredAccountInformation};

#[derive(Default)]
pub struct InMemoryAccountStorage {
    state: Mutex<Option<StoredAccountInformation>>,
}

#[async_trait::async_trait]
impl AccountInformationStorage for InMemoryAccountStorage {
    type StorageError = Infallible;

    async fn load_account(&self) -> Result<Option<StoredAccountInformation>, Self::StorageError> {
        Ok(self.state.lock().await.clone())
    }

    async fn store_account(
        &self,
        state: StoredAccountInformation,
    ) -> Result<(), Self::StorageError> {
        *self.state.lock().await = Some(state);
        Ok(())
    }

    async fn remove_account(&self) -> Result<(), Self::StorageError> {
        *self.state.lock().await = None;
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use nym_credentials_interface::KeyPairUser;
    use nym_vpn_lib_types::StoredAccountMode;

    use crate::account::StoredAccountIdentity;

    use super::*;

    fn logged_in() -> StoredAccountInformation {
        StoredAccountInformation {
            account_identity: StoredAccountIdentity {
                id: "juno1cma4czt2jnydvrvz3lrc9jvcmhpjxtds95s3c6".to_owned(),
                mode: StoredAccountMode::Api,
                pub_key: "".to_owned(),
                signature_base64: "".to_owned(),
            },
            ecash_keypair: KeyPairUser::new(),
        }
    }

    #[tokio::test]
    async fn load_defaults_to_none() {
        let storage = InMemoryAccountStorage::default();

        assert_eq!(storage.load_account().await.unwrap(), None);
    }

    #[tokio::test]
    async fn store_and_load() {
        let storage = InMemoryAccountStorage::default();
        let state = logged_in();
        storage.store_account(state.clone()).await.unwrap();

        assert_eq!(storage.load_account().await.unwrap(), Some(state));
    }

    #[tokio::test]
    async fn remove_resets_to_none() {
        let storage = InMemoryAccountStorage::default();
        storage.store_account(logged_in()).await.unwrap();
        storage.remove_account().await.unwrap();

        assert_eq!(storage.load_account().await.unwrap(), None);
    }
}
