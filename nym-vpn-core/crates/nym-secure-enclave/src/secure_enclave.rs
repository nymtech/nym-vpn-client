// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(target_os = "android")]
use std::sync::Arc;

use futures_util::TryFutureExt;

#[cfg(any(target_os = "macos", target_os = "ios"))]
use super::AppleSecureEnclaveProvider;
#[cfg(target_os = "android")]
use super::{AndroidKeyStoreProvider, AndroidSecureEnclaveProvider};
use super::{Result, SecureEnclaveError, SecureEnclaveProvider};

#[cfg(target_os = "android")]
type PlatformProvider = AndroidKeyStoreProvider;
#[cfg(any(target_os = "macos", target_os = "ios"))]
type PlatformProvider = AppleSecureEnclaveProvider;

/// Key type returned by [`SecureEnclave`] on the current platform.
pub type PlatformSecureEnclaveKey = <PlatformProvider as SecureEnclaveProvider>::Key;

pub struct SecureEnclave {
    inner: PlatformProvider,
}

impl SecureEnclave {
    pub fn new(
        #[cfg(target_os = "android")] provider: Arc<dyn AndroidSecureEnclaveProvider>,
    ) -> Self {
        Self {
            #[cfg(target_os = "android")]
            inner: AndroidKeyStoreProvider::new(provider),
            #[cfg(any(target_os = "macos", target_os = "ios"))]
            inner: AppleSecureEnclaveProvider::default(),
        }
    }

    pub async fn create_key(&self, alias: String) -> Result<PlatformSecureEnclaveKey> {
        self.inner.create_key(alias).await
    }

    pub async fn delete_key(&self, alias: String) -> Result<()> {
        self.inner.delete_key(alias).await
    }

    pub async fn get_key(&self, alias: String) -> Result<PlatformSecureEnclaveKey> {
        self.inner.get_key(alias).await
    }

    pub async fn get_or_create_key(&self, alias: String) -> Result<PlatformSecureEnclaveKey> {
        self.get_key(alias.clone())
            .or_else(|err| async {
                if err == SecureEnclaveError::KeyNotFound {
                    self.create_key(alias).await
                } else {
                    Err(err)
                }
            })
            .await
    }
}
