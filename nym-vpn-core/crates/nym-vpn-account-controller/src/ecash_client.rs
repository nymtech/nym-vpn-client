// Copyright 2024 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_credentials::{
    AggregatedCoinIndicesSignatures, AggregatedExpirationDateSignatures, EpochVerificationKey,
};
use nym_http_api_client::{Client, HttpClientError, NO_PARAMS};
use nym_validator_client::ecash::models::{
    AggregatedCoinIndicesSignatureResponse, AggregatedExpirationDateSignatureResponse,
    MasterVerificationKeyResponse,
};
use url::Url;

use crate::error::Error;

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

    async fn _get_master_verification_key(
        &self,
    ) -> Result<MasterVerificationKeyResponse, VpnEcashApiClientError> {
        self.inner
            .get_json(&["/v1", "/ecash", "/master-verification-key"], NO_PARAMS)
            .await
    }

    pub(crate) async fn get_master_verification_key(&self) -> Result<EpochVerificationKey, Error> {
        let master_verification_key = self
            ._get_master_verification_key()
            .await
            .map_err(Error::from)?
            .key;

        // Temporary workaround
        let current_epoch = self
            ._get_aggregated_coin_indices_signatures()
            .await?
            .epoch_id;

        Ok(EpochVerificationKey {
            epoch_id: current_epoch,
            key: master_verification_key,
        })
    }

    pub(crate) async fn _get_aggregated_coin_indices_signatures(
        &self,
    ) -> Result<AggregatedCoinIndicesSignatureResponse, VpnEcashApiClientError> {
        self.inner
            .get_json(
                &["/v1", "/ecash", "/aggregated-coin-indices-signatures"],
                NO_PARAMS,
            )
            .await
    }

    pub(crate) async fn get_aggregated_coin_indices_signatures(
        &self,
    ) -> Result<AggregatedCoinIndicesSignatures, Error> {
        self._get_aggregated_coin_indices_signatures()
            .await
            .map(|response| AggregatedCoinIndicesSignatures {
                epoch_id: response.epoch_id,
                signatures: response.signatures,
            })
            .map_err(Error::from)
    }

    pub(crate) async fn _get_aggregated_expiration_data_signatures(
        &self,
    ) -> Result<AggregatedExpirationDateSignatureResponse, VpnEcashApiClientError> {
        self.inner
            .get_json(
                &["/v1", "/ecash", "/aggregated-expiration-date-signatures"],
                NO_PARAMS,
            )
            .await
    }

    pub(crate) async fn get_aggregated_expiration_data_signatures(
        &self,
    ) -> Result<AggregatedExpirationDateSignatures, Error> {
        self._get_aggregated_expiration_data_signatures()
            .await
            .map(|response| AggregatedExpirationDateSignatures {
                epoch_id: response.epoch_id,
                expiration_date: response.expiration_date,
                signatures: response.signatures,
            })
            .map_err(Error::from)
    }
}
