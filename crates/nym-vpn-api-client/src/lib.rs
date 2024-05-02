mod client;
mod error;
mod helpers;
mod responses;
mod routes;

pub use client::{Client, VpnApiClientExt, VpnApiError};
pub use error::VpnApiClientError;
pub use helpers::get_gateways;
pub use responses::Gateway;
