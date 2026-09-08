// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{collections::HashMap, sync::Arc};

use nym_credential_proxy_requests::api::v1::ticketbook::models::PartialVerificationKeysResponse;
use nym_credentials_interface::{Base58, VerificationKeyAuth};
use nym_vpn_api_client::VpnApiClient;

use crate::VpnApiFetcherError;

// Generic struct to store cached data during the request process, both between concurrent requests
// for different types, and between requests for the same type.
#[derive(Clone)]
pub struct CachedData {
    // Cached partial verification keys, fetched from the API.
    partial_verification_keys:
        Arc<tokio::sync::Mutex<HashMap<u64, PartialVerificationKeysResponse>>>,

    // Cached master verification key, fetched from the API.
    master_verification_key: Arc<tokio::sync::Mutex<HashMap<u64, VerificationKeyAuth>>>,

    // nym-vpn-api client used to fetch new data
    vpn_api_client: VpnApiClient,
}

impl CachedData {
    pub fn new(vpn_api_client: VpnApiClient) -> Self {
        CachedData {
            partial_verification_keys: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
            master_verification_key: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
            vpn_api_client,
        }
    }

    pub async fn get_partial_verification_keys(
        &self,
        epoch_id: u64,
    ) -> Result<PartialVerificationKeysResponse, VpnApiFetcherError> {
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
                .get_directory_zk_nyms_ticketbook_partial_verification_keys(epoch_id)
                .await
                .map_err(VpnApiFetcherError::vpn_api_error(
                    "get_directory_zk_nyms_ticketbook_partial_verification_keys",
                ))?;

            // a vpn-api deployed without the `epoch-id` param ignores it and answers with the
            // current epoch, which is indistinguishable from never having asked
            if issuers.epoch_id != epoch_id {
                return Err(VpnApiFetcherError::EpochIdMismatch);
            }

            partial_verification_keys.insert(epoch_id, issuers.clone());
            Ok(issuers)
        }
    }

    pub async fn get_master_verification_key(
        &self,
        epoch_id: u64,
    ) -> Result<VerificationKeyAuth, VpnApiFetcherError> {
        // Get the partial verification keys for the given epoch if they exist in the cache.
        // Otherwise fetch it from the API, store it and then return it
        let mut master_verification_key = self.master_verification_key.lock().await;
        if let Some(key) = master_verification_key.get(&epoch_id) {
            tracing::debug!("Using cached master verification key for epoch: {epoch_id}");
            Ok(key.clone())
        } else {
            tracing::info!("Fetching master verification keys for epoch: {epoch_id}");

            let response = self
                .vpn_api_client
                .get_directory_zk_nyms_ticketbook_master_verification_key(epoch_id)
                .await
                .map_err(VpnApiFetcherError::vpn_api_error(
                    "get_directory_zk_nyms_ticketbook_master_verification_key",
                ))?;
            let key = VerificationKeyAuth::try_from_bs58(&response.bs58_encoded_key)
                .map_err(VpnApiFetcherError::InvalidVerificationKey)?;

            master_verification_key.insert(epoch_id, key.clone());
            Ok(key)
        }
    }
}
