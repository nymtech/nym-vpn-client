// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::time::{Duration, Instant};

use nym_compact_ecash::{Base58, BlindedSignature, VerificationKeyAuth, WithdrawalRequest};
use nym_credentials::IssuedTicketBook;
use nym_credentials_interface::{PublicKeyUser, RequestInfo, TicketType};
use nym_ecash_time::EcashTime;
use nym_vpn_api_client::{
    response::{NymVpnZkNym, NymVpnZkNymStatus},
    types::{Device, VpnApiAccount},
    VpnApiClientError,
};
use time::{format_description::well_known::Rfc3339, Date, OffsetDateTime};

use crate::{error::Error, models::WalletShare};

pub(crate) struct ZkNymRequestData {
    withdrawal_request: WithdrawalRequest,
    ecash_pubkey: PublicKeyUser,
    expiration_date: Date,
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
) -> (ZkNymRequestData, Result<NymVpnZkNym, Error>) {
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

pub(crate) async fn unblind_and_aggregate(
    response: NymVpnZkNym,
    ticketbook_type: TicketType,
    request_info: RequestInfo,
    account: VpnApiAccount,
    vk_auth: VerificationKeyAuth,
) -> Result<IssuedTicketBook, Error> {
    let ecash_keypair = account
        .create_ecash_keypair()
        .map_err(Error::CreateEcashKeyPair)?;

    let mut partial_wallets = Vec::new();
    for blinded_share in response.blinded_shares {
        // TODO: remove unwrap
        let blinded_share: WalletShare = serde_json::from_str(&blinded_share).unwrap();

        // TODO: remove unwrap
        let blinded_sig =
            BlindedSignature::try_from_bs58(&blinded_share.bs58_encoded_share).unwrap();

        match nym_compact_ecash::issue_verify(
            &vk_auth,
            ecash_keypair.secret_key(),
            &blinded_sig,
            &request_info,
            blinded_share.node_index,
        ) {
            Ok(partial_wallet) => partial_wallets.push(partial_wallet),
            Err(err) => {
                tracing::error!("Failed to issue verify: {:#?}", err);
                return Err(Error::ImportZkNym(err));
            }
        }
    }

    // TODO: remove unwrap
    let aggregated_wallets = nym_compact_ecash::aggregate_wallets(
        &vk_auth,
        ecash_keypair.secret_key(),
        &partial_wallets,
        &request_info,
    )
    .unwrap();

    // TODO: remove unwrap
    let expiration_date = OffsetDateTime::parse(&response.valid_until_utc, &Rfc3339).unwrap();

    let ticketbook = IssuedTicketBook::new(
        aggregated_wallets.into_wallet_signatures(),
        response.epoch.unwrap(),
        ecash_keypair.into(),
        ticketbook_type,
        expiration_date.ecash_date(),
    );

    Ok(ticketbook)
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
