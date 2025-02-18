// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{collections::HashMap, sync::Arc};

use nym_credential_proxy_requests::api::v1::ticketbook::models::PartialVerificationKeysResponse;
use nym_vpn_api_client::VpnApiClient;
use nym_vpn_lib_types::{RequestZkNymError, VpnApiErrorResponse};

// Generic struct to store cached data during the request process, both between concurrent requests
// for different types, and between requests for the same type.
#[derive(Clone)]
pub struct CachedData {
    // Cached data
    partial_verification_keys:
        Arc<tokio::sync::Mutex<HashMap<u64, PartialVerificationKeysResponse>>>,

    // nym-vpn-api client used to fetch new data
    vpn_api_client: VpnApiClient,
}

impl CachedData {
    pub fn new(vpn_api_client: VpnApiClient) -> Self {
        CachedData {
            partial_verification_keys: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
            vpn_api_client,
        }
    }

    pub async fn get_partial_verification_keys(
        &self,
        epoch_id: u64,
    ) -> Result<PartialVerificationKeysResponse, RequestZkNymError> {
        // Get the partial verification keys for the given epoch if they exist in the cache.
        // Otherwise fetch it from the API, store it and then return it
        let mut partial_verification_keys = self.partial_verification_keys.lock().await;
        if let Some(issuers) = partial_verification_keys.get(&epoch_id) {
            tracing::debug!("Using cached partial verification keys for epoch: {epoch_id}");
            Ok(issuers.clone())
        } else {
            tracing::info!("Fetching partial verification keys for epoch: {epoch_id}");
            let issuers = self
                .vpn_api_client
                .get_directory_zk_nyms_ticketbook_partial_verification_keys()
                .await
                .map_err(|err| {
                    VpnApiErrorResponse::try_from(err)
                        .map(|source| {
                            RequestZkNymError::GetPartialVerificationKeysEndpointFailure {
                                response: source,
                                epoch_id,
                            }
                        })
                        .unwrap_or_else(RequestZkNymError::unexpected_response)
                })?;

            if issuers.epoch_id != epoch_id {
                return Err(RequestZkNymError::EpochIdMismatch);
            }

            partial_verification_keys.insert(epoch_id, issuers.clone());
            Ok(issuers)
        }
    }
}
