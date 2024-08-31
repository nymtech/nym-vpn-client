pub mod responses;

mod client;
mod error;
pub(crate) mod headers;
// mod helpers;
pub(crate) mod jwt;
mod request;
mod routes;
pub mod types;

pub use client::VpnApiClient;
pub use error::VpnApiClientError;
// pub use helpers::{
//     get_countries, get_entry_countries, get_entry_gateways, get_exit_countries, get_exit_gateways,
//     get_gateways,
// };
pub use types::{Country, Gateway, Location};

pub type VpnApiHttpErrorResponse =
    nym_http_api_client::HttpClientError<responses::NymErrorResponse>;
pub type VpnApiHttpUnexpectedError =
    nym_http_api_client::HttpClientError<responses::UnexpectedError>;
