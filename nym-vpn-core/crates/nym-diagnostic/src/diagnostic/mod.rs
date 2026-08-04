// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::{
    diagnostic::{
        dns::DnsDiagnostic, gateway::GatewayDiagnostic, http::HttpDiagnostic,
        hybrid_transport::HybridTransportDiagnostic, registration::RegistrationDiagnostic,
    },
    error::{Error, Result},
};

use nym_http_api_client::Client;
use nym_vpn_api_client::{api_urls_to_urls, fronted_http_client};
use nym_vpn_lib_types::{
    DiagnosticRegisterParams, DiagnosticReport, DiagnosticRunParams, RegistrationReport,
};
use nym_vpn_network_config::Network;

mod dns;
mod gateway;
mod http;
mod hybrid_transport;
mod registration;

pub struct DiagnosticHandler;

impl DiagnosticHandler {
    pub async fn run(network: Network, parameters: DiagnosticRunParams) -> DiagnosticReport {
        let dns_report = if !parameters.skip_dns {
            Some(DnsDiagnostic::run_diagnostic(&network).await)
        } else {
            None
        };

        let http_report = if !parameters.skip_http {
            Some(
                HttpDiagnostic::run_diagnostic(&network)
                    .await
                    .inspect_err(|e| tracing::error!("Http diagnostic error : {}", e.to_string())),
            )
        } else {
            None
        };

        let gateway_report = match parameters.gateway {
            Some(id) => Some(GatewayDiagnostic::run_diagnostic(&network, &id).await),
            None => None,
        };

        let hybrid_transport_report = if !parameters.skip_hybrid_transport {
            Some(HybridTransportDiagnostic::run_diagnostic().await)
        } else {
            None
        };

        DiagnosticReport {
            dns: dns_report,
            http: http_report.map(Into::into),
            gateway: gateway_report,
            hybrid_transport: hybrid_transport_report.map(Into::into),
        }
    }

    pub async fn register(
        network: Network,
        parameters: DiagnosticRegisterParams,
    ) -> RegistrationReport {
        RegistrationDiagnostic::run_diagnostic(&network, &parameters).await
    }
}

pub async fn build_api_client(network: &Network) -> Result<Client> {
    let nym_urls = api_urls_to_urls(&network.nym_api_urls())?;
    if nym_urls.is_empty() {
        return Err(Error::MissingApiUrl);
    }

    Ok(fronted_http_client(nym_urls, None, None)?)
}
