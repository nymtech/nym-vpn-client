use std::time::Duration;

use nym_http_api_client::UserAgent;
use tracing::debug;

use crate::{
    error::Result,
    responses::{Country, Gateway},
    Client, ClientBuilder, VpnApiClientExt,
};

const NYM_VPN_API: &str = "https://nymvpn.com/api";

fn client_with_user_agent(user_agent: UserAgent) -> Result<Client> {
    ClientBuilder::new(NYM_VPN_API)?
        .with_timeout(Duration::from_secs(10))
        .with_user_agent(user_agent)
        .build()
        .map_err(Into::into)
}

pub async fn get_gateways(user_agent: UserAgent) -> Result<Vec<Gateway>> {
    debug!("Fetching gateways");
    client_with_user_agent(user_agent)?
        .get_gateways()
        .await
        .map_err(Into::into)
}

pub async fn get_entry_gateways(user_agent: UserAgent) -> Result<Vec<Gateway>> {
    debug!("Fetching entry gateways");
    client_with_user_agent(user_agent)?
        .get_entry_gateways()
        .await
        .map_err(Into::into)
}

pub async fn get_exit_gateways(user_agent: UserAgent) -> Result<Vec<Gateway>> {
    debug!("Fetching exit gateways");
    client_with_user_agent(user_agent)?
        .get_exit_gateways()
        .await
        .map_err(Into::into)
}

pub async fn get_countries(user_agent: UserAgent) -> Result<Vec<Country>> {
    debug!("Fetching countries");
    client_with_user_agent(user_agent)?
        .get_countries()
        .await
        .map_err(Into::into)
}

pub async fn get_entry_countries(user_agent: UserAgent) -> Result<Vec<Country>> {
    debug!("Fetching entry countries");
    client_with_user_agent(user_agent)?
        .get_entry_countries()
        .await
        .map_err(Into::into)
}

pub async fn get_exit_countries() -> Result<Vec<Country>> {
    debug!("Fetching exit countries");
    client_with_user_agent(user_agent)?
        .get_exit_countries()
        .await
        .map_err(Into::into)
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
