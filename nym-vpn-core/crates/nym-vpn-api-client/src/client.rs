use nym_http_api_client::{ApiClient, HttpClientError, NO_PARAMS};

pub use nym_http_api_client::{Client, ClientBuilder};
use tracing::debug;

use crate::{
    responses::{Country, Gateway},
    routes,
};

pub type VpnApiError = HttpClientError;

#[allow(async_fn_in_trait)]
pub trait VpnApiClientExt: ApiClient {
    async fn get_gateways(&self) -> Result<Vec<Gateway>, VpnApiError> {
        debug!("Fetching gateways");
        self.get_json(&[routes::DIRECTORY, routes::GATEWAYS], NO_PARAMS)
            .await
    }

    async fn get_entry_gateways(&self) -> Result<Vec<Gateway>, VpnApiError> {
        debug!("Fetching entry gateways");
        self.get_json(
            &[routes::DIRECTORY, routes::GATEWAYS, routes::ENTRY],
            NO_PARAMS,
        )
        .await
    }

    async fn get_exit_gateways(&self) -> Result<Vec<Gateway>, VpnApiError> {
        debug!("Fetching exit gateways");
        self.get_json(
            &[routes::DIRECTORY, routes::GATEWAYS, routes::EXIT],
            NO_PARAMS,
        )
        .await
    }

    async fn get_countries(&self) -> Result<Vec<Country>, VpnApiError> {
        debug!("Fetching countries");
        self.get_json(
            &[routes::DIRECTORY, routes::GATEWAYS, routes::COUNTRIES],
            NO_PARAMS,
        )
        .await
    }

    async fn get_entry_countries(&self) -> Result<Vec<Country>, VpnApiError> {
        debug!("Fetching entry countries");
        self.get_json(
            &[
                routes::DIRECTORY,
                routes::GATEWAYS,
                routes::ENTRY,
                routes::COUNTRIES,
            ],
            NO_PARAMS,
        )
        .await
    }

    async fn get_exit_countries(&self) -> Result<Vec<Country>, VpnApiError> {
        debug!("Fetching exit countries");
        self.get_json(
            &[
                routes::DIRECTORY,
                routes::GATEWAYS,
                routes::EXIT,
                routes::COUNTRIES,
            ],
            NO_PARAMS,
        )
        .await
    }
}

impl VpnApiClientExt for Client {}
