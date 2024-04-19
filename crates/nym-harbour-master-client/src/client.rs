use nym_http_api_client::{ApiClient, HttpClientError, NO_PARAMS};

pub use nym_http_api_client::Client;

use crate::{
    responses::{Gateway, PagedResult},
    routes,
};

// This is largely lifted from mix-fetch. The future of harbourmaster is uncertain, but ideally
// these two should be merged so they both can depend on the same crate.

pub type HarbourMasterApiError = HttpClientError;

#[allow(async_fn_in_trait)]
pub trait HarbourMasterApiClientExt: ApiClient {
    // TODO: paging
    async fn get_gateways(&self) -> Result<PagedResult<Gateway>, HarbourMasterApiError> {
        self.get_json(&[routes::API_VERSION, routes::GATEWAYS], NO_PARAMS)
            .await
    }
}

impl HarbourMasterApiClientExt for Client {}
