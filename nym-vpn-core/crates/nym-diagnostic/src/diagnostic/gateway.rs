// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::error::{Error, Result};

use nym_crypto::asymmetric::{ed25519, x25519};
use nym_gateway_directory::{Gateway, GatewayClient};
use nym_lp::{Ciphersuite, peer::LpRemotePeer};
use nym_lp_data::packet::version;
use nym_platform_metadata::new_user_agent;
use nym_registration_client::LpRegistrationClient;
use nym_vpn_lib_types::{DiagnosticResult, GatewayReport};
use nym_vpn_network_config::Network;

use futures::{SinkExt, StreamExt};
use std::{
    net::{IpAddr, SocketAddr},
    sync::Arc,
};
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
            lp_handshake: None,
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

        match Self::lp_handshake_test(&gateway, ip).await {
            Ok(()) => {
                gateway_report.lp_handshake = Some(DiagnosticResult::<()>::SUCCESS);
            }
            Err(e) => {
                gateway_report.lp_handshake = Some(DiagnosticResult::from_err(e));
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
                gateway_report.websocket_request = Some(DiagnosticResult::from_value(response));
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

        GatewayClient::new(config, new_user_agent!())
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

    async fn lp_handshake_test(gateway: &Gateway, gateway_ip: IpAddr) -> anyhow::Result<()> {
        let gateway_id_key = gateway.identity();

        let gateway_lp_data = gateway
            .lewes_protocol_details
            .clone()
            .ok_or(anyhow::anyhow!(
                "Node doesn't have published LP data : {}",
                gateway_id_key.to_base58_string()
            ))?;

        if !gateway_lp_data.verify(&gateway_id_key) {
            return Err(anyhow::anyhow!(
                "Node's lp data does not pass signature check : {gateway_id_key}"
            ));
        }

        let gateway_version = match gateway
            .version
            .as_ref()
            .map(|version| semver::Version::parse(version))
        {
            Some(Ok(version)) => version,
            Some(Err(e)) => Err(anyhow::anyhow!("Invalid provided version : {e}"))?,
            None => Err(anyhow::anyhow!(
                "No provided version, cannot infer information for LP handshake"
            ))?,
        };
        let lp_ciphersuite = Ciphersuite::from_node_version(gateway_version.clone()).ok_or(anyhow::anyhow!("Node is announcing LP information, but its provided version doesn't support it : {gateway_version}"))?;

        let gateway_lp_address = SocketAddr::new(gateway_ip, gateway_lp_data.content.control_port);

        tracing::debug!("Entry gateway LP address: {gateway_lp_address}");

        let gateway_lp_peer = LpRemotePeer::new(gateway_lp_data.content.x25519).with_key_digests(
            gateway_lp_data
                .content
                .kem_keys()
                .map_err(|e| anyhow::anyhow!("Incorrect kem key digests : {e}"))?,
        );

        let dh_keypair = x25519::DHKeyPair::new(&mut rand10::rng());
        let mut lp_client = LpRegistrationClient::<TcpStream>::new_with_default_config(
            Arc::new(dh_keypair),
            gateway_lp_peer.clone(),
            gateway_lp_address,
            lp_ciphersuite,
            version::CURRENT,
        );

        Ok(lp_client.perform_handshake().await?)
    }
}
