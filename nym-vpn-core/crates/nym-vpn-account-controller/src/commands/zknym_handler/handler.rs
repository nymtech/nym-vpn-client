// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{sync::Arc, time::Duration};

use nym_credentials_interface::TicketType;
use nym_vpn_api_client::{
    VpnApiClient,
    types::{Device, VpnAccount},
};
use nym_vpn_lib_types::{RequestZkNymError, RequestZkNymSuccess, VpnApiError};
use tokio::task::JoinSet;

use crate::storage::VpnCredentialStorage;

use super::{cached_data::CachedData, request::RequestZkNymTask};

pub(crate) type ZkNymId = String;

pub type RequestZkNymSummary = Vec<Result<RequestZkNymSuccess, RequestZkNymError>>;

pub(crate) struct RequestZkNymCommandHandler {
    account: Arc<VpnAccount>,
    device: Device,
    credential_storage: Arc<tokio::sync::Mutex<VpnCredentialStorage>>,
    vpn_api_client: VpnApiClient,
    cached_data: CachedData,
}

impl RequestZkNymCommandHandler {
    pub(crate) fn new(
        account: Arc<VpnAccount>,
        device: Device,
        storage: VpnCredentialStorage,
        vpn_api_client: VpnApiClient,
    ) -> RequestZkNymCommandHandler {
        RequestZkNymCommandHandler {
            account,
            device,
            credential_storage: Arc::new(tokio::sync::Mutex::new(storage)),
            vpn_api_client: vpn_api_client.clone(),
            cached_data: CachedData::new(vpn_api_client),
        }
    }

    #[tracing::instrument(skip(self), ret)]
    pub(crate) async fn request_zk_nyms(
        &self,
        ticket_types: Vec<TicketType>,
    ) -> RequestZkNymSummary {
        // If we have pending zk-nym ticketbooks, try those first
        let resumed_requests = self.resume_request_zk_nyms().await;

        let new_requests = if !ticket_types.is_empty() {
            self.request_zk_nyms_for_ticket_types(ticket_types).await
        } else {
            Vec::new()
        };

        resumed_requests
            .into_iter()
            .chain(new_requests.into_iter())
            .collect()
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

        if pending_requests_data.is_empty() {
            // early return to avoid unnecessary network call
            return Ok(Vec::new());
        }

        let zk_nyms_available_for_download = self.get_zk_nyms_available_for_download().await?;
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
                VpnApiError::try_from(err)
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
