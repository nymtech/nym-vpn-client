// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    diagnostic::build_api_client,
    error::{Error, Result},
};

use std::fmt;

use nym_http_api_client::{Client, ClientBuilder, FrontPolicy, Url};
use nym_platform_metadata::new_user_agent;
use nym_validator_client::nym_api::NymApiClientExt;
use nym_vpn_api_client::{VpnApiClient, api_urls_to_urls};
use nym_vpn_lib_types::{
    ApiTimeSkew, ApiUrl, DiagnosticEndpointResponse, DiagnosticResult, HttpReport,
};
use nym_vpn_network_config::Network;

#[derive(Debug, Clone)]
struct HttpDiagnosticError<E> {
    error: E,
    url: ApiUrl,
}

impl<E: fmt::Debug> fmt::Display for HttpDiagnosticError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "http diagnostic error for {:?}: {:?}",
            self.url, self.error
        )
    }
}

pub struct HttpDiagnostic;

impl HttpDiagnostic {
    /// Run a series of short tests to check that APIs are reachable.
    ///
    /// For the `by_endpoint` tests if there is a front defined in the output
    /// it means that the FrontPolicy was set to always and fronting was used.
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

        let mut http_report = Vec::new();
        http_report.extend_from_slice(&Self::test_nym_apis(network, false).await?);
        http_report.extend_from_slice(&Self::test_vpn_apis(network, false).await?);

        Ok(HttpReport {
            remote_time: remote_time.into(),
            health_response: health_response.into(),
            nb_nymnodes: nb_nodes.into(),
            by_endpoint: http_report,
        })
    }

    async fn test_nym_apis(
        network: &Network,
        parallel: bool,
    ) -> Result<Vec<DiagnosticResult<DiagnosticEndpointResponse>>> {
        tracing::info!("Running Nym api diagnostics");

        let api_clients = build_nym_api_clients(network).await?;
        let mut results = Vec::new();

        if parallel {
            let mut jobs = tokio::task::JoinSet::new();

            for api_client in api_clients {
                jobs.spawn(async move {
                    let url = url_from_client(&api_client);

                    match api_client.health().await {
                        Ok(res) => DiagnosticResult::from_value(DiagnosticEndpointResponse {
                            status: res.status.is_up().to_string(),
                            url: url_from_client(&api_client),
                        }),
                        Err(e) => {
                            let err = HttpDiagnosticError { url, error: e };
                            DiagnosticResult::from_err(err)
                        }
                    }
                });
            }

            jobs.join_all()
                .await
                .into_iter()
                .for_each(|r| results.push(r));
        } else {
            for api_client in api_clients {
                let url = url_from_client(&api_client);
                match api_client.health().await {
                    Ok(res) => {
                        results.push(DiagnosticResult::from_value(DiagnosticEndpointResponse {
                            status: res.status.is_up().to_string(),
                            url,
                        }))
                    }
                    Err(e) => {
                        let err = HttpDiagnosticError { url, error: e };
                        results.push(DiagnosticResult::from_err(err))
                    }
                }
            }
        }

        Ok(results)
    }

    async fn test_vpn_apis(
        network: &Network,
        parallel: bool,
    ) -> Result<Vec<DiagnosticResult<DiagnosticEndpointResponse>>> {
        tracing::info!("Running VPN api diagnostics");

        let api_clients = build_vpn_api_clients(network).await?;
        let mut results = Vec::new();

        if parallel {
            let mut jobs = tokio::task::JoinSet::new();

            for api_client in api_clients {
                jobs.spawn(async move {
                    let url = url_from_client(api_client.as_ref());
                    match api_client.get_health().await {
                        Ok(res) => DiagnosticResult::from_value(DiagnosticEndpointResponse {
                            status: res.status,
                            url: url_from_client(api_client.as_ref()),
                        }),
                        Err(e) => {
                            let err = HttpDiagnosticError { url, error: e };
                            DiagnosticResult::from_err(err)
                        }
                    }
                });
            }

            jobs.join_all()
                .await
                .into_iter()
                .for_each(|r| results.push(r));
        } else {
            for api_client in api_clients {
                let url = url_from_client(api_client.as_ref());
                match api_client.get_health().await {
                    Ok(res) => {
                        results.push(DiagnosticResult::from_value(DiagnosticEndpointResponse {
                            status: res.status,
                            url: url_from_client(api_client.as_ref()),
                        }))
                    }
                    Err(e) => {
                        let err = HttpDiagnosticError { url, error: e };
                        results.push(DiagnosticResult::from_err(err))
                    }
                }
            }
        }

        Ok(results)
    }
}

fn url_from_client(client: &Client) -> ApiUrl {
    ApiUrl {
        url: client.current_url().inner_url().to_string(),
        front_hosts: client
            .current_url()
            .fronts()
            .map(|v| v.iter().map(ToString::to_string).collect()),
    }
}

async fn build_nym_api_clients(network: &Network) -> Result<Vec<Client>> {
    let nym_urls = api_urls_to_urls(&network.nym_api_urls())?;
    if nym_urls.is_empty() {
        return Err(Error::MissingApiUrl);
    }
    
    let mut clients = Vec::new();
    for url in nym_urls {
        let plain_url =
            Url::new(url.inner_url().clone(), None).map_err(|_| Error::MissingApiUrl)?;
        let plain_client = ClientBuilder::new(plain_url)
            .map_err(|_| Error::MissingApiUrl)?
            .no_hickory_dns()
            .with_user_agent(new_user_agent!())
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
                    .with_user_agent(new_user_agent!())
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
    let nym_urls = api_urls_to_urls(&network.nym_vpn_api_urls())?;
    if nym_urls.is_empty() {
        return Err(Error::MissingApiUrl);
    }

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
