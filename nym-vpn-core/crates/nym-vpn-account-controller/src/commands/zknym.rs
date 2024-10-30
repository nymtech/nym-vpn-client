// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::time::{Duration, Instant};

use nym_compact_ecash::{Base58, WithdrawalRequest};
use nym_credentials_interface::{RequestInfo, TicketType};
use nym_ecash_time::EcashTime;
use nym_vpn_api_client::{
    response::{NymVpnZkNym, NymVpnZkNymStatus},
    types::{Device, VpnApiAccount},
    VpnApiClientError,
};

use crate::error::Error;

pub(crate) struct ZkNymRequestData {
    withdrawal_request: WithdrawalRequest,
    ecash_pubkey: String,
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

    let ecash_pubkey = ecash_keypair.public_key().to_base58_string();

    Ok(ZkNymRequestData {
        withdrawal_request,
        ecash_pubkey,
        ticketbook_type,
        request_info,
    })
}

pub(crate) async fn request_zk_nym(
    request: ZkNymRequestData,
    account: &VpnApiAccount,
    device: &Device,
    vpn_api_client: &nym_vpn_api_client::VpnApiClient,
) -> (ZkNymRequestData, Result<NymVpnZkNym, Error>) {
    let response = vpn_api_client
        .request_zk_nym(
            account,
            device,
            request.withdrawal_request.to_bs58(),
            request.ecash_pubkey.to_owned(),
            request.ticketbook_type.to_string(),
        )
        .await
        .map_err(Error::RequestZkNym);
    (request, response)
}

pub(crate) async fn poll_zk_nym(
    request: ZkNymRequestData,
    response: NymVpnZkNym,
    account: VpnApiAccount,
    device: Device,
    api_client: nym_vpn_api_client::VpnApiClient,
) -> PollingResult {
    tracing::info!("Starting zk-nym polling task for {}", response.id);
    let start_time = Instant::now();
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;

        tracing::info!("Polling zk-nym status: {}", &response.id);
        match api_client
            .get_zk_nym_by_id(&account, &device, &response.id)
            .await
        {
            Ok(poll_response) if response.status != NymVpnZkNymStatus::Pending => {
                tracing::info!("zk-nym polling finished: {:#?}", poll_response);
                return PollingResult::Finished(
                    poll_response,
                    request.ticketbook_type,
                    Box::new(request.request_info),
                );
            }
            Ok(poll_response) => {
                tracing::info!("zk-nym polling not finished: {:#?}", poll_response);
                if start_time.elapsed() > Duration::from_secs(60) {
                    tracing::error!("zk-nym polling timed out: {}", response.id);
                    return PollingResult::Timeout(poll_response);
                }
            }
            Err(error) => {
                tracing::error!(
                    "Failed to poll zk-nym ({}) status: {:#?}",
                    response.id,
                    error
                );
                return PollingResult::Error(PollingError {
                    id: response.id,
                    error,
                });
            }
        }
    }
}

#[derive(Debug)]
pub(crate) enum PollingResult {
    Finished(NymVpnZkNym, TicketType, Box<RequestInfo>),
    Timeout(NymVpnZkNym),
    Error(PollingError),
}

#[derive(Debug)]
pub(crate) struct PollingError {
    pub(crate) id: String,
    pub(crate) error: VpnApiClientError,
}
