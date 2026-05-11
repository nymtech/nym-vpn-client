// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    diagnostic::build_api_client,
    error::{Error, Result},
};

use nym_http_api_client::{Client, ClientBuilder, FrontPolicy, Url};
use nym_platform_metadata::new_user_agent;
use nym_validator_client::nym_api::NymApiClientExt;
use nym_vpn_api_client::VpnApiClient;
use nym_vpn_api_client::api_urls_to_urls;
use nym_vpn_lib_types::{ApiTimeSkew, HttpReport};
use nym_vpn_network_config::Network;

pub struct HttpDiagnostic;

impl HttpDiagnostic {
    pub async fn run_diagnostic(network: &Network) -> Result<HttpReport> {
        tracing::info!("Running http diagnostic");
        let nym_vpn_api_client =
            VpnApiClient::from_network(network.nym_network_details(), Some(new_user_agent!()))
                .await?;

        let api_client = build_api_client(network).await?;

        // Setup is done, we return a report from now on

        let health_response = nym_vpn_api_client.get_health().await.map(Into::into);
        let remote_time = nym_vpn_api_client
            .get_remote_time()
            .await
            .map(ApiTimeSkew::from);

        let nb_nodes = api_client
            .get_all_described_nodes_v2()
            .await
            .map(|list| list.len());

        Self::test_nym_apis(network, false).await?;
        Self::test_vpn_apis(network, false).await?;

        Ok(HttpReport {
            remote_time: remote_time.into(),
            health_response: health_response.into(),
            nb_nymnodes: nb_nodes.into(),
        })
    }

    async fn test_nym_apis(network: &Network, parallel: bool) -> Result<()> {
        tracing::info!("nym apis");

        let api_clients = build_nym_api_clients(network).await?;

        if parallel {
            let mut jobs = tokio::task::JoinSet::new();

            for api_client in api_clients {
                jobs.spawn(async move {
                    let _ = api_client.get_network_details().await;
                });
            }

            while jobs.join_next().await.is_some() {}
        } else {
            for api_client in api_clients {
                let _ = api_client.get_network_details().await;
            }
        }

        Ok(())
    }

    async fn test_vpn_apis(network: &Network, parallel: bool) -> Result<()> {
        tracing::info!("vpn apis");
        let api_clients = build_vpn_api_clients(network).await?;

        if parallel {
            let mut jobs = tokio::task::JoinSet::new();

            for api_client in api_clients {
                jobs.spawn(async move {
                    let _ = api_client.get_health().await;
                });
            }

            while jobs.join_next().await.is_some() {}
        } else {
            for api_client in api_clients {
                let _ = api_client.get_health().await;
            }
        }

        Ok(())
    }
}

async fn build_nym_api_clients(network: &Network) -> Result<Vec<Client>> {
    let nym_urls = api_urls_to_urls(&network.nym_api_urls().ok_or(Error::MissingApiUrl)?)?;
    let mut clients = Vec::new();
    for url in nym_urls {
        let plain_url =
            Url::new(url.inner_url().clone(), None).map_err(|_| Error::MissingApiUrl)?;
        let plain_client = ClientBuilder::new(plain_url)
            .map_err(|_| Error::MissingApiUrl)?
            .no_hickory_dns()
            .with_retries(0)
            .build()
            .map_err(|_| Error::MissingApiUrl)?;

        clients.push(plain_client);

        if let Some(fronts) = url.fronts() {
            for front in fronts {
                let fronted_url = Url::new(url.inner_url().clone(), Some(vec![front.clone()]))
                    .map_err(|_e| Error::MissingApiUrl)?;

                let fronted_client = ClientBuilder::new(fronted_url)
                    .map_err(|_| Error::MissingApiUrl)?
                    .no_hickory_dns()
                    .with_fronting(Some(FrontPolicy::Always))
                    .with_retries(0)
                    .build()
                    .map_err(|_| Error::MissingApiUrl)?;

                clients.push(fronted_client);
            }
        }
    }
    Ok(clients)
}

async fn build_vpn_api_clients(network: &Network) -> Result<Vec<VpnApiClient>> {
    let nym_urls = api_urls_to_urls(&network.nym_vpn_api_urls().ok_or(Error::MissingApiUrl)?)?;
    let mut clients = Vec::new();

    for url in nym_urls {
        let plain_url =
            Url::new(url.inner_url().clone(), None).map_err(|_| Error::MissingApiUrl)?;
        let mut plain_client = VpnApiClient::new(vec![plain_url], Some(new_user_agent!()))
            .map_err(|_| Error::MissingApiUrl)?;

        plain_client.as_mut().set_front_policy(FrontPolicy::Off);

        clients.push(plain_client);

        if let Some(fronts) = url.fronts() {
            for front in fronts {
                let fronted_url = Url::new(url.inner_url().clone(), Some(vec![front.clone()]))
                    .map_err(|_e| Error::MissingApiUrl)?;

                let mut fronted_client =
                    VpnApiClient::new(vec![fronted_url], Some(new_user_agent!()))
                        .map_err(|_| Error::MissingApiUrl)?;
                fronted_client
                    .as_mut()
                    .set_front_policy(FrontPolicy::Always);

                clients.push(fronted_client);
            }
        }
    }

    Ok(clients)
}
