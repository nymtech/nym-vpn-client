use crate::{
    error::Result,
    responses::{Gateway, PagedResult},
    Client, HarbourMasterApiClientExt,
};

const HARBOUR_MASTER: &str = "https://harbourmaster.nymtech.net";

pub async fn get_gateways() -> Result<PagedResult<Gateway>> {
    let client = Client::new_url(HARBOUR_MASTER, None)?;
    Ok(client.get_gateways().await?)
}
