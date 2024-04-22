use nym_http_api_client::{ApiClient, HttpClientError};

pub use nym_http_api_client::Client;
use tracing::{debug, error};

use crate::{
    responses::{Gateway, PagedResult},
    routes,
};

// This is largely lifted from mix-fetch. The future of harbourmaster is uncertain, but ideally
// these two should be merged so they both can depend on the same crate.

pub type HarbourMasterApiError = HttpClientError;

const PAGINATION_KEY: &str = "size";
const PAGINATION_SIZE: &str = "500";
const PAGINATION: (&str, &str) = (PAGINATION_KEY, PAGINATION_SIZE);

#[allow(async_fn_in_trait)]
pub trait HarbourMasterApiClientExt: ApiClient {
    async fn get_gateways_page(
        &self,
        page: u32,
    ) -> Result<PagedResult<Gateway>, HarbourMasterApiError> {
        debug!("Fetching gateways page {}", page);
        self.get_json(
            &[routes::API_VERSION, routes::GATEWAYS],
            &[PAGINATION, (("page"), (&page.to_string()))],
        )
        .await
    }

    // BEWARE: pagination isn't yet implemented in the harbourmaster API as far as I can tell, so
    // just set the pagination size large enough to cover it in one go for now.
    async fn get_gateways(&self) -> Result<Vec<Gateway>, HarbourMasterApiError> {
        debug!("Fetching gateways");
        let mut gateways = Vec::new();
        let mut page = 0;
        loop {
            let result = self.get_gateways_page(page).await?;
            debug!(
                "Got page={}, size={}, total={}, items.len()={}",
                result.page,
                result.size,
                result.total,
                result.items.len(),
            );
            gateways.extend(result.items);
            if gateways.len() >= result.total as usize {
                break;
            }
            page += 1;
            if page > 10 {
                error!("Too many pages fetched. Stopping.");
                break;
            }
        }
        Ok(gateways)
    }
}

impl HarbourMasterApiClientExt for Client {}
