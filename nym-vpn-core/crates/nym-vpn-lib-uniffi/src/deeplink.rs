// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use nym_vpn_store::account::Mnemonic;
use tokio::sync::Mutex;

use nym_vpn_account_controller::{CreateDeeplinkParams, Deeplinks};
use nym_vpn_lib_types::{DeeplinkClient, DeeplinkKind, GetDeeplinkParams};

use crate::{NymEnvironment, error::VpnError};

/// Thread-safe deep link handler.
#[derive(uniffi::Object)]
pub struct NymDeeplinks {
    deep_links: Arc<Mutex<Deeplinks>>,
    network_env: Arc<NymEnvironment>,
}

#[uniffi::export(async_runtime = "tokio")]
impl NymDeeplinks {
    #[uniffi::constructor]
    pub fn new(network_env: Arc<NymEnvironment>) -> Self {
        Self {
            deep_links: Arc::new(Mutex::new(Deeplinks::default())),
            network_env,
        }
    }

    /// Get a deeplink
    pub async fn get_deeplink(&self, params: GetDeeplinkParams) -> Result<String, VpnError> {
        let base_url = match params.kind {
            DeeplinkKind::Privy => {
                let Some(ref account_management) =
                    self.network_env.inner().nym_vpn_network.account_management
                else {
                    return Err(VpnError::DeeplinkError {
                        details: "No account management data is available at this time".to_string(),
                    });
                };

                let opt_url = match params.client {
                    DeeplinkClient::Mobile => account_management.privy_mobile_url(&params.locale),
                    DeeplinkClient::Desktop => account_management.privy_desktop_url(&params.locale),
                    DeeplinkClient::Web => account_management.privy_web_url(&params.locale),
                };

                opt_url.ok_or(VpnError::DeeplinkError {
                    details: "The privy path could not be determined".to_string(),
                })?
            }
        };

        let mut deeplink_guard = self.deep_links.lock().await;
        let params = CreateDeeplinkParams {
            kind: params.kind,
            name: params.name,
            base_url,
        };

        // Create a new Deeplink for this request
        let deeplink =
            deeplink_guard
                .create_deeplink(&params)
                .map_err(|e| VpnError::DeeplinkError {
                    details: e.to_string(),
                })?;

        // Create the deeplink URL
        let url = deeplink.create_url(&params.base_url);

        // Housekeeping
        deeplink_guard.remove_expired();

        Ok(url.to_string())
    }

    /// Derive mnemonic from deeplink callback URL
    pub async fn handle_callback_url(
        &self,
        deeplink_callback_url: String,
    ) -> Result<PrivyMnemonic, VpnError> {
        let mut deeplink_guard = self.deep_links.lock().await;

        // Derive the mnemonic from the provided deeplink URL
        let mnemonic = deeplink_guard
            .derive_mnemonic(&deeplink_callback_url)
            .map_err(|e| VpnError::DeeplinkError {
                details: e.to_string(),
            })?;

        // Housekeeping
        deeplink_guard.remove_expired();

        Ok(PrivyMnemonic { mnemonic })
    }
}

/// Opaque object holding privy mnemonic
#[derive(uniffi::Object)]
pub struct PrivyMnemonic {
    mnemonic: Mnemonic,
}

#[allow(unused)]
impl PrivyMnemonic {
    pub fn inner(&self) -> &Mnemonic {
        &self.mnemonic
    }
}
