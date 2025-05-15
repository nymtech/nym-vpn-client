use tracing::debug;

use crate::{Client, HarbourMasterApiClientExt, error::Result, responses::Gateway};

const HARBOUR_MASTER: &str = "https://harbourmaster.nymtech.net";

pub async fn get_gateways() -> Result<Vec<Gateway>> {
    debug!("Fetching gateways");
    let timeout = std::time::Duration::from_secs(10);
    let client = Client::new_url(HARBOUR_MASTER, Some(timeout))?;
    Ok(client.get_gateways().await?)
}
