// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{diagnostic::build_api_client, error::Result};

use nym_validator_client::nym_api::NymApiClientExt;
use nym_vpn_lib::new_user_agent;
use nym_vpn_lib_types::{ApiTimeSkew, HttpReport};
use nym_vpn_network_config::Network;

pub struct HttpDiagnostic;

impl HttpDiagnostic {
    pub async fn run_diagnostic(network: &Network) -> Result<HttpReport> {
        tracing::info!("Running http diagnostic");
        let nym_vpn_api_client = nym_vpn_api_client::VpnApiClient::from_network(
            network.nym_network_details(),
            Some(new_user_agent!()),
            None,
        )
        .await?;

        let api_client = build_api_client(network).await?;

        // Setup is done, we return a report from now on

        let health_response = nym_vpn_api_client.get_health().await.map(Into::into);
        let remote_time = nym_vpn_api_client
            .get_remote_time()
            .await
            .map(ApiTimeSkew::from);

        let nb_nodes = api_client
            .get_all_described_nodes()
            .await
            .map(|list| list.len());

        Ok(HttpReport {
            remote_time: remote_time.into(),
            health_response: health_response.into(),
            nb_nymnodes: nb_nodes.into(),
        })
    }
}
