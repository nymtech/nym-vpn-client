use nym_http_api_client::{ApiClient, HttpClientError, NO_PARAMS};

pub use nym_http_api_client::Client;
use tracing::debug;

use crate::{responses::Gateway, routes};

pub type VpnApiError = HttpClientError;

#[allow(async_fn_in_trait)]
pub trait VpnApiClientExt: ApiClient {
    async fn get_gateways(&self) -> Result<Vec<Gateway>, VpnApiError> {
        debug!("Fetching gateways");
        self.get_json(&[routes::DIRECTORY, routes::GATEWAYS], NO_PARAMS)
            .await
    }
}

impl VpnApiClientExt for Client {}
