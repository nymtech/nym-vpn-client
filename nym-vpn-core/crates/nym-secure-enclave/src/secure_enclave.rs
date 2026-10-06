// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

#[cfg(any(target_os = "macos", target_os = "ios"))]
use super::AppleSecureEnclaveProvider;
#[cfg(target_os = "android")]
use super::{AndroidKeyStoreProvider, AndroidSecureEnclaveProvider};
use super::{Result, SecureEnclaveKey, SecureEnclaveProvider};

pub struct SecureEnclave {
    #[cfg(target_os = "android")]
    inner: AndroidKeyStoreProvider,
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    inner: AppleSecureEnclaveProvider,
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

    pub fn create_key(&self, alias: &str) -> Result<Arc<dyn SecureEnclaveKey>> {
        self.inner.create_key(alias)
    }

    pub fn delete_key(&self, alias: &str) -> Result<()> {
        self.inner.delete_key(alias)
    }

    pub fn get_key(&self, alias: &str) -> Result<Arc<dyn SecureEnclaveKey>> {
        self.inner.get_key(alias)
    }
}
