mod client;
mod error;
mod helpers;
pub mod responses;
mod routes;

pub use client::{Client, ClientBuilder, VpnApiClientExt, VpnApiError};
pub use error::VpnApiClientError;
pub use helpers::{
    client_with_user_agent, get_countries, get_entry_countries, get_entry_gateways,
    get_exit_countries, get_exit_gateways, get_gateways,
};
pub use responses::{Country, Gateway, Location};
