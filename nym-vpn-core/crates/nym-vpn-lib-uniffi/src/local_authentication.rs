// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Uniffi interface for authenticating the device owner before accessing items requiring user
//! presence, such as the account mnemonic.

use std::sync::{Arc, LazyLock};

use nym_secure_enclave::{LocalAuthentication, Result};

static SHARED_INSTANCE: LazyLock<LocalAuthentication> = LazyLock::new(LocalAuthentication::default);

/// Local authentication shared by `NymAccountController` and `NymVpnAccountStorage`, so that a
/// single `authorize()` grants access to the mnemonic for both of them until `deauthorize()`.
#[derive(uniffi::Object)]
pub struct NymLocalAuthentication {
    inner: LocalAuthentication,
}

#[uniffi::export(async_runtime = "tokio")]
impl NymLocalAuthentication {
    #[uniffi::constructor]
    pub fn shared() -> Arc<Self> {
        Arc::new(Self {
            inner: shared_local_authentication(),
        })
    }

    /// Authenticates the device owner, so that items requiring user presence can be used
    /// without prompting until `deauthorize()` is called. `reason` is shown to the user.
    pub async fn authorize(&self, reason: String) -> Result<()> {
        self.inner.authorize(&reason).await
    }

    /// Revokes the authentication obtained with `authorize()`.
    pub async fn deauthorize(&self) -> Result<()> {
        self.inner.deauthorize().await
    }
}

pub fn shared_local_authentication() -> LocalAuthentication {
    SHARED_INSTANCE.clone()
}
