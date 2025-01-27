// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use nym_compact_ecash::{Base58, BlindedSignature, VerificationKeyAuth, WithdrawalRequest};
use nym_credential_proxy_requests::api::v1::ticketbook::models::{
    AggregatedCoinIndicesSignaturesResponse, AggregatedExpirationDateSignaturesResponse,
    MasterVerificationKeyResponse, PartialVerificationKeysResponse, TicketbookWalletSharesResponse,
};
use nym_credentials::{EpochVerificationKey, IssuedTicketBook};
use nym_credentials_interface::{PublicKeyUser, RequestInfo, TicketType};
use nym_ecash_time::EcashTime;
use nym_vpn_api_client::{
    response::{NymVpnZkNym, NymVpnZkNymPost, NymVpnZkNymStatus, StatusOk},
    types::{Device, VpnApiAccount},
    VpnApiClient,
};
use serde::{Deserialize, Serialize};
use time::Date;

use crate::{
    commands::VpnApiEndpointFailure,
    storage::{PendingCredentialRequest, VpnCredentialStorage},
};

use super::{cached_data::CachedData, RequestZkNymError, ZkNymId};

const ZK_NYM_POLLING_TIMEOUT: Duration = Duration::from_secs(60);
const ZK_NYM_POLLING_INTERVAL: Duration = Duration::from_secs(5);

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequestZkNymSuccess {
    pub id: ZkNymId,
}

impl RequestZkNymSuccess {
    pub fn new(id: ZkNymId) -> Self {
        RequestZkNymSuccess { id }
    }
}

pub(super) struct RequestZkNymTask {
    account: VpnApiAccount,
    device: Device,
    vpn_api_client: VpnApiClient,
    credential_storage: Arc<tokio::sync::Mutex<VpnCredentialStorage>>,
    cached_data: CachedData,
}

impl RequestZkNymTask {
    pub(super) fn new(
        account: VpnApiAccount,
        device: Device,
        vpn_api_client: VpnApiClient,
        credential_storage: Arc<tokio::sync::Mutex<VpnCredentialStorage>>,
        cached_data: CachedData,
    ) -> Self {
        RequestZkNymTask {
            account,
            device,
            vpn_api_client,
            credential_storage,
            cached_data,
        }
    }

    #[tracing::instrument(skip(self))]
    pub(super) async fn request_zk_nym_ticketbook(
        &self,
        ticketbook_type: TicketType,
    ) -> Result<RequestZkNymSuccess, RequestZkNymError> {
        // Construct the zk-nym request
        let request = self.construct_zk_nym_request_data(ticketbook_type)?;

        // Send the request to the nym-vpn-api. This starts the process of creating the zk-nym on
        // the vpn api side, where it delegates the actual work to the nym-credential-proxy and
        // then onwards to the Nym network. This call should be quick, but it will be some time
        // until the result is ready, which is why we need to poll for it later on.
        let response = self.send_request_zk_nym(&request).await?;
        verify_response(&request, &response)?;

        // Store the pending request data. We will need it to be able to unblind and aggregate the
        // resulting zk-nym ticketbook later.
        self.insert_pending_request(
            response.id.clone(),
            request.expiration_date,
            request.request_info.clone(),
        )
        .await?;

        // We have successfully requested the zk-nym ticketbook. Now we need to poll the
        // nym-vpn-api. This is equivalent to resuming an existing request.
        self.resume_request_zk_nym_ticketbook(response.id).await
    }

    #[tracing::instrument(skip(self))]
    pub(super) async fn resume_request_zk_nym_ticketbook(
        &self,
        id: ZkNymId,
    ) -> Result<RequestZkNymSuccess, RequestZkNymError> {
        let pending_request = self.get_pending_request(&id).await?;

        // Poll the nym-vpn-api for the zk-nym ticketbook to be ready. This could take some time,
        // but likely not more than a few seconds.
        let poll_result = self.poll_zk_nym(&id).await?;

        // The result might contain attached keys and signatures. If so, import them.
        self.import_attached_keys_and_signatures(&poll_result, pending_request.expiration_date)
            .await?;

        // Import the zk-nym ticketbook itself. This will unblind and aggregate the zk-nym shares
        self.import_zk_nym(poll_result, pending_request).await?;

        // Once we successfully manage to import the zk-nym ticketbook, we tell the vpn-api that we
        // have downloaded it.
        self.confirm_zk_nym_downloaded(&id).await?;

        // Remove the pending request from the storage. We no longer need it.
        self.remove_pending_request(&id).await?;

        Ok(RequestZkNymSuccess { id })
    }

    fn construct_zk_nym_request_data(
        &self,
        ticketbook_type: TicketType,
    ) -> Result<ZkNymRequestData, RequestZkNymError> {
        tracing::info!("Constructing zk-nym request");

        let ecash_keypair = self
            .account
            .create_ecash_keypair()
            .map_err(|err| RequestZkNymError::CreateEcashKeyPair(err.to_string()))?;
        let expiration_date = nym_ecash_time::ecash_default_expiration_date();

        let (withdrawal_request, request_info) = nym_compact_ecash::withdrawal_request(
            ecash_keypair.secret_key(),
            expiration_date.ecash_unix_timestamp(),
            ticketbook_type.encode(),
        )
        .map_err(|err| RequestZkNymError::ConstructWithdrawalRequest(err.to_string()))?;

        let ecash_pubkey = ecash_keypair.public_key();

        Ok(ZkNymRequestData {
            withdrawal_request,
            ecash_pubkey,
            expiration_date,
            ticketbook_type,
            request_info,
        })
    }

    async fn send_request_zk_nym(
        &self,
        request: &ZkNymRequestData,
    ) -> Result<NymVpnZkNymPost, RequestZkNymError> {
        tracing::info!("Requesting zk-nym ticketbook");
        self.vpn_api_client
            .request_zk_nym(
                &self.account,
                &self.device,
                request.withdrawal_request.to_bs58(),
                request.ecash_pubkey.to_base58_string().to_owned(),
                request.expiration_date.to_string(),
                request.ticketbook_type.to_string(),
            )
            .await
            .map_err(|err| {
                VpnApiEndpointFailure::try_from(err)
                    .map(|source| RequestZkNymError::RequestZkNymEndpointFailure {
                        source,
                        ticket_type: request.ticketbook_type.to_string(),
                    })
                    .unwrap_or_else(RequestZkNymError::unexpected_response)
            })
            .inspect(|response| tracing::info!("Successful zk-nym request: {}", response.id))
    }

    async fn insert_pending_request(
        &self,
        id: String,
        expiration_date: Date,
        request_info: RequestInfo,
    ) -> Result<(), RequestZkNymError> {
        tracing::info!("Inserting pending zk-nym request: {id}");
        let pending_request = PendingCredentialRequest {
            id,
            expiration_date,
            request_info,
        };
        self.credential_storage
            .lock()
            .await
            .insert_pending_request(pending_request)
            .await
            .map_err(|err| RequestZkNymError::CredentialStorage(err.to_string()))
    }

    async fn get_pending_request(
        &self,
        id: &str,
    ) -> Result<PendingCredentialRequest, RequestZkNymError> {
        self.credential_storage
            .lock()
            .await
            .get_pending_request_by_id(id)
            .await
            .map_err(|err| RequestZkNymError::CredentialStorage(err.to_string()))?
            .ok_or(RequestZkNymError::MissingPendingRequest(id.to_string()))
    }

    async fn poll_zk_nym(&self, id: &str) -> Result<NymVpnZkNym, RequestZkNymError> {
        tracing::info!("Starting zk-nym polling task");

        let start_time = Instant::now();
        loop {
            tracing::debug!("Polling zk-nym status");
            match self
                .vpn_api_client
                .get_zk_nym_by_id(&self.account, &self.device, id)
                .await
            {
                Ok(poll_response) if poll_response.status != NymVpnZkNymStatus::Pending => {
                    tracing::info!("Polling zk-nym finished");
                    tracing::debug!("Polling zk-nym finished: {:#?}", poll_response);
                    return Ok(poll_response);
                }
                Ok(poll_response) => {
                    tracing::info!("Polling zk-nym not finished: {}", poll_response.status);
                    tracing::debug!("Polling zk-nym not finished: {:#?}", poll_response);
                    if start_time.elapsed() > ZK_NYM_POLLING_TIMEOUT {
                        tracing::error!("Polling zk-nym timed out");
                        return Err(RequestZkNymError::PollingTimeout { id: id.to_string() });
                    }
                }
                Err(error) => {
                    return Err(VpnApiEndpointFailure::try_from(error)
                        .map(|source| RequestZkNymError::PollZkNymEndpointFailure { source })
                        .unwrap_or_else(RequestZkNymError::unexpected_response));
                }
            }

            tracing::trace!("Sleeping for {ZK_NYM_POLLING_INTERVAL:?}");
            tokio::time::sleep(ZK_NYM_POLLING_INTERVAL).await;
        }
    }

    async fn import_attached_master_verification_key(
        &self,
        epoch_id: u64,
        master_verification_key: &MasterVerificationKeyResponse,
    ) -> Result<(), RequestZkNymError> {
        if epoch_id != master_verification_key.epoch_id {
            return Err(RequestZkNymError::EpochIdMismatch);
        }

        let attached_master_vk = VerificationKeyAuth::try_from_bs58(
            &master_verification_key.bs58_encoded_key,
        )
        .map_err(|e| RequestZkNymError::ResponseHasInvalidMasterVerificationKey(e.to_string()))?;

        let attached_epoch_vk = EpochVerificationKey {
            epoch_id,
            key: attached_master_vk.clone(),
        };

        let stored_master_vk = self
            .credential_storage
            .lock()
            .await
            .get_master_verification_key(epoch_id)
            .await
            .map_err(|err| RequestZkNymError::CredentialStorage(err.to_string()))?;

        if stored_master_vk.is_none() {
            tracing::info!("Inserting master verification key for epoch: {epoch_id}",);
            self.credential_storage
                .lock()
                .await
                .insert_master_verification_key(&attached_epoch_vk)
                .await
                .inspect_err(|err| {
                    tracing::error!("Failed to insert master verification key: {:?}", err);
                })
                .map_err(|err| RequestZkNymError::CredentialStorage(err.to_string()))?;
        }
        Ok(())
    }

    async fn import_aggregated_coin_index_signatures(
        &self,
        epoch_id: u64,
        aggregated_coin_index_signatures: &AggregatedCoinIndicesSignaturesResponse,
    ) -> Result<(), RequestZkNymError> {
        if epoch_id != aggregated_coin_index_signatures.signatures.epoch_id {
            return Err(RequestZkNymError::EpochIdMismatch);
        }

        let stored_coin_index_signatures = self
            .credential_storage
            .lock()
            .await
            .get_coin_index_signatures(epoch_id)
            .await
            .map_err(|err| RequestZkNymError::CredentialStorage(err.to_string()))?;

        if stored_coin_index_signatures.is_none() {
            tracing::info!("Inserting coin index signatures for epoch: {epoch_id}",);
            self.credential_storage
                .lock()
                .await
                .insert_coin_index_signatures(&aggregated_coin_index_signatures.signatures)
                .await
                .inspect_err(|err| {
                    tracing::error!("Failed to insert coin index signatures: {:#?}", err);
                })
                .map_err(|err| RequestZkNymError::CredentialStorage(err.to_string()))?;
        }
        Ok(())
    }

    async fn import_aggregated_expiration_date_signatures(
        &self,
        epoch_id: u64,
        expiration_date: Date,
        aggregated_expiration_date_signatures: &AggregatedExpirationDateSignaturesResponse,
    ) -> Result<(), RequestZkNymError> {
        // Consistency checks
        if epoch_id != aggregated_expiration_date_signatures.signatures.epoch_id {
            return Err(RequestZkNymError::EpochIdMismatch);
        }
        if expiration_date
            != aggregated_expiration_date_signatures
                .signatures
                .expiration_date
        {
            return Err(RequestZkNymError::ExpirationDateMismatch);
        }

        let stored_expiration_date_signatures = self
            .credential_storage
            .lock()
            .await
            .get_expiration_date_signatures(expiration_date)
            .await
            .map_err(|err| RequestZkNymError::CredentialStorage(err.to_string()))?;

        if stored_expiration_date_signatures.is_none() {
            tracing::info!(
            "Inserting expiration date signatures for epoch {epoch_id} and date: {expiration_date}"
        );
            self.credential_storage
                .lock()
                .await
                .insert_expiration_date_signatures(
                    &aggregated_expiration_date_signatures.signatures,
                )
                .await
                .inspect_err(|err| {
                    tracing::error!("Failed to insert expiration date signatures: {:#?}", err);
                })
                .map_err(|err| RequestZkNymError::CredentialStorage(err.to_string()))?;
        }
        Ok(())
    }

    async fn import_attached_keys_and_signatures(
        &self,
        response: &NymVpnZkNym,
        expiration_date: Date,
    ) -> Result<(), RequestZkNymError> {
        tracing::info!("Importing attached keys and signatures, if available and needed");

        let Some(ref shares) = response.blinded_shares else {
            return Err(RequestZkNymError::MissingBlindedShares);
        };

        if let Some(ref attached_master_vk) = shares.master_verification_key {
            self.import_attached_master_verification_key(shares.epoch_id, attached_master_vk)
                .await?;
        }

        if let Some(ref aggregated_coin_index_signatures) = shares.aggregated_coin_index_signatures
        {
            self.import_aggregated_coin_index_signatures(
                shares.epoch_id,
                aggregated_coin_index_signatures,
            )
            .await?;
        }

        if let Some(ref aggregated_expiration_date_signatures) =
            shares.aggregated_expiration_date_signatures
        {
            self.import_aggregated_expiration_date_signatures(
                shares.epoch_id,
                expiration_date,
                aggregated_expiration_date_signatures,
            )
            .await?;
        }

        Ok(())
    }

    async fn import_zk_nym(
        &self,
        response: NymVpnZkNym,
        pending_request: PendingCredentialRequest,
    ) -> Result<(), RequestZkNymError> {
        tracing::info!("Importing zk-nym ticketbook");

        let Some(ref shares) = response.blinded_shares else {
            return Err(RequestZkNymError::MissingBlindedShares);
        };
        tracing::debug!("epoch_id: {}", shares.epoch_id);

        let issuers = self
            .cached_data
            .get_partial_verification_keys(shares.epoch_id)
            .await?;

        let master_vk = if let Some(stored_master_vk) = self
            .credential_storage
            .lock()
            .await
            .get_master_verification_key(shares.epoch_id)
            .await
            .map_err(|err| RequestZkNymError::CredentialStorage(err.to_string()))?
        {
            stored_master_vk
        } else {
            tracing::error!("No master verification key in storage");
            // TODO: implement fetching the missing master verification key from nym-vpn-api.
            // As of writing this, that endpoint does not yet exist.
            return Err(RequestZkNymError::NoMasterVerificationKeyInStorage);
        };

        let ticketbook_type = response
            .ticketbook_type
            .parse::<TicketType>()
            .map_err(|err| RequestZkNymError::InvalidTicketTypeInResponse(err.to_string()))?;

        let issued_ticketbook = self
            .unblind_and_aggregate(
                shares.clone(),
                issuers,
                master_vk.clone(),
                ticketbook_type,
                pending_request.expiration_date.ecash_date(),
                &pending_request.request_info,
            )
            .await?;

        // Check that we have the signatures we need to import
        if self
            .credential_storage
            .lock()
            .await
            .get_coin_index_signatures(shares.epoch_id)
            .await
            .map_err(|err| RequestZkNymError::CredentialStorage(err.to_string()))?
            .is_none()
        {
            tracing::error!("No coin index signatures in storage");
            // TODO: implement fetching the missing signatures from nym-vpn-api. As of writing this,
            // that endpoint does not yet exist.
            return Err(RequestZkNymError::NoCoinIndexSignaturesInStorage);
        }

        if self
            .credential_storage
            .lock()
            .await
            .get_expiration_date_signatures(pending_request.expiration_date)
            .await
            .map_err(|err| RequestZkNymError::CredentialStorage(err.to_string()))?
            .is_none()
        {
            tracing::error!("No expiration date signatures in storage");
            // TODO: implement fetching the missing signatures from nym-vpn-api. As of writing this,
            // that endpoint does not yet exist.
            return Err(RequestZkNymError::NoExpirationDateSignaturesInStorage);
        }

        tracing::info!("Inserting issued zk-nym ticketbook");
        self.credential_storage
            .lock()
            .await
            .insert_issued_ticketbook(&issued_ticketbook)
            .await
            .map_err(|err| RequestZkNymError::CredentialStorage(err.to_string()))?;

        Ok(())
    }

    async fn unblind_and_aggregate(
        &self,
        shares: TicketbookWalletSharesResponse,
        issuers: PartialVerificationKeysResponse,
        master_vk: VerificationKeyAuth,
        ticketbook_type: TicketType,
        expiration_date: Date,
        request_info: &RequestInfo,
    ) -> Result<IssuedTicketBook, RequestZkNymError> {
        tracing::info!("Unblinding and aggregating zk-nym shares");

        let ecash_keypair = self
            .account
            .create_ecash_keypair()
            .map_err(|err| RequestZkNymError::CreateEcashKeyPair(err.to_string()))?;

        tracing::debug!("Setting up decoded keys");
        let mut decoded_keys = HashMap::new();
        for key in issuers.keys {
            let vk = VerificationKeyAuth::try_from_bs58(&key.bs58_encoded_key)
                .inspect_err(|err| {
                    tracing::error!("Failed to create VerificationKeyAuth: {:#?}", err)
                })
                .map_err(|err| RequestZkNymError::InvalidVerificationKey(err.to_string()))?;
            decoded_keys.insert(key.node_index, vk);
        }

        tracing::debug!("Verifying zk-nym shares");
        let mut partial_wallets = Vec::new();
        for share in shares.shares {
            tracing::debug!("Creating blinded signature");
            let blinded_sig =
                BlindedSignature::try_from_bs58(&share.bs58_encoded_share).map_err(|err| {
                    tracing::error!("Failed to create BlindedSignature: {:#?}", err);
                    RequestZkNymError::DeserializeBlindedSignature(err.to_string())
                })?;

            let Some(vk) = decoded_keys.get(&share.node_index) else {
                return Err(RequestZkNymError::DecodedKeysMissingIndex);
            };

            tracing::debug!("Calling issue_verify");
            match nym_compact_ecash::issue_verify(
                vk,
                ecash_keypair.secret_key(),
                &blinded_sig,
                request_info,
                share.node_index,
            ) {
                Ok(partial_wallet) => {
                    tracing::debug!("Partial wallet created and appended");
                    partial_wallets.push(partial_wallet)
                }
                Err(err) => {
                    tracing::error!("Failed to issue verify: {:#?}", err);
                    return Err(RequestZkNymError::ImportZkNym {
                        ticket_type: ticketbook_type.to_string(),
                        error: err.to_string(),
                    });
                }
            }
        }

        tracing::debug!("Aggregating wallets");
        let aggregated_wallets = nym_compact_ecash::aggregate_wallets(
            &master_vk,
            ecash_keypair.secret_key(),
            &partial_wallets,
            request_info,
        )
        .map_err(|err| RequestZkNymError::AggregateWallets(err.to_string()))?;

        tracing::debug!("Creating ticketbook");
        let ticketbook = IssuedTicketBook::new(
            aggregated_wallets.into_wallet_signatures(),
            shares.epoch_id,
            ecash_keypair.into(),
            ticketbook_type,
            expiration_date,
        );

        Ok(ticketbook)
    }

    async fn confirm_zk_nym_downloaded(&self, id: &str) -> Result<StatusOk, RequestZkNymError> {
        tracing::info!("Confirming zk-nym downloaded");
        self.vpn_api_client
            .confirm_zk_nym_download_by_id(&self.account, &self.device, id)
            .await
            .map_err(|err| {
                VpnApiEndpointFailure::try_from(err)
                    .map(
                        |source| RequestZkNymError::ConfirmZkNymDownloadEndpointFailure {
                            source,
                            id: id.to_string(),
                        },
                    )
                    .unwrap_or_else(RequestZkNymError::unexpected_response)
            })
            .inspect(|response| tracing::debug!("Confirmed zk-nym download: {}", response))
    }

    async fn remove_pending_request(&self, id: &str) -> Result<(), RequestZkNymError> {
        tracing::info!("Removing pending zk-nym request");
        self.credential_storage
            .lock()
            .await
            .remove_pending_request(id)
            .await
            .map_err(|err| RequestZkNymError::CredentialStorage(err.to_string()))
    }
}

#[derive(Debug, Clone)]
struct ZkNymRequestData {
    withdrawal_request: WithdrawalRequest,
    ecash_pubkey: PublicKeyUser,
    expiration_date: Date,
    ticketbook_type: TicketType,
    request_info: RequestInfo,
}

fn verify_response(
    request: &ZkNymRequestData,
    response: &NymVpnZkNymPost,
) -> Result<(), RequestZkNymError> {
    tracing::debug!("Verifying zk-nym response");
    let ticketbook_type = response
        .ticketbook_type
        .parse::<TicketType>()
        .map_err(|err| RequestZkNymError::InvalidTicketTypeInResponse(err.to_string()))?;
    if ticketbook_type != request.ticketbook_type {
        return Err(RequestZkNymError::TicketTypeMismatch);
    }
    Ok(())
}
