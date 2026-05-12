// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Reachability check for Google's CTAP 2.2 Hybrid Transport relay
//! (`cable.ua5v.com`).
//!
//! Opens a TLS connection on port 443 and runs a WebSocket upgrade with
//! `Sec-WebSocket-Protocol: fido.cable` on `/cable/new/<tunnel_id>`. The
//! socket is closed as soon as the upgrade response arrives — we don't
//! consume a tunnel slot.
//!
//! Success requires both HTTP 101 *and* an `X-Cable-Routing-Id` response
//! header. 101 alone is not enough: Apple's relay (and any MITM that
//! terminates TLS and completes a WS upgrade) returns 101 too. The
//! routing-id header is what proves we reached a working Google bridging
//! backend.

use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use nym_vpn_lib_types::HybridTransportReport;
use rand::RngCore;
use rustls::{ClientConfig, RootCertStore, pki_types::ServerName};
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;
use tokio_tungstenite::tungstenite::{handshake::client::generate_key, http::Request};

const RELAY_HOST: &str = "cable.ua5v.com";
const WS_PATH_PREFIX: &str = "/cable";
const WS_SUBPROTOCOL: &str = "fido.cable";
const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(10);

pub struct HybridTransportDiagnostic;

impl HybridTransportDiagnostic {
    pub async fn run_diagnostic() -> Result<HybridTransportReport, String> {
        tracing::info!("Running hybrid transport diagnostic");

        let tls_config = build_tls_config()?;
        match tokio::time::timeout(HANDSHAKE_TIMEOUT, probe(tls_config)).await {
            Ok(result) => result,
            Err(_) => Err(format!("handshake timed out after {HANDSHAKE_TIMEOUT:?}")),
        }
    }
}

fn build_tls_config() -> Result<Arc<ClientConfig>, String> {
    let root_store = RootCertStore {
        roots: webpki_roots::TLS_SERVER_ROOTS.to_vec(),
    };
    let provider = rustls::crypto::CryptoProvider::get_default()
        .cloned()
        .unwrap_or_else(|| Arc::new(rustls::crypto::ring::default_provider()));
    let config = ClientConfig::builder_with_provider(provider)
        .with_safe_default_protocol_versions()
        .map_err(|e| format!("tls protocol versions: {e}"))?
        .with_root_certificates(root_store)
        .with_no_client_auth();
    Ok(Arc::new(config))
}

async fn probe(tls_config: Arc<ClientConfig>) -> Result<HybridTransportReport, String> {
    let tunnel_id = random_tunnel_id();
    let path = format!("{WS_PATH_PREFIX}/new/{tunnel_id}");

    tracing::debug!("hybrid transport probe -> {RELAY_HOST}{path} (tunnel-id {tunnel_id})");

    let start = Instant::now();

    let tcp = TcpStream::connect((RELAY_HOST, 443))
        .await
        .map_err(|e| format!("tcp connect: {e}"))?;

    let server_name = ServerName::try_from(RELAY_HOST)
        .map_err(|e| format!("invalid server name: {e}"))?
        .to_owned();
    let tls = TlsConnector::from(tls_config)
        .connect(server_name, tcp)
        .await
        .map_err(|e| format!("tls handshake: {e}"))?;

    let request = Request::builder()
        .uri(format!("wss://{RELAY_HOST}{path}"))
        .header("Host", RELAY_HOST)
        .header("Connection", "Upgrade")
        .header("Upgrade", "websocket")
        .header("Sec-WebSocket-Version", "13")
        .header("Sec-WebSocket-Key", generate_key())
        .header("Sec-WebSocket-Protocol", WS_SUBPROTOCOL)
        .body(())
        .map_err(|e| format!("build ws request: {e}"))?;

    let (_ws, response) = tokio_tungstenite::client_async(request, tls)
        .await
        .map_err(|e| format!("ws upgrade: {e}"))?;

    let handshake_duration_ms = start.elapsed().as_millis();
    let routing_id = response
        .headers()
        .get("X-Cable-Routing-Id")
        .and_then(|v| v.to_str().ok())
        .map(str::to_owned)
        .ok_or("missing X-Cable-Routing-Id header (101 alone is not proof of a working backend)")?;

    tracing::info!(
        "hybrid transport probe {RELAY_HOST} -> {} routing-id={routing_id} ({handshake_duration_ms}ms)",
        response.status()
    );

    Ok(HybridTransportReport {
        routing_id,
        handshake_duration_ms,
    })
}

fn random_tunnel_id() -> String {
    let mut bytes = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut bytes);
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
