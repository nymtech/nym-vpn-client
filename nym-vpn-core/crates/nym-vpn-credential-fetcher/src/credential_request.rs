// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use nym_bandwidth_controller::NymCredential;
use nym_credential_proxy_requests::api::v1::ticketbook::models::{
    PartialVerificationKeysResponse, TicketbookWalletSharesResponse,
};
use nym_credentials::IssuedTicketBook;
use nym_credentials_interface::{
    Base58, BlindedSignature, PublicKeyUser, RequestInfo, TicketType, VerificationKeyAuth,
    WithdrawalRequest,
};
use nym_ecash_time::EcashTime;
use nym_upgrade_mode_check::try_decode_upgrade_mode_jwt_claims;
use nym_vpn_api_client::{
    VpnApiClient,
    response::{NymVpnZkNym, NymVpnZkNymPost, NymVpnZkNymStatus, StatusOk},
    types::{Device, VpnAccount},
};
use time::{Date, OffsetDateTime};
use tracing::{info, warn};

use crate::{
    VpnApiFetcherError,
    cached_data::CachedData,
    storage::{PendingCredentialRequestsStorage, models::PendingCredentialRequest},
};

pub(crate) type ZkNymId = String;

const ZK_NYM_POLLING_TIMEOUT: Duration = Duration::from_secs(60);
const ZK_NYM_POLLING_INTERVAL: Duration = Duration::from_secs(5);

#[derive(Debug, Clone)]
struct ZkNymRequestData {
    withdrawal_request: WithdrawalRequest,
    ecash_pubkey: PublicKeyUser,
    expiration_date: Date,
    ticketbook_type: TicketType,
    request_info: RequestInfo,
}

pub(crate) struct CredentialRequestTask {
    account: Arc<VpnAccount>,
    device: Device,
    vpn_api_client: VpnApiClient,
    pending_storage: PendingCredentialRequestsStorage,
    cached_data: CachedData,
}

impl CredentialRequestTask {
    pub(crate) fn new(
        account: Arc<VpnAccount>,
        device: Device,
        pending_storage: PendingCredentialRequestsStorage,
        vpn_api_client: VpnApiClient,
        cached_data: CachedData,
    ) -> CredentialRequestTask {
        CredentialRequestTask {
            account,
            device,
            vpn_api_client: vpn_api_client.clone(),
            pending_storage,
            cached_data,
        }
    }

    //---------------------
    // Request flow
    //---------------------

    #[tracing::instrument(skip(self), level = "debug")]
    pub(crate) async fn request_zk_nym_ticketbook(
        &self,
        ticketbook_type: TicketType,
    ) -> Result<NymCredential, VpnApiFetcherError> {
        // Before issuing a new request, see if we already have a pending one of this type that
        // the API has finished preparing, and resume (download) that instead.
        if let Some(pending_request) = self.resumable_request_of_type(ticketbook_type).await? {
            tracing::info!(
                "Resuming pending zk-nym request '{}' for {ticketbook_type}",
                pending_request.id
            );
            return self.finalize_zk_nym_request(pending_request).await;
        }

        let pending_request = self.create_pending_zk_nym_request(ticketbook_type).await?;

        // We have successfully requested the zk-nym ticketbook. Now we need to poll the
        // nym-vpn-api. This is equivalent to resuming an existing request.
        self.finalize_zk_nym_request(pending_request).await
    }

    #[tracing::instrument(skip(self), level = "debug")]
    pub(crate) async fn create_pending_zk_nym_request(
        &self,
        ticketbook_type: TicketType,
    ) -> Result<PendingCredentialRequest, VpnApiFetcherError> {
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
        tracing::info!("Inserting pending zk-nym request: {}", response.id);
        self.pending_storage
            .insert_pending_request(PendingCredentialRequest {
                id: response.id.clone(),
                expiration_date: request.expiration_date,
                request_info: request.request_info.clone(),
            })
            .await?;

        Ok(PendingCredentialRequest {
            id: response.id,
            expiration_date: request.expiration_date,
            request_info: request.request_info,
        })
    }

    #[tracing::instrument(
        skip(self, pending_request),
        fields(id = %pending_request.id, expiration_date = %pending_request.expiration_date),
        level = "debug"
    )]
    pub(super) async fn finalize_zk_nym_request(
        &self,
        pending_request: PendingCredentialRequest,
    ) -> Result<NymCredential, VpnApiFetcherError> {
        let pending_request_id = pending_request.id.clone();
        // Poll the nym-vpn-api for the zk-nym ticketbook to be ready. This could take some time,
        // but likely not more than a few seconds.
        let poll_result = self.poll_zk_nym(&pending_request_id).await?;

        let success = match poll_result.status {
            NymVpnZkNymStatus::Pending => {
                unreachable!("poll_zk_nym would not return an Ok with Pending status")
            }
            NymVpnZkNymStatus::Revoking | NymVpnZkNymStatus::Revoked => {
                return Err(VpnApiFetcherError::ZkNymRevoked);
            }
            NymVpnZkNymStatus::Error => {
                return Err(VpnApiFetcherError::IssuanceError);
            }
            NymVpnZkNymStatus::Active => {
                let credential = self.build_credential(poll_result, pending_request).await?;
                // Once we successfully manage retreive the zk-nym ticketbook,
                // we tell the vpn-api that we have downloaded it.
                if let Err(e) = self.confirm_zk_nym_downloaded(&pending_request_id).await {
                    warn!("Non-fatal error trying to confirm zk_nym download : {e}");
                };
                NymCredential::Ticketbook(Box::new(credential))
            }
            NymVpnZkNymStatus::UpgradeMode => {
                self.process_upgrade_mode_response(poll_result).await?
            }
        };

        // Remove the pending request from the storage. We no longer need it.
        tracing::debug!("Removing pending zk-nym request");
        self.pending_storage
            .remove_pending_request(&pending_request_id)
            .await?;

        Ok(success)
    }

    //---------------------
    // Request flow utils
    //---------------------

    fn construct_zk_nym_request_data(
        &self,
        ticketbook_type: TicketType,
    ) -> Result<ZkNymRequestData, VpnApiFetcherError> {
        tracing::debug!("Constructing zk-nym request");

        let ecash_keypair = self
            .account
            .create_ecash_keypair()
            .map_err(|err| VpnApiFetcherError::CreateEcashKeyPair(err.to_string()))?;
        let expiration_date = nym_ecash_time::ecash_default_expiration_date();

        let (withdrawal_request, request_info) = nym_credentials_interface::withdrawal_request(
            ecash_keypair.secret_key(),
            expiration_date.ecash_unix_timestamp(),
            ticketbook_type.encode(),
        )
        .map_err(VpnApiFetcherError::ConstructWithdrawalRequest)?;

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
    ) -> Result<NymVpnZkNymPost, VpnApiFetcherError> {
        tracing::debug!("Requesting zk-nym ticketbook");
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
            .map_err(VpnApiFetcherError::vpn_api_error("request_zk_nym"))
            .inspect(|response| tracing::info!("Successful zk-nym request: {}", response.id))
    }

    async fn confirm_zk_nym_downloaded(&self, id: &str) -> Result<StatusOk, VpnApiFetcherError> {
        tracing::info!("Confirming zk-nym downloaded");
        self.vpn_api_client
            .confirm_zk_nym_download_by_id(&self.account, &self.device, id)
            .await
            .map_err(VpnApiFetcherError::vpn_api_error(
                "confirm_zk_nym_download_by_id",
            ))
            .inspect(|response| tracing::debug!("Confirmed zk-nym download: {}", response))
    }

    /// Find a locally-pending request of `ticketbook_type` that the API already has ready for
    /// download, if any, so it can be resumed instead of issuing a fresh request. Opportunistically
    /// cleans up stale pending requests.
    async fn resumable_request_of_type(
        &self,
        ticketbook_type: TicketType,
    ) -> Result<Option<PendingCredentialRequest>, VpnApiFetcherError> {
        let pending = self.pending_storage.get_pending_requests().await?;

        if pending.is_empty() {
            // early return to avoid unnecessary network call
            return Ok(None);
        }

        let ticketbook_type = ticketbook_type.to_string();
        let available_ids = self
            .get_zk_nyms_available_for_download()
            .await?
            .into_iter()
            .filter_map(|(id, typ)| (typ == ticketbook_type).then_some(id))
            .collect::<Vec<_>>();

        Ok(pending
            .into_iter()
            .find(|request| available_ids.contains(&request.id)))
    }

    /// Returns the `(id, ticketbook_type)` of every zk-nym the API has ready for us to download.
    async fn get_zk_nyms_available_for_download(
        &self,
    ) -> Result<Vec<(ZkNymId, String)>, VpnApiFetcherError> {
        self.vpn_api_client
            .get_zk_nyms_available_for_download(&self.account, &self.device)
            .await
            .map(|response| {
                response
                    .items
                    .into_iter()
                    .map(|item| (item.id, item.ticketbook_type))
                    .collect()
            })
            .map_err(VpnApiFetcherError::vpn_api_error(
                "get_zk_nyms_available_for_download",
            ))
    }

    async fn poll_zk_nym(&self, id: &str) -> Result<NymVpnZkNym, VpnApiFetcherError> {
        tracing::info!("Starting zk-nym polling task");

        let start_time = Instant::now();
        loop {
            tracing::debug!("Polling zk-nym status");

            let poll_response = self
                .vpn_api_client
                .get_zk_nym_by_id(&self.account, &self.device, id)
                .await
                .map_err(VpnApiFetcherError::vpn_api_error("polling_zk_nym"))?;

            if poll_response.status != NymVpnZkNymStatus::Pending {
                tracing::info!("Polling zk-nym finished");
                tracing::trace!("Polling zk-nym finished: {:#?}", poll_response);
                return Ok(poll_response);
            }

            tracing::info!("Polling zk-nym not finished: {}", poll_response.status);
            tracing::debug!("Polling zk-nym not finished: {:#?}", poll_response);
            if start_time.elapsed() > ZK_NYM_POLLING_TIMEOUT {
                tracing::error!("Polling zk-nym timed out");
                return Err(VpnApiFetcherError::PollingTimeout { id: id.to_string() });
            }

            tracing::trace!("Sleeping for {ZK_NYM_POLLING_INTERVAL:?}");
            tokio::time::sleep(ZK_NYM_POLLING_INTERVAL).await;
        }
    }

    //---------------------
    // Response processing
    //---------------------

    async fn process_upgrade_mode_response(
        &self,
        response: NymVpnZkNym,
    ) -> Result<NymCredential, VpnApiFetcherError> {
        let Some(upgrade_mode_data) = response.upgrade_mode else {
            // unless VPN API is faulty, this shouldn't be possible
            return Err(VpnApiFetcherError::InconsistentResponse(
                "VPN API response with status 'upgrade_mode' did not contain upgrade mode attestation".into(),
            ));
        };

        // ASSUMPTION: we trust our credential-proxy -> VPN API chain to have validated
        // that the attestation had been signed with expected key
        // (because otherwise, attempting to thread in environment-dependant key here would be quite a hassle)

        // decode the JWT to
        // 1. (optional) make sure it's correctly formed
        // 2. retrieve its expiration
        let jwt_payload = try_decode_upgrade_mode_jwt_claims(&upgrade_mode_data.upgrade_mode_jwt)
            .map_err(|_| VpnApiFetcherError::MalformedUpgradeModeJWT)?;

        // if the expiration is not set (it should always be!) set it to unix epoch,
        // i.e. treat it as expired for all intents and purposes
        let expiration = jwt_payload
            .expires_at
            .and_then(|exp| OffsetDateTime::from_unix_timestamp(exp.as_secs() as i64).ok())
            .unwrap_or(OffsetDateTime::UNIX_EPOCH);

        info!("the retrieved upgrade mode JWT is set to expire at {expiration}");

        Ok(NymCredential::UpgradeModeToken {
            jwt: upgrade_mode_data.upgrade_mode_jwt,
            expiration,
        })
    }

    #[tracing::instrument(skip_all)]
    async fn build_credential(
        &self,
        response: NymVpnZkNym,
        pending_request: PendingCredentialRequest,
    ) -> Result<IssuedTicketBook, VpnApiFetcherError> {
        let Some(ref shares) = response.blinded_shares else {
            return Err(VpnApiFetcherError::MissingBlindedShares);
        };
        let epoch_id = shares.epoch_id;
        tracing::debug!("epoch_id: {epoch_id}");

        let master_vk = self
            .cached_data
            .get_master_verification_key(epoch_id)
            .await?;

        let issuers = self
            .cached_data
            .get_partial_verification_keys(epoch_id)
            .await?;

        let ticketbook_type = response.ticketbook_type.parse().map_err(|_| {
            VpnApiFetcherError::InvalidTicketTypeInResponse(response.ticketbook_type)
        })?;

        let ticketbook = self
            .unblind_and_aggregate(
                shares.clone(),
                issuers,
                master_vk,
                ticketbook_type,
                pending_request.expiration_date.ecash_date(),
                &pending_request.request_info,
            )
            .await?;

        Ok(ticketbook)
    }

    async fn unblind_and_aggregate(
        &self,
        shares: TicketbookWalletSharesResponse,
        issuers: PartialVerificationKeysResponse,
        master_vk: VerificationKeyAuth,
        ticketbook_type: TicketType,
        expiration_date: Date,
        request_info: &RequestInfo,
    ) -> Result<IssuedTicketBook, VpnApiFetcherError> {
        tracing::trace!("Unblinding and aggregating zk-nym shares");

        let ecash_keypair = self
            .account
            .create_ecash_keypair()
            .map_err(|err| VpnApiFetcherError::CreateEcashKeyPair(err.to_string()))?;

        tracing::trace!("Setting up decoded keys");
        let mut decoded_keys = HashMap::new();
        for key in issuers.keys {
            let vk = VerificationKeyAuth::try_from_bs58(&key.bs58_encoded_key)
                .inspect_err(|err| {
                    tracing::error!("Failed to create VerificationKeyAuth: {err:#?}")
                })
                .map_err(VpnApiFetcherError::InvalidVerificationKey)?;
            decoded_keys.insert(key.node_index, vk);
        }

        tracing::trace!("Verifying zk-nym shares");
        let mut partial_wallets = Vec::new();
        for share in shares.shares {
            tracing::trace!("Creating blinded signature");
            let blinded_sig = BlindedSignature::try_from_bs58(&share.bs58_encoded_share)
                .inspect_err(|err| tracing::error!("Failed to create BlindedSignature: {err:#?}"))
                .map_err(VpnApiFetcherError::DeserializeBlindedSignature)?;

            let Some(vk) = decoded_keys.get(&share.node_index) else {
                return Err(VpnApiFetcherError::DecodedKeysMissingIndex);
            };

            tracing::trace!("Calling issue_verify");
            match nym_credentials_interface::issue_verify(
                vk,
                ecash_keypair.secret_key(),
                &blinded_sig,
                request_info,
                share.node_index,
            ) {
                Ok(partial_wallet) => {
                    tracing::trace!("Partial wallet created and appended");
                    partial_wallets.push(partial_wallet)
                }
                Err(err) => {
                    tracing::error!("Failed to issue verify: {err:#?}");
                    return Err(VpnApiFetcherError::IssuanceVerification(err));
                }
            }
        }

        tracing::trace!("Aggregating wallets");
        let aggregated_wallets = nym_credentials_interface::aggregate_wallets(
            &master_vk,
            ecash_keypair.secret_key(),
            &partial_wallets,
            request_info,
        )
        .map_err(VpnApiFetcherError::AggregateWallets)?;

        tracing::trace!("Creating ticketbook");
        let ticketbook = IssuedTicketBook::new(
            aggregated_wallets.into_wallet_signatures(),
            shares.epoch_id,
            ecash_keypair.into(),
            ticketbook_type,
            expiration_date,
        );

        Ok(ticketbook)
    }
}

fn verify_response(
    request: &ZkNymRequestData,
    response: &NymVpnZkNymPost,
) -> Result<(), VpnApiFetcherError> {
    tracing::debug!("Verifying zk-nym response");
    let ticketbook_type: TicketType = response.ticketbook_type.parse().map_err(|_| {
        VpnApiFetcherError::InvalidTicketTypeInResponse(response.ticketbook_type.clone())
    })?;
    if ticketbook_type != request.ticketbook_type {
        return Err(VpnApiFetcherError::TicketTypeMismatch);
    }
    Ok(())
}
