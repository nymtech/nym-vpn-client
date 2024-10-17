// Copyright 2024 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_http_api_client::{Client, HttpClientError, NO_PARAMS};
use nym_validator_client::ecash::models::{
    AggregatedCoinIndicesSignatureResponse, AggregatedExpirationDateSignatureResponse,
    MasterVerificationKeyResponse,
};
use url::Url;

pub(crate) type VpnEcashApiClientError = HttpClientError;

pub(crate) struct VpnEcashApiClient {
    inner: Client,
}

impl VpnEcashApiClient {
    pub(crate) fn new(base_url: Url) -> Result<VpnEcashApiClient, VpnEcashApiClientError> {
        Ok(Self {
            inner: Client::builder(base_url)?
                .with_user_agent(format!("ecash-client/{}", env!("CARGO_PKG_VERSION")))
                .build()?,
        })
    }

    pub(crate) async fn get_master_verification_key(
        &self,
    ) -> Result<MasterVerificationKeyResponse, VpnEcashApiClientError> {
        self.inner
            .get_json(&["/v1", "/ecash", "/master-verification-key"], NO_PARAMS)
            .await
    }

    pub(crate) async fn get_aggregated_coin_indices_signatures(
        &self,
    ) -> Result<AggregatedCoinIndicesSignatureResponse, VpnEcashApiClientError> {
        self.inner
            .get_json(
                &["/v1", "/ecash", "/aggregated-coin-indices-signatures"],
                NO_PARAMS,
            )
            .await
    }

    pub(crate) async fn get_aggregated_expiration_data_signatures(
        &self,
    ) -> Result<AggregatedExpirationDateSignatureResponse, VpnEcashApiClientError> {
        self.inner
            .get_json(
                &["/v1", "/ecash", "/aggregated-expiration-date-signatures"],
                NO_PARAMS,
            )
            .await
    }
}
