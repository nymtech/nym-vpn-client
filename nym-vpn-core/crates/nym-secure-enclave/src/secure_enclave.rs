// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(target_os = "android")]
use std::sync::Arc;

use futures_util::TryFutureExt;
use typed_builder::TypedBuilder;

#[cfg(target_os = "linux")]
use super::LinuxSecureEnclaveProvider;
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
#[cfg(target_os = "linux")]
type PlatformProvider = LinuxSecureEnclaveProvider;

/// Key type returned by [`SecureEnclave`] on the current platform.
pub type PlatformSecureEnclaveKey = <PlatformProvider as SecureEnclaveProvider>::Key;

pub struct SecureEnclave {
    inner: PlatformProvider,
}

/// Configuration of [`SecureEnclave`], built with [`SecureEnclave::builder`].
#[derive(TypedBuilder)]
#[builder(build_method(into = SecureEnclave))]
pub struct SecureEnclaveConfig {
    /// Keystore implemented by the app.
    #[cfg(target_os = "android")]
    provider: Arc<dyn AndroidSecureEnclaveProvider>,

    /// Authentication used with keys requiring user presence.
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    auth: LocalAuthentication,

    /// Keychain access group of the keys, or the default access group of the process if `None`.
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    #[builder(default)]
    keychain_access_group: Option<&'static str>,
}

impl From<SecureEnclaveConfig> for SecureEnclave {
    fn from(
        #[cfg_attr(
            any(target_os = "windows", target_os = "linux"),
            expect(unused_variables)
        )]
        config: SecureEnclaveConfig,
    ) -> Self {
        Self {
            #[cfg(target_os = "android")]
            inner: AndroidKeyStoreProvider::new(config.provider),
            #[cfg(any(target_os = "macos", target_os = "ios"))]
            inner: AppleSecureEnclaveProvider::new(config.auth, config.keychain_access_group),
            #[cfg(target_os = "windows")]
            inner: WindowsSecureEnclaveProvider,
            #[cfg(target_os = "linux")]
            inner: LinuxSecureEnclaveProvider,
        }
    }
}

impl SecureEnclave {
    pub fn builder() -> SecureEnclaveConfigBuilder {
        SecureEnclaveConfig::builder()
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
