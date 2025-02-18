// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

pub use nym_http_api_client::HttpClientError;

use nym_contracts_common::ContractsCommonError;

use crate::response::{ErrorMessage, NymErrorResponse, UnexpectedError};

#[derive(Debug, thiserror::Error)]
pub enum VpnApiClientError {
    #[error("failed tp create vpn api client")]
    FailedToCreateVpnApiClient(#[source] HttpClientError<UnexpectedError>),

    #[error("failed to get account")]
    FailedToGetAccount(#[source] HttpClientError<NymErrorResponse>),

    #[error("failed to get account summary")]
    FailedToGetAccountSummary(#[source] HttpClientError<NymErrorResponse>),

    #[error("failed to get devices")]
    FailedToGetDevices(#[source] HttpClientError<NymErrorResponse>),

    #[error("failed to register device")]
    FailedToRegisterDevice(#[source] HttpClientError<NymErrorResponse>),

    #[error("failed to get active devices")]
    FailedToGetActiveDevices(#[source] HttpClientError<NymErrorResponse>),

    #[error("failed to get device by id")]
    FailedToGetDeviceById(#[source] HttpClientError<NymErrorResponse>),

    #[error("failed to get device zk-nym")]
    FailedToGetDeviceZkNyms(#[source] HttpClientError<NymErrorResponse>),

    #[error("failed to update device")]
    FailedToUpdateDevice(#[source] HttpClientError<NymErrorResponse>),

    #[error("failed to request zk-nym")]
    FailedToRequestZkNym(#[source] HttpClientError<NymErrorResponse>),

    #[error("failed to get active zk-nym")]
    FailedToGetActiveZkNym(#[source] HttpClientError<NymErrorResponse>),

    #[error("failed to get zk-nym by id")]
    FailedToGetZkNymById(#[source] HttpClientError<NymErrorResponse>),

    #[error("failed to confirm zk-nym download")]
    FailedToConfirmZkNymDownloadById(#[source] HttpClientError<NymErrorResponse>),

    #[error("failed to get free passes")]
    FailedToGetFreePasses(#[source] HttpClientError<ErrorMessage>),

    #[error("failed to apply free pass")]
    FailedToApplyFreepass(#[source] HttpClientError<NymErrorResponse>),

    #[error("failed to get subscriptions")]
    FailedToGetSubscriptions(#[source] HttpClientError<NymErrorResponse>),

    #[error("failed to create subscription")]
    FailedToCreateSubscription(#[source] HttpClientError<NymErrorResponse>),

    #[error("failed to get active subscription")]
    FailedToGetActiveSubscriptions(#[source] HttpClientError<NymErrorResponse>),

    #[error("failed to get gateways")]
    FailedToGetGateways(#[source] HttpClientError<UnexpectedError>),

    #[error("failed to get gateway countries")]
    FailedToGetGatewayCountries(#[source] HttpClientError<UnexpectedError>),

    #[error("failed to get entry gateways")]
    FailedToGetEntryGateways(#[source] HttpClientError<UnexpectedError>),

    #[error("failed to get entry gateway countries")]
    FailedToGetEntryGatewayCountries(#[source] HttpClientError<UnexpectedError>),

    #[error("failed to get exit gateways")]
    FailedToGetExitGateways(#[source] HttpClientError<UnexpectedError>),

    #[error("failed to get exit gateway countries")]
    FailedToGetExitGatewayCountries(#[source] HttpClientError<UnexpectedError>),

    #[error("failed to get vpn gateways")]
    FailedToGetVpnGateways(#[source] HttpClientError<UnexpectedError>),

    #[error("failed to get vpn gateway countries")]
    FailedToGetVpnGatewayCountries(#[source] HttpClientError<UnexpectedError>),

    #[error("invalud percent value")]
    InvalidPercentValue(#[source] ContractsCommonError),

    #[error("failed to derive from path")]
    CosmosDeriveFromPath(
        #[source] nym_validator_client::signing::direct_wallet::DirectSecp256k1HdWalletError,
    ),

    #[error("failed to get directory zk-nym ticketbook partial verification keys")]
    FailedToGetDirectoryZkNymsTicketbookPartialVerificationKeys(
        #[source] HttpClientError<ErrorMessage>,
    ),

    #[error("failed to get health")]
    FailedToGetHealth(#[source] HttpClientError<UnexpectedError>),

    #[error("failed to get usage")]
    FailedToGetUsage(#[source] HttpClientError<UnexpectedError>),

    #[error("failed to get registered network environments")]
    FailedToGetNetworkEnvs(#[source] HttpClientError<UnexpectedError>),

    #[error("failed to get discovery info")]
    FailedToGetDiscoveryInfo(#[source] HttpClientError<UnexpectedError>),

    #[error("failed to get vpn network Details")]
    FailedToGetVpnNetworkDetails(#[source] HttpClientError<UnexpectedError>),
}

pub type Result<T> = std::result::Result<T, VpnApiClientError>;

impl TryFrom<VpnApiClientError> for NymErrorResponse {
    type Error = VpnApiClientError;

    fn try_from(response: VpnApiClientError) -> std::result::Result<Self, Self::Error> {
        crate::response::extract_error_response(&response).ok_or(response)
    }
}
