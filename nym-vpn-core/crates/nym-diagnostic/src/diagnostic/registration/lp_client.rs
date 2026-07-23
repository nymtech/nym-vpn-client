// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::diagnostic::{build_api_client, registration::setup_bandwidth_provider};

use anyhow::Context;
use nym_bandwidth_controller::{BandwidthTicketProvider, SystemSpendTimeProvider};
use nym_credentials_interface::TicketType;
use nym_lp::{Ciphersuite, peer::LpRemotePeer};
use nym_lp_data::packet::version;
use nym_registration_client::LpRegistrationClient;
use nym_registration_common::WireguardConfiguration;
use nym_sdk::mixnet::{ed25519, x25519};
use nym_validator_client::client::NymApiClientExt;
use nym_vpn_lib_types::{DiagnosticRegisterParams, DiagnosticResult, RegistrationReport};
use nym_vpn_network_config::Network;
use rand09::SeedableRng;
use std::{net::SocketAddr, path::PathBuf, sync::Arc};
use tokio::net::TcpStream;

struct WgRegistrationConfig {
    local_wg_keypair: Arc<x25519::KeyPair>,
    gateway_id_key: ed25519::PublicKey,
    bandwidth_provider: Box<dyn BandwidthTicketProvider>,
}

pub(crate) struct LpClientRegistration;

impl LpClientRegistration {
    pub(crate) async fn register_with_lp(
        registration_report: &mut RegistrationReport,
        network: &Network,
        parameters: &DiagnosticRegisterParams,
    ) -> Option<(WireguardConfiguration, Arc<x25519::KeyPair>)> {
        tracing::info!("Starting LP regsitration");

        let (registration_config, mut lp_client) = match setup_registration(
            network,
            &parameters.gateway,
            parameters.storage_path.as_ref(),
        )
        .await
        {
            Ok(config) => config,
            Err(e) => {
                registration_report.lp_based_dvpn_registration = Some(DiagnosticResult::from_err(
                    format!("Failed to setup LP registration: {e}"),
                ));
                return None;
            }
        };
        tracing::info!("LP Handshake...");
        // Perform handshake with gateway
        if let Err(e) = lp_client.perform_handshake().await {
            registration_report.lp_handshake = Some(DiagnosticResult::from_err(e));
            // Close credential storage before early return so OS file handles are released promptly.
            registration_config.bandwidth_provider.close().await;
            return None;
        } else {
            registration_report.lp_handshake = Some(DiagnosticResult::<()>::SUCCESS)
        }

        // dVPN registration
        tracing::info!("Registering with entry gateway");
        let dvpn_result = lp_client
            .register_dvpn(
                &mut rand09::rngs::StdRng::from_os_rng(),
                &registration_config.local_wg_keypair,
                &registration_config.gateway_id_key,
                &registration_config.bandwidth_provider,
                &SystemSpendTimeProvider,
                TicketType::V1WireguardEntry,
            )
            .await;

        // Explicitly close the credential storage before registration_config is dropped so
        // that the underlying SQLite pool releases OS file handles promptly (Windows).
        registration_config.bandwidth_provider.close().await;

        match dvpn_result {
            Ok(response) => {
                registration_report.lp_based_dvpn_registration =
                    Some(DiagnosticResult::from_value((&response).into()));
                Some((response, registration_config.local_wg_keypair))
            }
            Err(e) => {
                registration_report.lp_based_dvpn_registration =
                    Some(DiagnosticResult::from_err(e));
                None
            }
        }
    }
}

async fn setup_registration(
    network: &Network,
    gateway_id: &str,
    storage_path: Option<&PathBuf>,
) -> anyhow::Result<(WgRegistrationConfig, LpRegistrationClient)> {
    let storage_path = storage_path.ok_or(anyhow::anyhow!("No storage path provided"))?;

    let api_client = build_api_client(network).await?;

    let described_nodes = api_client
        .get_all_described_nodes_v2()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch described nodes : {e}"))?;
    let gateway = described_nodes
        .iter()
        .find(|g| g.ed25519_identity_key().to_base58_string() == gateway_id)
        .ok_or(anyhow::anyhow!("Gateway requested not found"))?
        .clone();

    let local_wg_keypair = Arc::new(x25519::KeyPair::new(&mut rand::rngs::OsRng));
    let local_dh_keypair = Arc::new(x25519::DHKeyPair::new(
        &mut rand09::rngs::StdRng::from_os_rng(),
    ));

    let gateway_ip = *gateway
        .description
        .host_information
        .ip_address
        .first()
        .ok_or(anyhow::anyhow!(
            "Chosen gateway does not have announced IP addresses",
        ))?;
    let gateway_id_key = gateway.ed25519_identity_key();

    // Extract and validate LP data
    let gateway_lp_data = gateway
        .description
        .lewes_protocol
        .ok_or(anyhow::anyhow!(
            "Node doesn't have published LP data : {gateway_id}"
        ))?
        .content;

    let gateway_version =
        semver::Version::parse(&gateway.description.build_information.build_version)
            .context("Invalid provided version : {e}")?;

    let lp_ciphersuite = Ciphersuite::from_node_version(gateway_version.clone()).ok_or(anyhow::anyhow!("Node is announcing LP information, but its provided version doesn't support it : {gateway_version}"))?;

    let gateway_lp_address = SocketAddr::new(gateway_ip, gateway_lp_data.control_port);

    tracing::debug!("Entry gateway LP address: {gateway_lp_address}");

    let gateway_lp_peer = LpRemotePeer::new(gateway_lp_data.x25519).with_key_digests(
        gateway_lp_data
            .kem_keys()
            .map_err(|e| anyhow::anyhow!("Incorrect kem key digests : {e}"))?,
    );

    let bandwidth_provider = setup_bandwidth_provider(storage_path)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to setup bandwidth provider : {e}"))?;

    let lp_client = LpRegistrationClient::<TcpStream>::new_with_default_config(
        local_dh_keypair,
        gateway_lp_peer.clone(),
        gateway_lp_address,
        lp_ciphersuite,
        version::CURRENT,
    );

    Ok((
        WgRegistrationConfig {
            local_wg_keypair,
            gateway_id_key,
            bandwidth_provider,
        },
        lp_client,
    ))
}
