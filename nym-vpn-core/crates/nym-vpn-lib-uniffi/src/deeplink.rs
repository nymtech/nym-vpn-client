// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::sync::Arc;

use tokio::sync::Mutex;

use nym_vpn_account_controller::{CreateDeeplinkParams, DeeplinkMnemonic, Deeplinks};
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
        let Some(ref account_management) =
            self.network_env.inner().nym_vpn_network.account_management
        else {
            return Err(VpnError::DeeplinkError {
                details: "No account management data is available at this time".to_owned(),
            });
        };

        let base_url = match params.kind {
            DeeplinkKind::Privy | DeeplinkKind::PrivyLink => {
                let url = match params.client {
                    DeeplinkClient::Mobile => account_management.privy_mobile_url(&params.locale),
                    DeeplinkClient::Desktop => account_management.privy_desktop_url(&params.locale),
                    DeeplinkClient::Web => account_management.privy_web_url(&params.locale),
                };
                url.ok_or(VpnError::DeeplinkError {
                    details: "The privy path could not be determined".to_owned(),
                })?
            }
            DeeplinkKind::CreateAccount => {
                account_management
                    .pricing_url(&params.locale)
                    .ok_or(VpnError::DeeplinkError {
                        details: "The pricing path could not be determined".to_owned(),
                    })?
            }
            _ => {
                return Err(VpnError::DeeplinkError {
                    details: "Invalid deeplink kind".to_owned(),
                });
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
    pub async fn derive_mnemonic(
        &self,
        deeplink_callback_url: String,
    ) -> Result<NymDeeplinkMnemonic, VpnError> {
        let mut deeplink_guard = self.deep_links.lock().await;

        // Derive the mnemonic from the provided deeplink URL
        let deeplink_mnemonic = deeplink_guard
            .derive_mnemonic(&deeplink_callback_url)
            .map_err(|e| VpnError::DeeplinkError {
                details: e.to_string(),
            })?;

        // Housekeeping
        deeplink_guard.remove_expired();

        Ok(NymDeeplinkMnemonic { deeplink_mnemonic })
    }
}

/// Opaque object holding privy mnemonic
#[derive(uniffi::Object)]
pub struct NymDeeplinkMnemonic {
    deeplink_mnemonic: DeeplinkMnemonic,
}

impl NymDeeplinkMnemonic {
    pub fn inner(&self) -> &DeeplinkMnemonic {
        &self.deeplink_mnemonic
    }
}
