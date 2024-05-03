use tracing::debug;

use crate::{error::Result, responses::Gateway, Client, VpnApiClientExt};

const NYM_VPN_API: &str = "https://nymvpn.com/api";

pub async fn get_gateways() -> Result<Vec<Gateway>> {
    debug!("Fetching gateways");
    let timeout = std::time::Duration::from_secs(10);
    let client = Client::new_url(NYM_VPN_API, Some(timeout))?;
    Ok(client.get_gateways().await?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_gateways() {
        let gateways = get_gateways().await.unwrap();
        assert!(!gateways.is_empty());
    }
}
