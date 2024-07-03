use nym_http_api_client::UserAgent;
use tracing::debug;

use crate::{
    error::Result,
    responses::{Country, Gateway},
    Client, ClientBuilder, VpnApiClientExt,
};

const NYM_VPN_API: &str = "https://nymvpn.com/api";

fn get_client_with_user_agent(user_agent: UserAgent) -> Result<Client> {
    let timeout = std::time::Duration::from_secs(10);
    ClientBuilder::new(NYM_VPN_API)?
        .with_timeout(timeout)
        .with_user_agent(user_agent)
        .build()
        .map_err(Into::into)
}

pub async fn get_gateways(user_agent: UserAgent) -> Result<Vec<Gateway>> {
    debug!("Fetching gateways");
    Ok(get_client_with_user_agent(user_agent)?
        .get_gateways()
        .await?)
}

pub async fn get_entry_gateways(user_agent: UserAgent) -> Result<Vec<Gateway>> {
    debug!("Fetching entry gateways");
    let timeout = std::time::Duration::from_secs(10);
    let client = ClientBuilder::new(NYM_VPN_API)?
        .with_timeout(timeout)
        .with_user_agent(user_agent)
        .build()?;
    Ok(client.get_entry_gateways().await?)
}

pub async fn get_exit_gateways(user_agent: UserAgent) -> Result<Vec<Gateway>> {
    debug!("Fetching exit gateways");
    let timeout = std::time::Duration::from_secs(10);
    let client = ClientBuilder::new(NYM_VPN_API)?
        .with_timeout(timeout)
        .with_user_agent(user_agent)
        .build()?;
    Ok(client.get_exit_gateways().await?)
}

pub async fn get_countries(user_agent: UserAgent) -> Result<Vec<Country>> {
    debug!("Fetching countries");
    let timeout = std::time::Duration::from_secs(10);
    let client = ClientBuilder::new(NYM_VPN_API)?
        .with_timeout(timeout)
        .with_user_agent(user_agent)
        .build()?;
    Ok(client.get_countries().await?)
}

pub async fn get_entry_countries(user_agent: UserAgent) -> Result<Vec<Country>> {
    debug!("Fetching entry countries");
    let timeout = std::time::Duration::from_secs(10);
    let client = ClientBuilder::new(NYM_VPN_API)?
        .with_timeout(timeout)
        .with_user_agent(user_agent)
        .build()?;
    Ok(client.get_entry_countries().await?)
}

pub async fn get_exit_countries() -> Result<Vec<Country>> {
    debug!("Fetching exit countries");
    let timeout = std::time::Duration::from_secs(10);
    let client = Client::new_url(NYM_VPN_API, Some(timeout))?;
    Ok(client.get_exit_countries().await?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_gateways() {
        let user_agent = UserAgent::new("nym-vpn-api-client", "arch", "test", "test");
        let gateways = get_gateways().await.unwrap();
        assert!(!gateways.is_empty());
    }

    #[tokio::test]
    async fn test_get_entry_gateways() {
        let gateways = get_entry_gateways().await.unwrap();
        assert!(!gateways.is_empty());
    }

    #[tokio::test]
    async fn test_get_exit_gateways() {
        let gateways = get_exit_gateways().await.unwrap();
        assert!(!gateways.is_empty());
    }

    #[tokio::test]
    async fn test_get_countries() {
        let countries = get_countries().await.unwrap();
        assert!(!countries.is_empty());
    }

    #[tokio::test]
    async fn test_get_entry_countries() {
        let countries = get_entry_countries().await.unwrap();
        assert!(!countries.is_empty());
    }

    #[tokio::test]
    async fn test_get_exit_countries() {
        let countries = get_exit_countries().await.unwrap();
        assert!(!countries.is_empty());
    }
}
