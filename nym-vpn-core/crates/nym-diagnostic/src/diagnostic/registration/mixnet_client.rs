// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only
use crate::diagnostic::{build_api_client, registration::setup_bandwidth_provider};

use nym_authenticator_client::{AuthClientMixnetListener, AuthenticatorClient, RegistrationError};
use nym_bandwidth_controller::BandwidthTicketProvider;
use nym_client_core::client::topology_control::nym_api_provider::Config;
use nym_credentials_interface::TicketType;
use nym_ip_packet_client::IprClientConnect;
use nym_platform_metadata::new_user_agent;
use nym_registration_common::WireguardConfiguration;
use nym_sdk::{
    DebugConfig, NymApiTopologyProvider, NymNetworkDetails, TopologyProvider,
    mixnet::{
        DisconnectedMixnetClient, Ephemeral, MixnetClient, MixnetClientBuilder, Recipient, x25519,
    },
};
use nym_topology::HardcodedTopologyProvider;
use nym_validator_client::{client::NymApiClientExt, models::described::v2::NymNodeDescriptionV2};
use nym_vpn_lib_types::{DiagnosticRegisterParams, DiagnosticResult, RegistrationReport};
use nym_vpn_network_config::Network;

use std::{net::IpAddr, path::PathBuf, sync::Arc, time::Duration};
use tokio_util::sync::CancellationToken;

struct WgRegistrationConfig {
    gateway_auth_address: Recipient,
    gateway_version: String,
    gateway_keypair: Arc<x25519::KeyPair>,
    gateway_ip: IpAddr,
    bandwidth_provider: Box<dyn BandwidthTicketProvider>,
}

pub(crate) struct MixnetClientRegistration;

impl MixnetClientRegistration {
    pub(crate) async fn register_with_mixnet_client(
        registration_report: &mut RegistrationReport,
        network: &Network,
        parameters: &DiagnosticRegisterParams,
    ) -> Option<(WireguardConfiguration, Arc<x25519::KeyPair>)> {
        let topology_provider = match setup_topology(network).await {
            Ok(provider) => provider,
            Err(e) => {
                registration_report.mixnet_client_build = DiagnosticResult::from_err(e);
                return None;
            }
        };

        tracing::info!("Starting mixnet client");

        let disconnected_mixnet_client = match Self::build_mixnet_client(
            network.nym_network_details().clone(),
            &parameters.gateway,
            Box::new(topology_provider),
        ) {
            Ok(client) => {
                registration_report.mixnet_client_build = DiagnosticResult::<()>::SUCCESS;
                client
            }
            Err(e) => {
                registration_report.mixnet_client_build = DiagnosticResult::from_err(e);
                return None;
            }
        };

        let mixnet_client = match Box::pin(tokio::time::timeout(
            Duration::from_secs(10),
            disconnected_mixnet_client.connect_to_mixnet(),
        ))
        .await
        {
            Ok(Ok(client)) => {
                registration_report.mixnet_client_start = Some(DiagnosticResult::<()>::SUCCESS);
                client
            }
            Ok(Err(e)) => {
                registration_report.mixnet_client_start = Some(DiagnosticResult::from_err(e));
                return None;
            }
            Err(e) => {
                registration_report.mixnet_client_start = Some(DiagnosticResult::from_err(e));
                return None;
            }
        };

        tracing::info!("Mixnet client started");

        let gateway = match describe_gateway(network, &parameters.gateway).await {
            Ok(g) => g,
            Err(e) => {
                let msg = format!("Gateway lookup failed: {e}");
                registration_report.mixnet_ipr_connect =
                    Some(DiagnosticResult::from_err(msg.clone()));
                registration_report.mixnet_based_dvpn_registration =
                    Some(DiagnosticResult::from_err(msg));
                mixnet_client.disconnect().await;
                return None;
            }
        };

        let mixnet_client = match lookup_ipr_address(&gateway) {
            Ok(address) => {
                tracing::info!("Connecting to IPR...");
                let (result, mixnet_client) =
                    Self::mixnet_ipr_connect(mixnet_client, address).await;
                match result {
                    Ok(()) => {
                        registration_report.mixnet_ipr_connect =
                            Some(DiagnosticResult::<()>::SUCCESS)
                    }
                    Err(e) => {
                        registration_report.mixnet_ipr_connect = Some(DiagnosticResult::from_err(
                            format!("IPR handshake failed: {e}"),
                        ))
                    }
                }
                mixnet_client
            }
            Err(e) => {
                registration_report.mixnet_ipr_connect = Some(DiagnosticResult::from_err(format!(
                    "IPR address lookup failed: {e}"
                )));
                mixnet_client
            }
        };

        let registration_config =
            match setup_wg_registration(&gateway, parameters.storage_path.as_ref()).await {
                Ok(config) => config,
                Err(e) => {
                    registration_report.mixnet_based_dvpn_registration = Some(
                        DiagnosticResult::from_err(format!("Registration not possible: {e}")),
                    );
                    mixnet_client.disconnect().await;
                    return None;
                }
            };

        tracing::info!("Registering...");

        let registration_result =
            Self::wireguard_registration(mixnet_client, &registration_config).await;
        // Explicitly close the credential storage before registration_config is dropped so
        // that the underlying SQLite pool releases OS file handles promptly (Windows).
        registration_config.bandwidth_provider.close().await;

        match registration_result {
            Ok(response) => {
                registration_report.mixnet_based_dvpn_registration =
                    Some(DiagnosticResult::from_value((&response).into()));
                Some((response, registration_config.gateway_keypair))
            }
            Err(e) => {
                registration_report.mixnet_based_dvpn_registration = Some(
                    DiagnosticResult::from_err(format!("Registration error: {e}")),
                );
                None
            }
        }
    }

    fn build_mixnet_client(
        network: NymNetworkDetails,
        gateway_id: &str,
        topology_provider: Box<dyn TopologyProvider + Send + Sync>,
    ) -> Result<DisconnectedMixnetClient<Ephemeral>, Box<nym_sdk::Error>> {
        let builder = MixnetClientBuilder::new_ephemeral()
            .with_user_agent(new_user_agent!())
            .request_gateway(gateway_id.into())
            .network_details(network)
            .debug_config(debug_config())
            .credentials_mode(false)
            .no_hostname(true)
            .custom_topology_provider(topology_provider);

        builder.build().map_err(Box::new)
    }

    async fn wireguard_registration(
        mixnet_client: MixnetClient,
        wg_registration_config: &WgRegistrationConfig,
    ) -> Result<WireguardConfiguration, RegistrationError> {
        let address = *mixnet_client.nym_address();

        let mixnet_listener =
            AuthClientMixnetListener::new(mixnet_client, CancellationToken::new()).start();
        let mut auth_client = AuthenticatorClient::new(
            mixnet_listener.subscribe(),
            mixnet_listener.mixnet_sender(),
            address,
            wg_registration_config.gateway_auth_address,
            wg_registration_config.gateway_version.clone().into(),
            wg_registration_config.gateway_keypair.clone(),
            wg_registration_config.gateway_ip,
        );

        // Embedded timeout
        let auth_res = auth_client
            .register_wireguard(
                &*wg_registration_config.bandwidth_provider,
                None,
                TicketType::V1WireguardEntry,
            )
            .await;

        // Stopping mixnet client
        mixnet_listener.stop().await;

        auth_res
    }

    async fn mixnet_ipr_connect(
        mixnet_client: MixnetClient,
        ipr_address: Recipient,
    ) -> (Result<(), nym_ip_packet_client::Error>, MixnetClient) {
        let mut ipr_client = IprClientConnect::new(mixnet_client, CancellationToken::new());

        let result = ipr_client.connect(ipr_address).await;

        (result.map(|_| ()), ipr_client.into_mixnet_client())
    }
}

fn debug_config() -> DebugConfig {
    let mut debug_config = DebugConfig::default();

    debug_config.traffic.average_packet_delay = Duration::ZERO;
    debug_config.traffic.disable_mix_hops = true;
    debug_config
        .traffic
        .disable_main_poisson_packet_distribution = true;
    debug_config.cover_traffic.disable_loop_cover_traffic_stream = true;
    debug_config.topology.minimum_mixnode_performance = 0;
    debug_config.topology.minimum_gateway_performance = 0;
    debug_config
}

async fn setup_topology(network: &Network) -> anyhow::Result<HardcodedTopologyProvider> {
    let api_client = build_api_client(network).await?;
    const DEFAULT_CONFIG: Config = Config {
        min_mixnode_performance: 0,
        min_gateway_performance: 0,
        use_extended_topology: true,
        ignore_egress_epoch_role: true,
    };

    let base_urls: Vec<url::Url> = api_client
        .base_urls()
        .iter()
        .cloned()
        .map(Into::into)
        .collect();

    let mut topology_provider = NymApiTopologyProvider::new(DEFAULT_CONFIG, base_urls, api_client);

    let topology = topology_provider
        .get_new_topology()
        .await
        .ok_or(anyhow::anyhow!("Failed to get topology"))?;

    Ok(HardcodedTopologyProvider::new(topology))
}

async fn describe_gateway(
    network: &Network,
    gateway_id: &str,
) -> anyhow::Result<NymNodeDescriptionV2> {
    let api_client = build_api_client(network).await?;

    let described_nodes = api_client
        .get_all_described_nodes_v2()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch described nodes : {e}"))?;

    described_nodes
        .into_iter()
        .find(|g| g.ed25519_identity_key().to_base58_string() == gateway_id)
        .ok_or(anyhow::anyhow!("Gateway requested not found"))
}

fn lookup_ipr_address(gateway: &NymNodeDescriptionV2) -> anyhow::Result<Recipient> {
    gateway
        .description
        .ip_packet_router
        .as_ref()
        .and_then(|a| Recipient::try_from_base58_string(&a.address).ok())
        .ok_or(anyhow::anyhow!(
            "Failed to get IPR address for chosen gateway",
        ))
}

async fn setup_wg_registration(
    gateway: &NymNodeDescriptionV2,
    storage_path: Option<&PathBuf>,
) -> anyhow::Result<WgRegistrationConfig> {
    let storage_path = storage_path.ok_or(anyhow::anyhow!("No storage path provided"))?;

    let gateway_keypair = Arc::new(x25519::KeyPair::new(&mut rand::rngs::OsRng));

    let gateway_version = gateway.version().to_string();
    let authenticator_address = gateway
        .description
        .authenticator
        .as_ref()
        .and_then(|a| Recipient::try_from_base58_string(&a.address).ok())
        .ok_or(anyhow::anyhow!(
            "Failed to get authenticator address for chosen gateway",
        ))?;
    let gateway_ip = *gateway
        .description
        .host_information
        .ip_address
        .first()
        .ok_or(anyhow::anyhow!(
            "Chosen gateway does not have announced IP addresses",
        ))?;

    let bandwidth_provider = setup_bandwidth_provider(storage_path)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to setup bandwidth provider : {e}"))?;

    Ok(WgRegistrationConfig {
        gateway_auth_address: authenticator_address,
        gateway_version,
        gateway_keypair,
        gateway_ip,
        bandwidth_provider,
    })
}
