// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{collections::HashSet, error::Error};

use crate::{response::NymErrorResponse, types::AccountError};
pub use nym_http_api_client::HttpClientError;
// Error code id to allow error catching. These are emitted by the backend and are unique.

// https://github.com/nymtech/websites/blob/e92383143e195c97c2a3043d93daff06debaab74/www/vpn-api/src/app/api/public/v1/account/%5BaccountId%5D/device/%5BdeviceId%5D/route.ts#L218
pub const UNREGISTER_NON_EXISTENT_DEVICE_CODE_ID: &str = "235ba475-8c64-4c46-8147-d1d523df972c";

// https://github.com/nymtech/websites/blob/e92383143e195c97c2a3043d93daff06debaab74/www/vpn-api/src/app/api/public/v1/account/%5BaccountId%5D/device/%5BdeviceId%5D/zknym/route.ts#L255
pub const FAIR_USAGE_DEPLETED_CODE_ID: &str = "e0b78604-bb9b-4524-add1-f50fe26144c6";

// The credential proxy refused issuance on a condition that clears by itself within minutes, most
// notably while the network rotates its signing keys. Retrying the same request later succeeds.
// https://github.com/nymtech/websites/blob/27bb28f1f765703e3f26e0c04995f8386246f9a4/www/vpn-api/src/app/api/public/v1/account/%5BaccountId%5D/device/%5BdeviceId%5D/zknym/route.ts#L284
pub const UPSTREAM_UNAVAILABLE_CODE_ID: &str = "b4f0a4a9-7c31-4e7c-9b0e-52c1f8e2d6a7";

#[derive(Debug, thiserror::Error)]
pub enum VpnApiClientError {
    #[error("failed to create vpn api client")]
    CreateVpnApiClient(#[source] Box<HttpClientError>),

    #[error("failed to get account")]
    GetAccount(#[source] Box<HttpClientError>),

    #[error("failed to get account summary")]
    GetAccountSummary(#[source] Box<HttpClientError>),

    #[error("failed to get canonical account identity")]
    GetCanonicalAccountIdentity(#[source] Box<HttpClientError>),

    #[error("failed to get account summary with device")]
    GetAccountSummaryWithDevice(#[source] Box<HttpClientError>),

    #[error("failed to link privy account")]
    LinkPrivyAccount(#[source] Box<HttpClientError>),

    #[error("failed to get devices")]
    GetDevices(#[source] Box<HttpClientError>),

    #[error("failed to register device")]
    RegisterDevice(#[source] Box<HttpClientError>),

    #[error("failed to get active devices")]
    GetActiveDevices(#[source] Box<HttpClientError>),

    #[error("failed to get device by id")]
    GetDeviceById(#[source] Box<HttpClientError>),

    #[error("failed to get device zk-nym")]
    GetDeviceZkNyms(#[source] Box<HttpClientError>),

    #[error("failed to update device")]
    UpdateDevice(#[source] Box<HttpClientError>),

    #[error("failed to delete device")]
    DeleteDevice(#[source] Box<HttpClientError>),

    #[error("failed to request zk-nym")]
    RequestZkNym(#[source] Box<HttpClientError>),

    #[error("failed to get active zk-nym")]
    GetActiveZkNym(#[source] Box<HttpClientError>),

    #[error("failed to get zk-nym by id")]
    GetZkNymById(#[source] Box<HttpClientError>),

    #[error("failed to confirm zk-nym download")]
    ConfirmZkNymDownloadById(#[source] Box<HttpClientError>),

    #[error("failed to get free passes")]
    GetFreePasses(#[source] Box<HttpClientError>),

    #[error("failed to apply free pass")]
    ApplyFreepass(#[source] Box<HttpClientError>),

    #[error("failed to get subscriptions")]
    GetSubscriptions(#[source] Box<HttpClientError>),

    #[error("failed to create subscription")]
    CreateSubscription(#[source] Box<HttpClientError>),

    #[error("failed to get active subscription")]
    GetActiveSubscriptions(#[source] Box<HttpClientError>),

    #[error("failed to get gateways")]
    GetGateways(#[source] Box<HttpClientError>),

    #[error("failed to get gateway countries")]
    GetGatewayCountries(#[source] Box<HttpClientError>),

    #[error("failed to get entry gateways")]
    GetEntryGateways(#[source] Box<HttpClientError>),

    #[error("failed to get entry gateway countries")]
    GetEntryGatewayCountries(#[source] Box<HttpClientError>),

    #[error("failed to get exit gateways")]
    GetExitGateways(#[source] Box<HttpClientError>),

    #[error("failed to get exit gateway countries")]
    GetExitGatewayCountries(#[source] Box<HttpClientError>),

    #[error("failed to get vpn gateways")]
    GetVpnGateways(#[source] Box<HttpClientError>),

    #[error("failed to get vpn gateway countries")]
    GetVpnGatewayCountries(#[source] Box<HttpClientError>),

    #[error("failed to get directory zk-nym ticketbook partial verification keys")]
    GetDirectoryZkNymsTicketbookPartialVerificationKeys(#[source] Box<HttpClientError>),

    #[error("failed to get directory zk-nym ticketbook master verification key")]
    GetDirectoryZkNymsTicketbookMasterVerificationKey(#[source] Box<HttpClientError>),

    #[error("failed to get directory zk-nym ticketbook aggregated coin indices signatures")]
    GetDirectoryZkNymsTicketbookAggregatedCoinIndicesSignatures(#[source] Box<HttpClientError>),

    #[error("failed to get directory zk-nym ticketbook aggregated expiration date signatures")]
    GetDirectoryZkNymsTicketbookAggregatedExpirationDateSignatures(#[source] Box<HttpClientError>),

    #[error("failed to get health")]
    GetHealth(#[source] Box<HttpClientError>),

    #[error("failed to get wellknown environments")]
    GetWellknownEnvs(#[source] Box<HttpClientError>),

    #[error("failed to get wellknown discovery")]
    GetWellknownDiscovery(#[source] Box<HttpClientError>),

    #[error("failed to get usage")]
    GetUsage(#[source] Box<HttpClientError>),

    #[error("failed to get registered network environments")]
    GetNetworkEnvs(#[source] Box<HttpClientError>),

    #[error("failed to get discovery info")]
    GetDiscoveryInfo(#[source] Box<HttpClientError>),

    #[error("failed to get vpn network Details")]
    GetVpnNetworkDetails(#[source] Box<HttpClientError>),

    #[error("failed to get user geolocation")]
    GetGeoIp(#[source] Box<HttpClientError>),

    #[error("failed to post account")]
    PostAccount(#[source] Box<HttpClientError>),

    #[error("failed to resolve hostname: {hostname}")]
    DnsResolutionFailure {
        hostname: String,
        source: nym_http_api_client::ResolveError,
    },

    #[error("Failed to resolve hostname(s): {hostnames:?}")]
    HostnamesResolutionError { hostnames: HashSet<String> },

    #[error("the url {url} doesn't parse to a host and/or a port: {reason}")]
    UrlError { url: url::Url, reason: String },

    #[error("the url {url} is invalid")]
    InvalidUrl { url: String },

    #[error("account api client error: {0}")]
    AccountError(#[source] Box<AccountError>),

    /// Error that happens when device is suspended or time travelling while receiving remote time
    /// Multiple attempts to receive remote time should happen before this error is returned
    #[error("too much time travel preventing receiving remote time")]
    TimeTravelTooMuch,

    #[error("payload error: {0}")]
    PayloadError(String),
}

pub type Result<T, E = VpnApiClientError> = std::result::Result<T, E>;

impl TryFrom<VpnApiClientError> for NymErrorResponse {
    type Error = VpnApiClientError;

    fn try_from(response: VpnApiClientError) -> std::result::Result<Self, Self::Error> {
        crate::response::extract_error_response(&response).ok_or(response)
    }
}
impl VpnApiClientError {
    pub fn http_client_error(&self) -> Option<&HttpClientError> {
        self.source()
            .and_then(|source| source.downcast_ref::<Box<HttpClientError>>())
            .map(|boxed| boxed.as_ref())
    }
}
