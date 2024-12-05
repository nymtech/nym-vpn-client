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

use futures::StreamExt;
use nym_compact_ecash::{Base58, BlindedSignature, VerificationKeyAuth, WithdrawalRequest};
use nym_credential_proxy_requests::api::v1::ticketbook::models::{
    PartialVerificationKeysResponse, TicketbookWalletSharesResponse,
};
use nym_credentials::{EpochVerificationKey, IssuedTicketBook};
use nym_credentials_interface::{PublicKeyUser, RequestInfo, TicketType};
use nym_ecash_time::EcashTime;
use nym_vpn_api_client::{
    response::{NymVpnZkNym, NymVpnZkNymPost, NymVpnZkNymStatus},
    types::{Device, VpnApiAccount},
    VpnApiClient,
};
use serde::{Deserialize, Serialize};
use time::Date;
use tokio::task::JoinSet;

use crate::{
    commands::VpnApiEndpointFailure, error::Error, shared_state::RequestZkNymResult,
    storage::VpnCredentialStorage, SharedAccountState,
};

use super::{AccountCommandError, AccountCommandResult};

// The maximum number of zk-nym requests that can fail in a row before we disable background
// refresh
const ZK_NYM_MAX_FAILS: u32 = 10;

pub(crate) struct WaitingRequestZkNymCommandHandler {
    credential_storage: VpnCredentialStorage,
    account_state: SharedAccountState,
    vpn_api_client: VpnApiClient,
    zk_nym_fails_in_a_row: Arc<AtomicU32>,
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
        }
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
}

impl RequestZkNymCommandHandler {
    fn id_str(&self) -> String {
        format!("{:.8}", self.id.to_string())
    }

    pub(crate) async fn run(self) -> AccountCommandResult {
        AccountCommandResult::RequestZkNym(self.request_zk_nym().await)
    }

    #[tracing::instrument(
        skip(self),
        fields(id = %self.id_str()),
        ret,
        err,
    )]
    async fn request_zk_nym(mut self) -> Result<RequestZkNymSuccessSummary, AccountCommandError> {
        tracing::debug!("Running zk-nym request command handler: {}", self.id);

        // Defensive check for something that should not be possible
        if let Some(RequestZkNymResult::InProgress) =
            self.account_state.lock().await.request_zk_nym_result
        {
            return Err(AccountCommandError::internal(
                "duplicate zk-nym request command",
            ));
        }
        let ticket_types = self.check_ticket_types_running_low().await?;
        tracing::debug!("Ticket types running low: {:?}", ticket_types);

        self.account_state
            .set_zk_nym_request(RequestZkNymResult::InProgress)
            .await;

        match self.request_zk_nym_inner(ticket_types).await {
            Ok(success_summary) => {
                self.account_state
                    .set_zk_nym_request(RequestZkNymResult::from(success_summary.clone()))
                    .await;
                Ok(success_summary)
            }
            Err(error_summary) => {
                self.account_state
                    .set_zk_nym_request(RequestZkNymResult::from(error_summary.clone()))
                    .await;
                tracing::warn!(
                    "We have reached {} zk-nym fails in a row",
                    self.zk_nym_fails_in_a_row.load(Ordering::Relaxed),
                );
                Err(AccountCommandError::from(error_summary))
            }
        }
    }

    async fn check_ticket_types_running_low(&self) -> Result<Vec<TicketType>, AccountCommandError> {
        self.credential_storage
            .check_ticket_types_running_low()
            .await
            .map_err(AccountCommandError::general)
    }

    async fn request_zk_nym_inner(
        &mut self,
        ticket_types: Vec<TicketType>,
    ) -> Result<RequestZkNymSuccessSummary, RequestZkNymErrorSummary> {
        tracing::info!("Checking which ticket types are running low");
        if ticket_types.is_empty() {
            tracing::info!("No ticket types needed, skipping zk-nym request");
            return Ok(RequestZkNymSuccessSummary::NoneNeeded);
        }
        tracing::info!("Ticket types needed: {:?}", ticket_types);

        // Request zk-nyms for each ticket type that we need
        let responses = futures::stream::iter(ticket_types)
            .filter_map(|ticket_type| {
                let account = self.account.clone();
                async move { construct_zk_nym_request_data(&account, ticket_type).ok() }
            })
            .map(|request| {
                let account = self.account.clone();
                let device = self.device.clone();
                let vpn_api_client = self.vpn_api_client.clone();
                async move { request_zk_nym(request, &account, &device, &vpn_api_client).await }
            })
            .buffer_unordered(4)
            .collect::<Vec<_>>()
            .await;

        // Spawn polling tasks for each zk-nym request to monitor the outcome
        let (zk_nym_polling_tasks, mut request_zk_nym_errors) =
            self.handle_request_zk_nym_responses(responses).await;

        // Wait for the polling tasks to finish
        let zk_nym_successes = self
            .wait_for_polling_tasks(zk_nym_polling_tasks, &mut request_zk_nym_errors)
            .await;

        if request_zk_nym_errors.is_empty() {
            tracing::info!("zk-nym request command handler finished: {}", self.id);
            Ok(RequestZkNymSuccessSummary::All(zk_nym_successes))
        } else {
            tracing::warn!(
                "zk-nym request command handler finished with errors: {}",
                self.id
            );
            Err(RequestZkNymErrorSummary {
                successes: zk_nym_successes,
                failed: request_zk_nym_errors,
            })
        }
    }

    async fn handle_request_zk_nym_responses(
        &self,
        responses: Vec<(ZkNymRequestData, Result<NymVpnZkNymPost, Error>)>,
    ) -> (
        JoinSet<Result<PollingResult, RequestZkNymError>>,
        Vec<RequestZkNymError>,
    ) {
        let mut request_zk_nym_errors = Vec::new();
        let mut zk_nym_polling_tasks = JoinSet::new();
        for (request, response) in responses {
            match response {
                Ok(response) => {
                    zk_nym_polling_tasks.spawn(poll_zk_nym(
                        request,
                        response,
                        self.account.clone(),
                        self.device.clone(),
                        self.vpn_api_client.clone(),
                    ));
                }
                Err(err) => {
                    tracing::debug!("Failed to request zk-nym: {:#?}", err);
                    let err = nym_vpn_api_client::response::extract_error_response(&err)
                        .map(|e| {
                            tracing::warn!(
                                "nym-vpn-api reports: message={}, message_id={:?}, code_reference_id={:?}",
                                e.message,
                                e.message_id,
                                e.code_reference_id,
                            );
                            RequestZkNymError::RequestZkNymEndpointFailure {
                                endpoint_failure: VpnApiEndpointFailure {
                                    message_id: e.message_id.clone(),
                                    message: e.message.clone(),
                                    code_reference_id: e.code_reference_id.clone(),
                                },
                                ticket_type: request.ticketbook_type.to_string(),
                            }
                        })
                        .unwrap_or_else(|| RequestZkNymError::internal(err));
                    self.zk_nym_fails_in_a_row.fetch_add(1, Ordering::Relaxed);
                    request_zk_nym_errors.push(err);
                }
            }
        }
        (zk_nym_polling_tasks, request_zk_nym_errors)
    }

    async fn wait_for_polling_tasks(
        &mut self,
        mut zk_nym_polling_tasks: JoinSet<Result<PollingResult, RequestZkNymError>>,
        request_zk_nym_errors: &mut Vec<RequestZkNymError>,
    ) -> Vec<RequestZkNymSuccess> {
        let mut zk_nym_successes = Vec::new();
        while let Some(polling_result) = zk_nym_polling_tasks.join_next().await {
            let result = match polling_result {
                Ok(result) => result,
                Err(err) => {
                    tracing::error!("Failed to join zk-nym polling task: {:#?}", err);
                    request_zk_nym_errors.push(RequestZkNymError::PollingTaskError);
                    continue;
                }
            };

            match result {
                Ok(PollingResult::Finished(
                    response,
                    ticketbook_type,
                    request_info,
                    request_data,
                )) => {
                    if response.status == NymVpnZkNymStatus::Active {
                        tracing::info!(
                            "Polling finished succesfully, importing ticketbook: {}",
                            response.id
                        );
                        let id = response.id.clone();
                        match self
                            .import_zk_nym(response, ticketbook_type, *request_info, *request_data)
                            .await
                        {
                            Ok(_) => {
                                tracing::info!("Successfully imported zk-nym: {}", id);
                                self.zk_nym_fails_in_a_row.store(0, Ordering::Relaxed);
                                zk_nym_successes.push(RequestZkNymSuccess::new(id.clone()));
                                if let Err(err) = self.confirm_zk_nym_downloaded(&id).await {
                                    tracing::warn!("Failed to confirm zk-nym downloaded: {err:?}");
                                }
                            }
                            Err(err) => {
                                tracing::error!("Failed to import zk-nym: {:#?}", err);
                                self.zk_nym_fails_in_a_row.fetch_add(1, Ordering::Relaxed);
                                request_zk_nym_errors.push(RequestZkNymError::Import {
                                    id,
                                    ticket_type: ticketbook_type.to_string(),
                                    error: err.to_string(),
                                });
                            }
                        }
                    } else {
                        tracing::warn!(
                            "Polling for {} finished with NOT active status: {:?}",
                            response.id,
                            response.status,
                        );
                        tracing::warn!("Not importing zk-nym: {}", response.id);
                        self.zk_nym_fails_in_a_row.fetch_add(1, Ordering::Relaxed);
                        request_zk_nym_errors.push(RequestZkNymError::FinishedWithError {
                            id: response.id.clone(),
                            ticket_type: ticketbook_type.to_string(),
                            status: response.status.clone(),
                        });
                    }
                }
                Err(err) => {
                    tracing::error!("zk-nym polling error: {:#?}", err);
                    self.zk_nym_fails_in_a_row.fetch_add(1, Ordering::Relaxed);
                    request_zk_nym_errors.push(err);
                }
            }
        }
        zk_nym_successes
    }

    async fn import_zk_nym(
        &mut self,
        response: NymVpnZkNym,
        ticketbook_type: TicketType,
        request_info: RequestInfo,
        request: ZkNymRequestData,
    ) -> Result<(), Error> {
        tracing::info!("Importing zk-nym: {}", response.id);

        let Some(ref shares) = response.blinded_shares else {
            return Err(Error::MissingBlindedShares);
        };

        let issuers = self
            .vpn_api_client
            .get_directory_zk_nyms_ticketbookt_partial_verification_keys()
            .await
            .map_err(Error::GetZkNyms)?;

        if shares.epoch_id != issuers.epoch_id {
            return Err(Error::InconsistentEpochId);
        }

        tracing::info!("epoch_id: {}", shares.epoch_id);

        let master_vk_bs58 = shares
            .master_verification_key
            .clone()
            .ok_or(Error::MissingMasterVerificationKey)?
            .bs58_encoded_key;

        let master_vk = VerificationKeyAuth::try_from_bs58(&master_vk_bs58)
            .map_err(Error::InvalidMasterVerificationKey)?;

        let expiration_date = request.expiration_date;

        let issued_ticketbook = crate::commands::request_zknym::unblind_and_aggregate(
            shares.clone(),
            issuers,
            master_vk.clone(),
            ticketbook_type,
            expiration_date.ecash_date(),
            request_info,
            self.account.clone(),
        )
        .await?;

        // Insert master verification key
        tracing::info!("Inserting master verification key");
        let epoch_vk = EpochVerificationKey {
            epoch_id: shares.epoch_id,
            key: master_vk,
        };
        self.credential_storage
            .insert_master_verification_key(&epoch_vk)
            .await
            .inspect_err(|err| {
                tracing::error!("Failed to insert master verification key: {:#?}", err);
            })
            .ok();

        // Insert aggregated coin index signatures, if available
        if let Some(aggregated_coin_index_signatures) = &shares.aggregated_coin_index_signatures {
            tracing::info!("Inserting coin index signatures");
            self.credential_storage
                .insert_coin_index_signatures(&aggregated_coin_index_signatures.signatures)
                .await
                .inspect_err(|err| {
                    tracing::error!("Failed to insert coin index signatures: {:#?}", err);
                })
                .ok();
        }

        // Insert aggregated expiration date signatures, if available
        if let Some(aggregated_expiration_date_signatures) =
            &shares.aggregated_expiration_date_signatures
        {
            tracing::info!("Inserting expiration date signatures");
            self.credential_storage
                .insert_expiration_date_signatures(
                    &aggregated_expiration_date_signatures.signatures,
                )
                .await
                .inspect_err(|err| {
                    tracing::error!("Failed to insert expiration date signatures: {:#?}", err);
                })
                .ok();
        }

        tracing::info!("Inserting issued ticketbook");
        self.credential_storage
            .insert_issued_ticketbook(&issued_ticketbook)
            .await?;

        Ok(())
    }

    async fn confirm_zk_nym_downloaded(&self, id: &str) -> Result<(), Error> {
        self.vpn_api_client
            .confirm_zk_nym_download_by_id(&self.account, &self.device, id)
            .await
            .map_err(Error::ConfirmZkNymDownload)?;
        tracing::info!("Confirmed zk-nym downloaded: {}", id);
        Ok(())
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

pub(crate) fn construct_zk_nym_request_data(
    account: &VpnApiAccount,
    ticketbook_type: TicketType,
) -> Result<ZkNymRequestData, Error> {
    tracing::info!("Requesting zk-nym by type: {}", ticketbook_type);

    let ecash_keypair = account
        .create_ecash_keypair()
        .map_err(Error::CreateEcashKeyPair)?;
    let expiration_date = nym_ecash_time::ecash_default_expiration_date();

    let (withdrawal_request, request_info) = nym_compact_ecash::withdrawal_request(
        ecash_keypair.secret_key(),
        expiration_date.ecash_unix_timestamp(),
        ticketbook_type.encode(),
    )
    .map_err(Error::ConstructWithdrawalRequest)?;

    let ecash_pubkey = ecash_keypair.public_key();

    Ok(ZkNymRequestData {
        withdrawal_request,
        ecash_pubkey,
        expiration_date,
        ticketbook_type,
        request_info,
    })
}

pub(crate) async fn request_zk_nym(
    request: ZkNymRequestData,
    account: &VpnApiAccount,
    device: &Device,
    vpn_api_client: &nym_vpn_api_client::VpnApiClient,
) -> (ZkNymRequestData, Result<NymVpnZkNymPost, Error>) {
    let response = vpn_api_client
        .request_zk_nym(
            account,
            device,
            request.withdrawal_request.to_bs58(),
            request.ecash_pubkey.to_base58_string().to_owned(),
            request.expiration_date.to_string(),
            request.ticketbook_type.to_string(),
        )
        .await
        .map_err(Error::RequestZkNym);
    (request, response)
}

pub(crate) async fn poll_zk_nym(
    request: ZkNymRequestData,
    response: NymVpnZkNymPost,
    account: VpnApiAccount,
    device: Device,
    api_client: nym_vpn_api_client::VpnApiClient,
) -> Result<PollingResult, RequestZkNymError> {
    tracing::info!("Starting zk-nym polling task for {}", response.id);
    tracing::info!("which had response : {:#?}", response);
    let start_time = Instant::now();
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;

        tracing::info!("Polling zk-nym status: {}", &response.id);
        match api_client
            .get_zk_nym_by_id(&account, &device, &response.id)
            .await
        {
            Ok(poll_response) if poll_response.status != NymVpnZkNymStatus::Pending => {
                tracing::info!("zk-nym polling finished: {}", poll_response.id);
                tracing::debug!("zk-nym polling finished: {:#?}", poll_response);
                return Ok(PollingResult::Finished(
                    poll_response,
                    request.ticketbook_type,
                    Box::new(request.request_info.clone()),
                    Box::new(request),
                ));
            }
            Ok(poll_response) => {
                tracing::info!("zk-nym polling not finished: {:#?}", poll_response);
                if start_time.elapsed() > Duration::from_secs(60) {
                    tracing::error!("zk-nym polling timed out: {}", response.id);
                    return Err(RequestZkNymError::PollingTimeout {
                        id: response.id.clone(),
                        ticket_type: request.ticketbook_type.to_string(),
                    });
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
                            ticket_type: request.ticketbook_type.to_string(),
                        }
                    })
                    .unwrap_or_else(|| RequestZkNymError::internal(error)))
            }
        }
    }
}

pub(crate) async fn unblind_and_aggregate(
    shares: TicketbookWalletSharesResponse,
    issuers: PartialVerificationKeysResponse,
    master_vk: VerificationKeyAuth,
    ticketbook_type: TicketType,
    expiration_date: Date,
    request_info: RequestInfo,
    account: VpnApiAccount,
) -> Result<IssuedTicketBook, Error> {
    let ecash_keypair = account
        .create_ecash_keypair()
        .map_err(Error::CreateEcashKeyPair)?;

    tracing::info!("Setting up decoded keys");

    let mut decoded_keys = HashMap::new();
    for key in issuers.keys {
        let vk = VerificationKeyAuth::try_from_bs58(&key.bs58_encoded_key)
            .inspect_err(|err| tracing::error!("Failed to create VerificationKeyAuth: {:#?}", err))
            .map_err(Error::InvalidVerificationKey)?;
        decoded_keys.insert(key.node_index, vk);
    }

    tracing::info!("Verifying zk-nym shares");

    let mut partial_wallets = Vec::new();
    for share in shares.shares {
        tracing::info!("Creating BlindedSignature");
        let blinded_sig =
            BlindedSignature::try_from_bs58(&share.bs58_encoded_share).map_err(|err| {
                tracing::error!("Failed to create BlindedSignature: {:#?}", err);
                Error::DeserializeBlindedSignature(err)
            })?;

        let Some(vk) = decoded_keys.get(&share.node_index) else {
            return Err(Error::DecodedKeysMissingIndex);
        };

        tracing::info!("Calling issue_verify");
        match nym_compact_ecash::issue_verify(
            vk,
            ecash_keypair.secret_key(),
            &blinded_sig,
            &request_info,
            share.node_index,
        ) {
            Ok(partial_wallet) => {
                tracing::info!("Partial wallet created and appended");
                partial_wallets.push(partial_wallet)
            }
            Err(err) => {
                tracing::error!("Failed to issue verify: {:#?}", err);
                return Err(Error::ImportZkNym(err));
            }
        }
    }

    tracing::info!("Aggregating wallets");

    let aggregated_wallets = nym_compact_ecash::aggregate_wallets(
        &master_vk,
        ecash_keypair.secret_key(),
        &partial_wallets,
        &request_info,
    )
    .map_err(Error::AggregateWallets)?;

    tracing::info!("Creating ticketbook");

    let ticketbook = IssuedTicketBook::new(
        aggregated_wallets.into_wallet_signatures(),
        shares.epoch_id,
        ecash_keypair.into(),
        ticketbook_type,
        expiration_date,
    );

    Ok(ticketbook)
}

#[derive(Debug)]
pub(crate) enum PollingResult {
    Finished(
        NymVpnZkNym,
        TicketType,
        Box<RequestInfo>,
        Box<ZkNymRequestData>,
    ),
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RequestZkNymSuccessSummary {
    NoneNeeded,
    All(Vec<RequestZkNymSuccess>),
}

impl RequestZkNymSuccessSummary {
    pub(crate) fn successful_zknym_requests(&self) -> impl Iterator<Item = &RequestZkNymSuccess> {
        match self {
            RequestZkNymSuccessSummary::NoneNeeded => [].iter(),
            RequestZkNymSuccessSummary::All(ids) => ids.iter(),
        }
    }
}

#[derive(Clone, Debug, thiserror::Error, PartialEq, Eq, Serialize, Deserialize)]
pub enum RequestZkNymError {
    #[error("failed to request zk nym endpoint for ticket type: {ticket_type}")]
    RequestZkNymEndpointFailure {
        endpoint_failure: VpnApiEndpointFailure,
        ticket_type: String,
    },

    #[error("error polling for zknym result for ticket type: {ticket_type}")]
    PollZkNymEndpointFailure {
        endpoint_failure: VpnApiEndpointFailure,
        ticket_type: String,
    },

    #[error("polling task failed")]
    PollingTaskError,

    #[error("timeout polling for zknym {id} for ticket type: {ticket_type}")]
    PollingTimeout { id: ZkNymId, ticket_type: String },

    #[error("polling for zknym {id} finished with error for ticket type: {ticket_type}")]
    FinishedWithError {
        id: ZkNymId,
        ticket_type: String,
        status: NymVpnZkNymStatus,
    },

    #[error("failed to import zknym for ticket type: {ticket_type}")]
    Import {
        id: ZkNymId,
        ticket_type: String,
        error: String,
    },

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
            | RequestZkNymError::Import {
                id: _,
                ticket_type,
                error: _,
            }
            | RequestZkNymError::PollingTimeout { id: _, ticket_type } => Some(ticket_type.clone()),
            RequestZkNymError::PollingTaskError | RequestZkNymError::Internal(_) => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequestZkNymErrorSummary {
    pub successes: Vec<RequestZkNymSuccess>,
    pub failed: Vec<RequestZkNymError>,
}
