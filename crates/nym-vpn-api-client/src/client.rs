use nym_http_api_client::{ApiClient, HttpClientError, NO_PARAMS};

pub use nym_http_api_client::Client;
use tracing::debug;

use crate::{responses::Gateway, routes};

pub type VpnApiError = HttpClientError;

const PAGINATION_KEY: &str = "size";
const PAGINATION_SIZE: &str = "500";
const PAGINATION: (&str, &str) = (PAGINATION_KEY, PAGINATION_SIZE);

#[allow(async_fn_in_trait)]
pub trait VpnApiClientExt: ApiClient {
    async fn get_gateways(&self) -> Result<Vec<Gateway>, VpnApiError> {
        debug!("Fetching gateways");
        self.get_json(&[routes::DIRECTORY, routes::GATEWAYS], NO_PARAMS)
            .await
    }

    // async fn get_gateways(&self) -> Result<Vec<Gateway>, VpnApiError> {
    //     debug!("Fetching gateways");
    //     let mut gateways = Vec::new();
    //     let mut page = 0;
    //     loop {
    //         let result = self.get_gateways_page(page).await.inspect_err(|e| {
    //             error!("Failed to fetch gateways: {}", e);
    //         })?;
    //         debug!(
    //             "Got page={}, size={}, total={}, items.len()={}",
    //             result.page,
    //             result.size,
    //             result.total,
    //             result.items.len(),
    //         );
    //         gateways.extend(result.items);
    //         if gateways.len() >= result.total as usize {
    //             break;
    //         }
    //         page += 1;
    //         if page > 10 {
    //             error!("Too many pages fetched. Stopping.");
    //             break;
    //         }
    //     }
    //     Ok(gateways)
    // }
}

impl VpnApiClientExt for Client {}
