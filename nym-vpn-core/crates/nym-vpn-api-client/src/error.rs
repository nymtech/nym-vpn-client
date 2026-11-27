// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::error::Error;

use itertools::Itertools;

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
#[error("failed to resolve {hostname}")]
pub struct DnsResolutionFailure {
    pub hostname: String,
    #[source]
    pub source: nym_http_api_client::ResolveError,
}

#[derive(Debug, thiserror::Error)]
pub enum VpnApiClientError {
    #[error("http client error")]
    HttpClient(#[source] Box<HttpClientError>),

    #[error(
        "failed to resolve hostname(s): {}",
        failures.iter().map(|failure| failure.hostname.as_str()).join(", ")
    )]
    DnsResolution { failures: Vec<DnsResolutionFailure> },

    #[error("invalid url {url}: {reason}")]
    InvalidUrl { url: String, reason: String },

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

    /// The request was valid, but the account was never registered with the API. See
    /// [`NymErrorResponse::is_account_not_found`].
    pub fn is_account_not_found(&self) -> bool {
        crate::response::extract_error_response(self).is_some_and(|e| e.is_account_not_found())
    }

    /// The API refused the request because too many were sent.
    pub fn is_rate_limited(&self) -> bool {
        self.http_client_error()
            .and_then(HttpClientError::status_code)
            .is_some_and(|status| status.as_u16() == 429)
    }
}
