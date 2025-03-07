// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    },
    time::Duration,
};

use nym_credentials_interface::TicketType;
use nym_vpn_api_client::{
    types::{Device, VpnApiAccount},
    VpnApiClient,
};
use nym_vpn_lib_types::{RequestZkNymError, RequestZkNymSuccess, VpnApiErrorResponse};
use tokio::task::JoinSet;

use crate::{
    commands::AccountCommandResult, shared_state::RequestZkNymResult,
    storage::VpnCredentialStorage, SharedAccountState,
};

use super::{cached_data::CachedData, request::RequestZkNymTask};

// The maximum number of zk-nym requests that can fail in a row before we disable background
// refresh
const ZK_NYM_MAX_FAILS: u32 = 10;

pub(crate) type ZkNymId = String;

pub type RequestZkNymSummary = Vec<Result<RequestZkNymSuccess, RequestZkNymError>>;

pub(crate) struct WaitingRequestZkNymCommandHandler {
    credential_storage: Arc<tokio::sync::Mutex<VpnCredentialStorage>>,
    account_state: SharedAccountState,
    vpn_api_client: VpnApiClient,
    zk_nym_fails_in_a_row: Arc<AtomicU32>,

    // Cache some of the data used to import zk-nyms between requests, to speed things up. Consider
    // persisting this to storage
    cached_data: CachedData,
}

impl WaitingRequestZkNymCommandHandler {
    pub(crate) fn new(
        credential_storage: Arc<tokio::sync::Mutex<VpnCredentialStorage>>,
        account_state: SharedAccountState,
        vpn_api_client: nym_vpn_api_client::VpnApiClient,
    ) -> Self {
        let cached_data = CachedData::new(vpn_api_client.clone());
        WaitingRequestZkNymCommandHandler {
            credential_storage,
            account_state,
            vpn_api_client,
            zk_nym_fails_in_a_row: Default::default(),
            cached_data,
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

    pub(crate) fn update_vpn_api_client(&mut self, vpn_api_client: VpnApiClient) {
        self.vpn_api_client = vpn_api_client.clone();
        self.cached_data.update_vpn_api_client(vpn_api_client);
    }
}

pub(crate) struct RequestZkNymCommandHandler {
    id: uuid::Uuid,
    account: VpnApiAccount,
    device: Device,
    credential_storage: Arc<tokio::sync::Mutex<VpnCredentialStorage>>,
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

    async fn request_zk_nyms_outer(self) -> Result<RequestZkNymSummary, RequestZkNymError> {
        tracing::debug!("Running zk-nym request command handler: {}", self.id);

        // Defensive check for something that should not be possible
        if self.account_state.is_zk_nym_request_in_progress().await {
            return Err(RequestZkNymError::internal(
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
                Err(err)
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

        let ticket_types = self.get_ticket_types_running_low().await?;
        tracing::debug!("Ticket types running low: {ticket_types:?}");

        let new_requests = if !ticket_types.is_empty() {
            self.request_zk_nyms_for_ticket_types(ticket_types).await
        } else {
            Vec::new()
        };

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

    async fn get_ticket_types_running_low(&self) -> Result<Vec<TicketType>, RequestZkNymError> {
        self.credential_storage
            .lock()
            .await
            .get_ticket_types_running_low()
            .await
            .map_err(RequestZkNymError::internal)
    }

    async fn request_zk_nyms_for_ticket_types(
        &self,
        ticket_types: Vec<TicketType>,
    ) -> Vec<Result<RequestZkNymSuccess, RequestZkNymError>> {
        tracing::info!("Requesting zk-nym ticketbooks for: {ticket_types:?}");

        let mut join_set = JoinSet::new();
        for ticket_type in ticket_types {
            let task = RequestZkNymTask::new(
                self.account.clone(),
                self.device.clone(),
                self.vpn_api_client.clone(),
                self.credential_storage.clone(),
                self.cached_data.clone(),
            );
            join_set.spawn(async move { task.request_zk_nym_ticketbook(ticket_type).await });
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

        // Cleaning up stale requests as a tidy task. Calling this here out of convenience but it
        // could just as well be a background task
        self.credential_storage
            .lock()
            .await
            .clean_up_stale_requests()
            .await
            .inspect_err(|err| {
                tracing::error!("Failed to clean up stale requests: {:?}", err);
            })
            .ok();

        let pending_requests_data = self
            .credential_storage
            .lock()
            .await
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
                VpnApiErrorResponse::try_from(err)
                    .map(|response| {
                        RequestZkNymError::GetZkNymsAvailableForDownloadEndpointFailure { response }
                    })
                    .unwrap_or_else(RequestZkNymError::unexpected_response)
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

        let mut join_set = JoinSet::new();
        for pending_request in pending_requests {
            let task = RequestZkNymTask::new(
                self.account.clone(),
                self.device.clone(),
                self.vpn_api_client.clone(),
                self.credential_storage.clone(),
                self.cached_data.clone(),
            );
            join_set
                .spawn(async move { task.resume_request_zk_nym_ticketbook(pending_request).await });
        }
        wait_for_join_set(join_set).await
    }
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
