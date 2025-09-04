// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::error::Error;

pub use nym_http_api_client::HttpClientError;
use crate::response::NymErrorResponse;
// Error code id to allow error catching. These are emitted by the backend and are unique.

// https://github.com/nymtech/websites/blob/e92383143e195c97c2a3043d93daff06debaab74/www/vpn-api/src/app/api/public/v1/account/%5BaccountId%5D/device/%5BdeviceId%5D/route.ts#L218
pub const UNREGISTER_NON_EXISTENT_DEVICE_CODE_ID: &str = "235ba475-8c64-4c46-8147-d1d523df972c";

// https://github.com/nymtech/websites/blob/e92383143e195c97c2a3043d93daff06debaab74/www/vpn-api/src/app/api/public/v1/account/%5BaccountId%5D/device/%5BdeviceId%5D/zknym/route.ts#L255
pub const FAIR_USAGE_DEPLETED_CODE_ID: &str = "e0b78604-bb9b-4524-add1-f50fe26144c6";

#[derive(Debug, thiserror::Error)]
pub enum VpnApiClientError {
    #[error("failed to create vpn api client")]
    CreateVpnApiClient(#[source] HttpClientError),

    #[error("failed to get account")]
    GetAccount(#[source] HttpClientError),

    #[error("failed to get account summary")]
    GetAccountSummary(#[source] HttpClientError),

    #[error("failed to get account summary with device")]
    GetAccountSummaryWithDevice(#[source] HttpClientError),

    #[error("failed to get devices")]
    GetDevices(#[source] HttpClientError),

    #[error("failed to register device")]
    RegisterDevice(#[source] HttpClientError),

    #[error("failed to get active devices")]
    GetActiveDevices(#[source] HttpClientError),

    #[error("failed to get device by id")]
    GetDeviceById(#[source] HttpClientError),

    #[error("failed to get device zk-nym")]
    GetDeviceZkNyms(#[source] HttpClientError),

    #[error("failed to update device")]
    UpdateDevice(#[source] HttpClientError),

    #[error("failed to request zk-nym")]
    RequestZkNym(#[source] HttpClientError),

    #[error("failed to get active zk-nym")]
    GetActiveZkNym(#[source] HttpClientError),

    #[error("failed to get zk-nym by id")]
    GetZkNymById(#[source] HttpClientError),

    #[error("failed to confirm zk-nym download")]
    ConfirmZkNymDownloadById(#[source] HttpClientError),

    #[error("failed to get free passes")]
    GetFreePasses(#[source] HttpClientError),

    #[error("failed to apply free pass")]
    ApplyFreepass(#[source] HttpClientError),

    #[error("failed to get subscriptions")]
    GetSubscriptions(#[source] HttpClientError),

    #[error("failed to create subscription")]
    CreateSubscription(#[source] HttpClientError),

    #[error("failed to get active subscription")]
    GetActiveSubscriptions(#[source] HttpClientError),

    #[error("failed to get gateways")]
    GetGateways(#[source] HttpClientError),

    #[error("failed to get gateway countries")]
    GetGatewayCountries(#[source] HttpClientError),

    #[error("failed to get entry gateways")]
    GetEntryGateways(#[source] HttpClientError),

    #[error("failed to get entry gateway countries")]
    GetEntryGatewayCountries(#[source] HttpClientError),

    #[error("failed to get exit gateways")]
    GetExitGateways(#[source] HttpClientError),

    #[error("failed to get exit gateway countries")]
    GetExitGatewayCountries(#[source] HttpClientError),

    #[error("failed to get vpn gateways")]
    GetVpnGateways(#[source] HttpClientError),

    #[error("failed to get vpn gateway countries")]
    GetVpnGatewayCountries(#[source] HttpClientError),

    #[error("failed to get directory zk-nym ticketbook partial verification keys")]
    GetDirectoryZkNymsTicketbookPartialVerificationKeys(#[source] HttpClientError),

    #[error("failed to get health")]
    GetHealth(#[source] HttpClientError),

    #[error("failed to get wellknown environments")]
    GetWellknownEnvs(#[source] HttpClientError),

    #[error("failed to get wellknown discovery")]
    GetWellknownDiscovery(#[source] HttpClientError),

    #[error("failed to get usage")]
    GetUsage(#[source] HttpClientError),

    #[error("failed to get registered network environments")]
    GetNetworkEnvs(#[source] HttpClientError),

    #[error("failed to get discovery info")]
    GetDiscoveryInfo(#[source] HttpClientError),

    #[error("failed to get vpn network Details")]
    GetVpnNetworkDetails(#[source] HttpClientError),

    #[error("failed to post account")]
    PostAccount(#[source] HttpClientError),
}

pub type Result<T, E = VpnApiClientError> = std::result::Result<T, E>;

impl From<VpnApiClientError> for NymErrorResponse {
    fn from(response: VpnApiClientError) -> Self {
        crate::response::extract_error_response(response)
    }
}

impl VpnApiClientError {
    pub fn http_client_error(&self) -> Option<&HttpClientError> {
        self.source()
            .and_then(|source| source.downcast_ref::<HttpClientError>())
    }
}
