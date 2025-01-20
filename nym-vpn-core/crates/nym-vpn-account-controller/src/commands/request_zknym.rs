// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    },
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
use tokio::task::JoinSet;

use crate::{
    commands::VpnApiEndpointFailure,
    shared_state::RequestZkNymResult,
    storage::{PendingCredentialRequest, VpnCredentialStorage},
    SharedAccountState,
};

use super::{AccountCommandError, AccountCommandResult};

// The maximum number of zk-nym requests that can fail in a row before we disable background
// refresh
const ZK_NYM_MAX_FAILS: u32 = 10;

const ZK_NYM_POLLING_TIMEOUT: Duration = Duration::from_secs(60);
const ZK_NYM_POLLING_INTERVAL: Duration = Duration::from_secs(5);

#[derive(Clone, Default)]
struct CachedData {
    partial_verification_keys:
        Arc<tokio::sync::Mutex<HashMap<u64, PartialVerificationKeysResponse>>>,
}

impl CachedData {
    async fn get_partial_verification_keys(
        &self,
        epoch_id: u64,
        vpn_api_client: &VpnApiClient,
    ) -> Result<PartialVerificationKeysResponse, RequestZkNymError> {
        // Get the partial verification keys for the given epoch if they exist in the cache.
        // Otherwise fetch it from the API, store it and then return it
        let mut partial_verification_keys = self.partial_verification_keys.lock().await;
        if let Some(issuers) = partial_verification_keys.get(&epoch_id) {
            tracing::debug!("Using cached partial verification keys for epoch: {epoch_id}");
            Ok(issuers.clone())
        } else {
            tracing::info!("Fetching partial verification keys for epoch: {epoch_id}");
            let issuers = vpn_api_client
                .get_directory_zk_nyms_ticketbook_partial_verification_keys()
                .await
                .map_err(|err| {
                    nym_vpn_api_client::response::extract_error_response(&err)
                        .map(
                            |e| RequestZkNymError::GetPartialVerificationKeysEndpointFailure {
                                endpoint_failure: VpnApiEndpointFailure {
                                    message_id: e.message_id.clone(),
                                    message: e.message.clone(),
                                    code_reference_id: e.code_reference_id.clone(),
                                },
                                epoch_id,
                            },
                        )
                        .unwrap_or_else(|| RequestZkNymError::internal(err))
                })?;

            if issuers.epoch_id != epoch_id {
                return Err(RequestZkNymError::EpochIdMismatch);
            }

            partial_verification_keys.insert(epoch_id, issuers.clone());
            Ok(issuers)
        }
    }
}

pub(crate) struct WaitingRequestZkNymCommandHandler {
    credential_storage: VpnCredentialStorage,
    account_state: SharedAccountState,
    vpn_api_client: VpnApiClient,
    zk_nym_fails_in_a_row: Arc<AtomicU32>,

    // Cache some of the data used to import zk-nyms between requests, to speed things up. Consider
    // persisting this to storage
    cached_data: CachedData,
}

impl WaitingRequestZkNymCommandHandler {
    pub(crate) fn new(
        credential_storage: VpnCredentialStorage,
        account_state: SharedAccountState,
        vpn_api_client: nym_vpn_api_client::VpnApiClient,
    ) -> Self {
        WaitingRequestZkNymCommandHandler {
            credential_storage,
            account_state,
            vpn_api_client,
            zk_nym_fails_in_a_row: Default::default(),
            cached_data: Default::default(),
        }
    }

    pub(crate) fn build(
        &self,
        account: VpnApiAccount,
        device: Device,
    ) -> RequestZkNymCommandHandler {
        let id = uuid::Uuid::new_v4();
        tracing::debug!("Created new zk-nym request command handler: {}", id);
        RequestZkNymCommandHandler {
            id,
            account,
            device,
            credential_storage: self.credential_storage.clone(),
            account_state: self.account_state.clone(),
            vpn_api_client: self.vpn_api_client.clone(),
            zk_nym_fails_in_a_row: self.zk_nym_fails_in_a_row.clone(),
            cached_data: self.cached_data.clone(),
        }
    }

    pub(crate) fn reset(&self) {
        self.zk_nym_fails_in_a_row.store(0, Ordering::Relaxed);
    }

    pub(crate) async fn max_fails_reached(&self) -> bool {
        self.zk_nym_fails_in_a_row.load(Ordering::Relaxed) >= ZK_NYM_MAX_FAILS
    }
}

pub(crate) struct RequestZkNymCommandHandler {
    id: uuid::Uuid,
    account: VpnApiAccount,
    device: Device,
    credential_storage: VpnCredentialStorage,
    account_state: SharedAccountState,
    vpn_api_client: VpnApiClient,

    zk_nym_fails_in_a_row: Arc<AtomicU32>,
    cached_data: CachedData,
}

impl RequestZkNymCommandHandler {
    fn id_str(&self) -> String {
        format!("{:.8}", self.id.to_string())
    }

    pub(crate) async fn run(self) -> AccountCommandResult {
        AccountCommandResult::RequestZkNym(self.request_zk_nyms_outer().await)
    }

    #[tracing::instrument(
        skip(self),
        fields(id = %self.id_str()),
        ret,
        err,
    )]
    async fn request_zk_nyms_outer(self) -> Result<RequestZkNymSummary, AccountCommandError> {
        tracing::debug!("Running zk-nym request command handler: {}", self.id);

        // Defensive check for something that should not be possible
        if self.account_state.is_zk_nym_request_in_progress().await {
            return Err(AccountCommandError::internal(
                "duplicate zk-nym request command",
            ));
        }

        self.account_state
            .set_zk_nym_request(RequestZkNymResult::InProgress)
            .await;

        match self.request_zk_nyms().await {
            Ok(success) => {
                self.account_state
                    .set_zk_nym_request(RequestZkNymResult::from(success.clone()))
                    .await;
                Ok(success)
            }
            Err(err) => {
                self.account_state
                    .set_zk_nym_request(RequestZkNymResult::from(err.clone()))
                    .await;
                Err(AccountCommandError::from(err))
            }
        }
    }

    #[tracing::instrument(
        skip(self),
        fields(id = %self.id_str()),
        ret,
        err,
    )]
    async fn request_zk_nyms(&self) -> Result<RequestZkNymSummary, RequestZkNymError> {
        tracing::debug!("Running zk-nym request command handler: {}", self.id);

        // If we have pending zk-nym ticketbooks, try those first
        let resumed_requests = self.resume_request_zk_nyms().await;

        let ticket_types = self.check_ticket_types_running_low().await?;
        tracing::debug!("Ticket types running low: {:?}", ticket_types);

        let new_requests = self.request_zk_nyms_for_ticket_types(ticket_types).await;

        let zk_nym_fails_in_a_row = self.zk_nym_fails_in_a_row.load(Ordering::Relaxed);
        if zk_nym_fails_in_a_row > 0 {
            tracing::warn!("We have reached {zk_nym_fails_in_a_row} zk-nym fails in a row",);
        }

        let result = resumed_requests
            .into_iter()
            .chain(new_requests.into_iter())
            .collect();

        Ok(result)
    }

    async fn check_ticket_types_running_low(&self) -> Result<Vec<TicketType>, RequestZkNymError> {
        self.credential_storage
            .check_ticket_types_running_low()
            .await
            .map_err(RequestZkNymError::internal)
    }

    async fn request_zk_nyms_for_ticket_types(
        &self,
        ticket_types: Vec<TicketType>,
    ) -> Vec<Result<RequestZkNymSuccess, RequestZkNymError>> {
        tracing::info!("Requesting zk-nym ticketbooks for: {:?}", ticket_types);

        let account = self.account.clone();
        let device = self.device.clone();
        let vpn_api_client = self.vpn_api_client.clone();
        let credential_storage = self.credential_storage.clone();
        let cached_data = self.cached_data.clone();

        let mut join_set = JoinSet::new();
        for ticket_type in ticket_types {
            join_set.spawn(request_zk_nym(
                ticket_type,
                account.clone(),
                device.clone(),
                vpn_api_client.clone(),
                credential_storage.clone(),
                cached_data.clone(),
            ));
        }
        wait_for_join_set(join_set).await
    }

    async fn resume_request_zk_nyms(&self) -> Vec<Result<RequestZkNymSuccess, RequestZkNymError>> {
        let to_resume = self
            .check_zk_nyms_possible_to_resume()
            .await
            .inspect_err(|err| {
                tracing::error!("Failed to check zk-nyms possible to resume: {:?}", err);
            })
            .unwrap_or_default();
        self.resume_request_zk_nyms_for_ids(to_resume).await
    }

    async fn check_zk_nyms_possible_to_resume(&self) -> Result<Vec<ZkNymId>, RequestZkNymError> {
        let zk_nyms_available_for_download = self.get_zk_nyms_available_for_download().await?;

        self.credential_storage
            .clean_up_stale_requests()
            .await
            .inspect_err(|err| {
                tracing::error!("Failed to clean up stale requests: {:?}", err);
            })
            .ok();

        let pending_requests_data = self
            .credential_storage
            .get_pending_request_ids()
            .await
            .map_err(RequestZkNymError::internal)?;

        let zk_nyms_possible_to_resume = zk_nyms_available_for_download
            .into_iter()
            .filter(|zk_nym| pending_requests_data.contains(zk_nym))
            .collect();

        Ok(zk_nyms_possible_to_resume)
    }

    async fn get_zk_nyms_available_for_download(&self) -> Result<Vec<ZkNymId>, RequestZkNymError> {
        self.vpn_api_client
            .get_zk_nyms_available_for_download(&self.account, &self.device)
            .await
            .map(|response| response.items.into_iter().map(|item| item.id).collect())
            .map_err(|err| {
                nym_vpn_api_client::response::extract_error_response(&err)
                    .map(
                        |e| RequestZkNymError::GetZkNymsAvailableForDownloadEndpointFailure {
                            endpoint_failure: VpnApiEndpointFailure {
                                message_id: e.message_id.clone(),
                                message: e.message.clone(),
                                code_reference_id: e.code_reference_id.clone(),
                            },
                        },
                    )
                    .unwrap_or_else(|| RequestZkNymError::internal(err))
            })
    }

    async fn resume_request_zk_nyms_for_ids(
        &self,
        pending_requests: Vec<ZkNymId>,
    ) -> Vec<Result<RequestZkNymSuccess, RequestZkNymError>> {
        if pending_requests.is_empty() {
            return Vec::new();
        }
        tracing::info!("Resuming {} zk-nym requests", pending_requests.len());

        let account = self.account.clone();
        let device = self.device.clone();
        let vpn_api_client = self.vpn_api_client.clone();
        let credential_storage = self.credential_storage.clone();
        let cached_data = self.cached_data.clone();

        let mut join_set = JoinSet::new();
        for pending_request in pending_requests {
            join_set.spawn(resume_request_zk_nym(
                pending_request,
                account.clone(),
                device.clone(),
                vpn_api_client.clone(),
                credential_storage.clone(),
                cached_data.clone(),
            ));
        }
        wait_for_join_set(join_set).await
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ZkNymRequestData {
    withdrawal_request: WithdrawalRequest,
    ecash_pubkey: PublicKeyUser,
    pub(crate) expiration_date: Date,
    ticketbook_type: TicketType,
    request_info: RequestInfo,
}

#[tracing::instrument(skip(account, device, vpn_api_client, credential_storage, cached_data))]
async fn request_zk_nym(
    ticketbook_type: TicketType,
    account: VpnApiAccount,
    device: Device,
    vpn_api_client: VpnApiClient,
    credential_storage: VpnCredentialStorage,
    cached_data: CachedData,
) -> Result<RequestZkNymSuccess, RequestZkNymError> {
    let request = construct_zk_nym_request_data(&account, ticketbook_type)?;

    let response = send_request_zk_nym(&request, &account, &device, &vpn_api_client).await?;
    verify_response(&request, &response)?;

    insert_pending_request(
        response.id.clone(),
        request.expiration_date,
        request.request_info.clone(),
        &credential_storage,
    )
    .await?;

    resume_request_zk_nym(
        response.id,
        account,
        device,
        vpn_api_client,
        credential_storage,
        cached_data,
    )
    .await
}

#[tracing::instrument(skip(account, device, vpn_api_client, credential_storage, cached_data))]
async fn resume_request_zk_nym(
    id: ZkNymId,
    account: VpnApiAccount,
    device: Device,
    vpn_api_client: VpnApiClient,
    credential_storage: VpnCredentialStorage,
    cached_data: CachedData,
) -> Result<RequestZkNymSuccess, RequestZkNymError> {
    let pending_request = credential_storage
        .get_pending_request_by_id(&id)
        .await
        .map_err(|err| RequestZkNymError::CredentialStorage(err.to_string()))?
        .ok_or(RequestZkNymError::MissingPendingRequest(id.clone()))?;

    let poll_result = poll_zk_nym(&id, &account, &device, &vpn_api_client).await?;

    import_attached_keys_and_signatures(
        &poll_result,
        pending_request.expiration_date,
        &credential_storage,
    )
    .await?;

    import_zk_nym(
        poll_result,
        pending_request,
        &account,
        &credential_storage,
        &cached_data,
        &vpn_api_client,
    )
    .await?;

    confirm_zk_nym_downloaded(&id, &account, &device, &vpn_api_client).await?;

    tracing::info!("Removing pending zk-nym request");
    credential_storage
        .remove_pending_request(&id)
        .await
        .map_err(|err| RequestZkNymError::CredentialStorage(err.to_string()))?;

    Ok(RequestZkNymSuccess { id })
}

fn construct_zk_nym_request_data(
    account: &VpnApiAccount,
    ticketbook_type: TicketType,
) -> Result<ZkNymRequestData, RequestZkNymError> {
    tracing::info!("Constructing zk-nym request");

    let ecash_keypair = account
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
    request: &ZkNymRequestData,
    account: &VpnApiAccount,
    device: &Device,
    vpn_api_client: &nym_vpn_api_client::VpnApiClient,
) -> Result<NymVpnZkNymPost, RequestZkNymError> {
    tracing::info!("Requesting zk-nym ticketbook");
    vpn_api_client
        .request_zk_nym(
            account,
            device,
            request.withdrawal_request.to_bs58(),
            request.ecash_pubkey.to_base58_string().to_owned(),
            request.expiration_date.to_string(),
            request.ticketbook_type.to_string(),
        )
        .await
        .map_err(|err| {
            nym_vpn_api_client::response::extract_error_response(&err)
                .map(|e| RequestZkNymError::RequestZkNymEndpointFailure {
                    endpoint_failure: VpnApiEndpointFailure {
                        message_id: e.message_id.clone(),
                        message: e.message.clone(),
                        code_reference_id: e.code_reference_id.clone(),
                    },
                    ticket_type: request.ticketbook_type.to_string(),
                })
                .unwrap_or_else(|| RequestZkNymError::internal(err))
        })
        .inspect(|response| tracing::info!("Successful zk-nym request: {}", response.id))
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

async fn insert_pending_request(
    id: String,
    expiration_date: Date,
    request_info: RequestInfo,
    credential_storage: &VpnCredentialStorage,
) -> Result<(), RequestZkNymError> {
    tracing::info!("Inserting pending zk-nym request: {id}");
    let pending_request = PendingCredentialRequest {
        id,
        expiration_date,
        request_info,
    };
    credential_storage
        .insert_pending_request(pending_request)
        .await
        .map_err(|err| RequestZkNymError::CredentialStorage(err.to_string()))
}

async fn poll_zk_nym(
    id: &str,
    account: &VpnApiAccount,
    device: &Device,
    api_client: &nym_vpn_api_client::VpnApiClient,
) -> Result<NymVpnZkNym, RequestZkNymError> {
    tracing::info!("Starting zk-nym polling task");

    let start_time = Instant::now();
    loop {
        tokio::time::sleep(ZK_NYM_POLLING_INTERVAL).await;

        tracing::debug!("Polling zk-nym status");
        match api_client.get_zk_nym_by_id(account, device, id).await {
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
                return Err(nym_vpn_api_client::response::extract_error_response(&error)
                    .map(|e| {
                        tracing::warn!(
                        "nym-vpn-api reports: message={}, message_id={:?}, code_reference_id={:?}",
                        e.message,
                        e.message_id,
                        e.code_reference_id,
                    );
                        RequestZkNymError::PollZkNymEndpointFailure {
                            endpoint_failure: VpnApiEndpointFailure {
                                message_id: e.message_id.clone(),
                                message: e.message.clone(),
                                code_reference_id: e.code_reference_id.clone(),
                            },
                            // TODO: remove this field
                            ticket_type: "".to_string(),
                        }
                    })
                    .unwrap_or_else(|| RequestZkNymError::internal(error)));
            }
        }
    }
}

async fn import_attached_master_verification_key(
    epoch_id: u64,
    master_verification_key: &MasterVerificationKeyResponse,
    credential_storage: &VpnCredentialStorage,
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

    let stored_master_vk = credential_storage
        .get_master_verification_key(epoch_id)
        .await
        .map_err(|err| RequestZkNymError::CredentialStorage(err.to_string()))?;

    if stored_master_vk.is_none() {
        tracing::info!("Inserting master verification key for epoch: {epoch_id}",);
        credential_storage
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
    epoch_id: u64,
    aggregated_coin_index_signatures: &AggregatedCoinIndicesSignaturesResponse,
    credential_storage: &VpnCredentialStorage,
) -> Result<(), RequestZkNymError> {
    if epoch_id != aggregated_coin_index_signatures.signatures.epoch_id {
        return Err(RequestZkNymError::EpochIdMismatch);
    }

    let stored_coin_index_signatures = credential_storage
        .get_coin_index_signatures(epoch_id)
        .await
        .map_err(|err| RequestZkNymError::CredentialStorage(err.to_string()))?;

    if stored_coin_index_signatures.is_none() {
        tracing::info!("Inserting coin index signatures for epoch: {epoch_id}",);
        credential_storage
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
    epoch_id: u64,
    expiration_date: Date,
    aggregated_expiration_date_signatures: &AggregatedExpirationDateSignaturesResponse,
    credential_storage: &VpnCredentialStorage,
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

    let stored_expiration_date_signatures = credential_storage
        .get_expiration_date_signatures(expiration_date)
        .await
        .map_err(|err| RequestZkNymError::CredentialStorage(err.to_string()))?;

    if stored_expiration_date_signatures.is_none() {
        tracing::info!(
            "Inserting expiration date signatures for epoch {epoch_id} and date: {expiration_date}"
        );
        credential_storage
            .insert_expiration_date_signatures(&aggregated_expiration_date_signatures.signatures)
            .await
            .inspect_err(|err| {
                tracing::error!("Failed to insert expiration date signatures: {:#?}", err);
            })
            .map_err(|err| RequestZkNymError::CredentialStorage(err.to_string()))?;
    }
    Ok(())
}

// If the response contains attached keys and signatures, import them if we don't already
// have them
async fn import_attached_keys_and_signatures(
    response: &NymVpnZkNym,
    expiration_date: Date,
    credential_storage: &VpnCredentialStorage,
) -> Result<(), RequestZkNymError> {
    tracing::info!("Importing attached keys and signatures, if available and needed");

    let Some(ref shares) = response.blinded_shares else {
        return Err(RequestZkNymError::MissingBlindedShares);
    };

    if let Some(ref attached_master_vk) = shares.master_verification_key {
        import_attached_master_verification_key(
            shares.epoch_id,
            attached_master_vk,
            credential_storage,
        )
        .await?;
    }

    if let Some(ref aggregated_coin_index_signatures) = shares.aggregated_coin_index_signatures {
        import_aggregated_coin_index_signatures(
            shares.epoch_id,
            aggregated_coin_index_signatures,
            credential_storage,
        )
        .await?;
    }

    if let Some(ref aggregated_expiration_date_signatures) =
        shares.aggregated_expiration_date_signatures
    {
        import_aggregated_expiration_date_signatures(
            shares.epoch_id,
            expiration_date,
            aggregated_expiration_date_signatures,
            credential_storage,
        )
        .await?;
    }

    Ok(())
}

async fn import_zk_nym(
    response: NymVpnZkNym,
    pending_request: PendingCredentialRequest,
    account: &VpnApiAccount,
    credential_storage: &VpnCredentialStorage,
    cached_data: &CachedData,
    vpn_api_client: &VpnApiClient,
) -> Result<(), RequestZkNymError> {
    tracing::info!("Importing zk-nym ticketbook");

    let Some(ref shares) = response.blinded_shares else {
        return Err(RequestZkNymError::MissingBlindedShares);
    };
    tracing::debug!("epoch_id: {}", shares.epoch_id);

    let issuers = cached_data
        .get_partial_verification_keys(shares.epoch_id, vpn_api_client)
        .await?;

    let master_vk = if let Some(stored_master_vk) = credential_storage
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

    let issued_ticketbook = unblind_and_aggregate(
        shares.clone(),
        issuers,
        master_vk.clone(),
        ticketbook_type,
        pending_request.expiration_date.ecash_date(),
        &pending_request.request_info,
        account.clone(),
    )
    .await?;

    // Check that we have the signatures we need to import
    if credential_storage
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

    if credential_storage
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
    credential_storage
        .insert_issued_ticketbook(&issued_ticketbook)
        .await
        .map_err(|err| RequestZkNymError::CredentialStorage(err.to_string()))?;

    Ok(())
}

async fn unblind_and_aggregate(
    shares: TicketbookWalletSharesResponse,
    issuers: PartialVerificationKeysResponse,
    master_vk: VerificationKeyAuth,
    ticketbook_type: TicketType,
    expiration_date: Date,
    request_info: &RequestInfo,
    account: VpnApiAccount,
) -> Result<IssuedTicketBook, RequestZkNymError> {
    tracing::info!("Unblinding and aggregating zk-nym shares");

    let ecash_keypair = account
        .create_ecash_keypair()
        .map_err(|err| RequestZkNymError::CreateEcashKeyPair(err.to_string()))?;

    tracing::debug!("Setting up decoded keys");
    let mut decoded_keys = HashMap::new();
    for key in issuers.keys {
        let vk = VerificationKeyAuth::try_from_bs58(&key.bs58_encoded_key)
            .inspect_err(|err| tracing::error!("Failed to create VerificationKeyAuth: {:#?}", err))
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

async fn confirm_zk_nym_downloaded(
    id: &str,
    account: &VpnApiAccount,
    device: &Device,
    vpn_api_client: &VpnApiClient,
) -> Result<StatusOk, RequestZkNymError> {
    tracing::info!("Confirming zk-nym downloaded");
    vpn_api_client
        .confirm_zk_nym_download_by_id(account, device, id)
        .await
        .map_err(|err| {
            nym_vpn_api_client::response::extract_error_response(&err)
                .map(|e| RequestZkNymError::ConfirmZkNymDownloadEndpointFailure {
                    endpoint_failure: VpnApiEndpointFailure {
                        message_id: e.message_id.clone(),
                        message: e.message.clone(),
                        code_reference_id: e.code_reference_id.clone(),
                    },
                    id: id.to_string(),
                })
                .unwrap_or_else(|| RequestZkNymError::internal(err))
        })
        .inspect(|response| tracing::debug!("Confirmed zk-nym download: {}", response))
}

async fn wait_for_join_set(
    mut join_set: JoinSet<Result<RequestZkNymSuccess, RequestZkNymError>>,
) -> Vec<Result<RequestZkNymSuccess, RequestZkNymError>> {
    let mut partial_results = Vec::new();
    loop {
        tokio::select! {
            _ = tokio::time::sleep(Duration::from_secs(5 * 60)) => {
                tracing::warn!("Request zk-nym timed out");
                break;
            }
            result = join_set.join_next() => match result {
                Some(Ok(result)) => {
                    partial_results.push(result);
                }
                Some(Err(err)) => {
                    tracing::error!("Failed to wait for task: {:?}", err);
                }
                None => {
                    tracing::debug!("All zk-nym requests finished");
                    break;
                }
            }
        }
    }
    partial_results
}

pub(crate) type ZkNymId = String;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequestZkNymSuccess {
    pub id: ZkNymId,
}

impl RequestZkNymSuccess {
    pub fn new(id: ZkNymId) -> Self {
        RequestZkNymSuccess { id }
    }
}

#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq, Serialize, Deserialize)]
pub enum RequestZkNymError {
    #[error("failed to get zk-nyms available for download")]
    GetZkNymsAvailableForDownloadEndpointFailure {
        endpoint_failure: VpnApiEndpointFailure,
    },

    #[error("failed to create ecash keypair: {0}")]
    CreateEcashKeyPair(String),

    #[error("failed to construct withdrawal request: {0}")]
    ConstructWithdrawalRequest(String),

    #[error("failed to request zknym endpoint for ticket type: {ticket_type}")]
    RequestZkNymEndpointFailure {
        endpoint_failure: VpnApiEndpointFailure,
        ticket_type: String,
    },

    #[error("response contains invalid ticketbook type: {0}")]
    InvalidTicketTypeInResponse(String),

    #[error("ticket type mismatch")]
    TicketTypeMismatch,

    #[error("error polling for zknym result for ticket type: {ticket_type}")]
    PollZkNymEndpointFailure {
        endpoint_failure: VpnApiEndpointFailure,
        ticket_type: String,
    },

    #[error("polling task failed")]
    PollingTaskError,

    #[error("timeout polling for zknym {id}")]
    PollingTimeout { id: ZkNymId },

    #[error("polling for zknym {id} finished with error for ticket type: {ticket_type}")]
    FinishedWithError {
        id: ZkNymId,
        ticket_type: String,
        status: NymVpnZkNymStatus,
    },

    #[error("response is missing blinded shares")]
    MissingBlindedShares,

    #[error("response contains invalid master verification key: {0}")]
    ResponseHasInvalidMasterVerificationKey(String),

    #[error("epoch id mismatch")]
    EpochIdMismatch,

    #[error("expiration date mismatch")]
    ExpirationDateMismatch,

    #[error("failed to request partial verification keys for epoch {epoch_id}")]
    GetPartialVerificationKeysEndpointFailure {
        endpoint_failure: VpnApiEndpointFailure,
        epoch_id: u64,
    },

    #[error("no master verification key in storage")]
    NoMasterVerificationKeyInStorage,

    #[error("no coin index signatures in storage")]
    NoCoinIndexSignaturesInStorage,

    #[error("no expiration date signatures in storage")]
    NoExpirationDateSignaturesInStorage,

    #[error("invalid verification key: {0}")]
    InvalidVerificationKey(String),

    #[error("failed to deserialize blinded signature: {0}")]
    DeserializeBlindedSignature(String),

    #[error("decoded keys missing index")]
    DecodedKeysMissingIndex,

    #[error("failed to import zknym")]
    ImportZkNym { ticket_type: String, error: String },

    #[error("failed to aggregate wallets: {0}")]
    AggregateWallets(String),

    #[error("failed to confirm zknym download")]
    ConfirmZkNymDownloadEndpointFailure {
        endpoint_failure: VpnApiEndpointFailure,
        id: ZkNymId,
    },

    #[error("missing pending request: {0}")]
    MissingPendingRequest(ZkNymId),

    #[error("failed to remove pending zk-nym request {id}: {error}")]
    RemovePendingRequest { id: String, error: String },

    #[error("credential storage error: {0}")]
    CredentialStorage(String),

    #[error("internal error: {0}")]
    Internal(String),
}

impl RequestZkNymError {
    pub fn internal(message: impl ToString) -> Self {
        RequestZkNymError::Internal(message.to_string())
    }

    pub fn message(&self) -> String {
        match self {
            RequestZkNymError::RequestZkNymEndpointFailure {
                endpoint_failure,
                ticket_type: _,
            }
            | RequestZkNymError::PollZkNymEndpointFailure {
                endpoint_failure,
                ticket_type: _,
            } => endpoint_failure.message.clone(),
            other => other.to_string(),
        }
    }

    pub fn message_id(&self) -> Option<String> {
        match self {
            RequestZkNymError::RequestZkNymEndpointFailure {
                endpoint_failure,
                ticket_type: _,
            }
            | RequestZkNymError::PollZkNymEndpointFailure {
                endpoint_failure,
                ticket_type: _,
            } => endpoint_failure.message_id.clone(),
            _ => None,
        }
    }

    pub fn ticket_type(&self) -> Option<String> {
        match self {
            RequestZkNymError::RequestZkNymEndpointFailure {
                endpoint_failure: _,
                ticket_type,
            }
            | RequestZkNymError::PollZkNymEndpointFailure {
                endpoint_failure: _,
                ticket_type,
            } => Some(ticket_type.clone()),
            RequestZkNymError::FinishedWithError {
                id: _,
                ticket_type,
                status: _,
            }
            | RequestZkNymError::ImportZkNym {
                ticket_type,
                error: _,
            } => Some(ticket_type.clone()),
            _ => None,
        }
    }
}

pub type RequestZkNymSummary = Vec<Result<RequestZkNymSuccess, RequestZkNymError>>;
