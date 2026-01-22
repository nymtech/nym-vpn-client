// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::error::{Error, Result};

use nym_crypto::asymmetric::ed25519;
use nym_gateway_directory::{Gateway, GatewayClient};
use nym_vpn_lib::new_user_agent;
use nym_vpn_lib_types::{DiagnosticResult, GatewayReport};
use nym_vpn_network_config::Network;

use futures::{SinkExt, StreamExt};
use std::net::{IpAddr, SocketAddr};
use tokio::net::{TcpSocket, TcpStream};
use tokio_tungstenite::{WebSocketStream, tungstenite::http::Response};

pub struct GatewayDiagnostic;

impl GatewayDiagnostic {
    pub async fn run_diagnostic(network: &Network, gateway_id: &str) -> GatewayReport {
        tracing::info!("Running gateway diagnostic on {}", gateway_id);

        let mut gateway_report = GatewayReport {
            gateway: DiagnosticResult::from_value(None),
            tcp: None,
            websocket: None,
            websocket_request: None,
        };

        let gateway_id_key = if let Ok(key) = ed25519::PublicKey::from_base58_string(gateway_id) {
            key
        } else {
            gateway_report.gateway = DiagnosticResult::from_err("Invalid gateway key");
            return gateway_report;
        };

        let gateway_client = match Self::setup_gateway_client(network).await {
            Ok(client) => client,
            Err(e) => {
                gateway_report.gateway =
                    DiagnosticResult::from_err(format!("Gateway client setup {e}"));
                return gateway_report;
            }
        };

        // Setup is done, we return a report from now on

        let gateway = match Self::lookup_gateway(&gateway_client, &gateway_id_key).await {
            Ok(maybe_gateway) => {
                gateway_report.gateway =
                    DiagnosticResult::from_value(maybe_gateway.clone().map(Into::into));
                match maybe_gateway {
                    Some(gateway) => gateway,
                    None => return gateway_report,
                }
            }
            Err(e) => {
                gateway_report.gateway = DiagnosticResult::from_err(e);
                return gateway_report;
            }
        };

        // Check that gateway has an IP and a port before proceeding
        let (ip, port) = match (gateway.lookup_ip(), gateway.clients_ws_port) {
            (Some(ip), Some(port)) => (ip, port),
            (_, _) => {
                gateway_report.tcp = Some(DiagnosticResult::from_err(
                    "Missing IP address or websocket port",
                ));
                return gateway_report;
            }
        };

        let tcp_stream = match Self::tcp_connection_test(ip, port).await {
            Ok(stream) => {
                gateway_report.tcp = Some(DiagnosticResult::<()>::SUCCESS);
                stream
            }
            Err(e) => {
                gateway_report.tcp = Some(DiagnosticResult::from_err(e));
                return gateway_report;
            }
        };

        let ws_stream = match Self::ws_connection_test(ip, port, tcp_stream).await {
            Ok((stream, _)) => {
                gateway_report.websocket = Some(DiagnosticResult::<()>::SUCCESS);
                stream
            }
            Err(e) => {
                gateway_report.websocket = Some(DiagnosticResult::from_err(e));
                return gateway_report;
            }
        };

        match Self::ws_request_test(ws_stream).await {
            Ok(response) => {
                gateway_report.websocket_request = Some(DiagnosticResult::from_value(response))
            }
            Err(e) => {
                gateway_report.websocket_request = Some(DiagnosticResult::from_err(e));
            }
        };

        gateway_report
    }

    async fn setup_gateway_client(
        network: &Network,
    ) -> Result<GatewayClient, nym_gateway_directory::Error> {
        let nyxd_url = network.nyxd_url();

        // If they were None, config creation will catch the error. And they shouldn't be None anyway
        let nym_api_urls = network.nym_api_urls().unwrap_or_default();
        let nym_vpn_api_urls = network.nym_vpn_api_urls().unwrap_or_default();

        let config = nym_gateway_directory::Config::new(
            nyxd_url,
            nym_api_urls.clone(),
            nym_vpn_api_urls,
            None,
        )?;

        nym_gateway_directory::GatewayClient::new(config, new_user_agent!()).await
    }

    async fn lookup_gateway(
        gateway_client: &GatewayClient,
        gateway_id: &ed25519::PublicKey,
    ) -> Result<Option<Gateway>, nym_gateway_directory::Error> {
        let nym_nodes = gateway_client.lookup_all_nymnodes().await?;

        Ok(nym_nodes.gateway_with_identity(gateway_id).cloned())
    }

    async fn tcp_connection_test(ip: IpAddr, port: u16) -> Result<TcpStream, std::io::Error> {
        let socket_addr = SocketAddr::new(ip, port);

        let socket = if socket_addr.is_ipv4() {
            TcpSocket::new_v4()
        } else {
            TcpSocket::new_v6()
        }?;

        socket.connect(socket_addr).await
    }

    async fn ws_connection_test(
        ip: IpAddr,
        port: u16,
        tcp_stream: TcpStream,
    ) -> Result<(WebSocketStream<TcpStream>, Response<Option<Vec<u8>>>)> {
        let endpoint = format!("ws://{ip}:{port}");
        Ok(tokio_tungstenite::client_async(endpoint, tcp_stream).await?)
    }

    async fn ws_request_test(mut ws_stream: WebSocketStream<TcpStream>) -> Result<String> {
        // At the time of writing, this is a request that can be sent without authentication
        let client_request = nym_gateway_requests::ClientControlRequest::SupportedProtocol {};

        // From<ClientControlRequest> for Message uses this, but some reason rustc doesn't want to know about it
        #[allow(clippy::unwrap_used)]
        let ws_request = serde_json::to_string(&client_request).unwrap().into();

        ws_stream.send(ws_request).await?;
        let response = ws_stream.next().await.ok_or(Error::WsStreamClosed)??;

        Ok(response.to_text()?.into())
    }
}
