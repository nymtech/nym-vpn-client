// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(target_os = "android")]
use std::sync::Arc;

use futures_util::TryFutureExt;

#[cfg(target_os = "windows")]
use super::WindowsSecureEnclaveProvider;
#[cfg(target_os = "android")]
use super::{AndroidKeyStoreProvider, AndroidSecureEnclaveProvider};
#[cfg(any(target_os = "macos", target_os = "ios"))]
use super::{AppleSecureEnclaveProvider, LocalAuthentication};
use super::{Result, SecureEnclaveError, SecureEnclaveProvider};

#[cfg(target_os = "android")]
type PlatformProvider = AndroidKeyStoreProvider;
#[cfg(any(target_os = "macos", target_os = "ios"))]
type PlatformProvider = AppleSecureEnclaveProvider;
#[cfg(target_os = "windows")]
type PlatformProvider = WindowsSecureEnclaveProvider;

/// Key type returned by [`SecureEnclave`] on the current platform.
pub type PlatformSecureEnclaveKey = <PlatformProvider as SecureEnclaveProvider>::Key;

pub struct SecureEnclave {
    inner: PlatformProvider,
}

#[cfg(target_os = "windows")]
impl Default for SecureEnclave {
    fn default() -> Self {
        Self::new()
    }
}

impl SecureEnclave {
    /// On Apple platforms, keys requiring user presence are used with the authentication of
    /// `auth`, and are stored in `keychain_access_group`, or in the default access group of the
    /// process if `None`.
    pub fn new(
        #[cfg(target_os = "android")] provider: Arc<dyn AndroidSecureEnclaveProvider>,
        #[cfg(any(target_os = "macos", target_os = "ios"))] auth: LocalAuthentication,
        #[cfg(any(target_os = "macos", target_os = "ios"))] keychain_access_group: Option<&str>,
    ) -> Self {
        Self {
            #[cfg(target_os = "android")]
            inner: AndroidKeyStoreProvider::new(provider),
            #[cfg(any(target_os = "macos", target_os = "ios"))]
            inner: AppleSecureEnclaveProvider::new(auth, keychain_access_group),
            #[cfg(target_os = "windows")]
            inner: WindowsSecureEnclaveProvider,
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn create_key(
        &self,
        alias: &str,
        user_presence: bool,
    ) -> Result<PlatformSecureEnclaveKey> {
        self.inner.create_key(alias, user_presence).await
    }

    pub async fn delete_key(&self, alias: &str) -> Result<()> {
        self.inner.delete_key(alias).await
    }

    pub async fn get_key(&self, alias: &str) -> Result<PlatformSecureEnclaveKey> {
        self.inner.get_key(alias).await
    }

    /// Authenticate the user once, so that keys requiring user presence can be used without
    /// prompting until [`Self::lock`] is called.
    #[tracing::instrument(skip_all)]
    pub async fn unlock(&self, reason: &str) -> Result<()> {
        self.inner.unlock(reason).await
    }

    /// Revoke the authentication obtained with [`Self::unlock`].
    pub async fn lock(&self) -> Result<()> {
        self.inner.lock().await
    }

    #[tracing::instrument(skip(self))]
    pub async fn get_or_create_key(
        &self,
        alias: &str,
        user_presence: bool,
    ) -> Result<PlatformSecureEnclaveKey> {
        self.get_key(alias)
            .or_else(|err| async {
                if err == SecureEnclaveError::KeyNotFound {
                    self.create_key(alias, user_presence).await
                } else {
                    Err(err)
                }
            })
            .await
    }
}
