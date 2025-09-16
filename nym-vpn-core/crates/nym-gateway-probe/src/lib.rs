// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    time::Duration,
};

use crate::{netstack::NetstackResult, types::Entry};
use anyhow::{Context, anyhow, bail};
use base64::{Engine as _, engine::general_purpose};
use bytes::BytesMut;
use clap::Args;
use futures::StreamExt;
use nym_authenticator_client::{AuthClientMixnetListener, AuthenticatorClient};
use nym_authenticator_requests::{
    AuthenticatorVersion, client_message::ClientMessage, response::AuthenticatorResponse, v2, v3,
    v4, v5,
};
use nym_bandwidth_controller::error::BandwidthControllerError;
use nym_client_core::config::ForgetMe;
use nym_config::defaults::{
    NymNetworkDetails,
    mixnet_vpn::{NYM_TUN_DEVICE_ADDRESS_V4, NYM_TUN_DEVICE_ADDRESS_V6},
};
use nym_connection_monitor::self_ping_and_wait;
use nym_credentials_interface::{CredentialSpendingData, TicketType};
use nym_gateway_directory::{
    AuthAddress, Config as GatewayDirectoryConfig, EntryPoint,
    GatewayClient as GatewayDirectoryClient, GatewayList, GatewayMinPerformance,
    IpPacketRouterAddress, NymNode, Recipient,
};
use nym_ip_packet_client::IprClientConnect;
use nym_ip_packet_requests::{
    IpPair,
    codec::MultiIpPacketCodec,
    v8::response::{
        ControlResponse, DataResponse, InfoLevel, IpPacketResponse, IpPacketResponseData,
    },
};
use nym_sdk::mixnet::{
    DisconnectedMixnetClient, Ephemeral, MixnetClient, MixnetClientBuilder, MixnetClientStorage,
    NodeIdentity, ReconstructedMessage,
};
use nym_validator_client::nyxd::error::NyxdError;
use nym_wireguard_types::PeerPublicKey;
use tokio_util::{codec::Decoder, sync::CancellationToken};
use tracing::*;
use types::WgProbeResults;

use crate::{
    icmp::{check_for_icmp_beacon_reply, icmp_identifier, send_ping_v4, send_ping_v6},
    types::Exit,
};

use netstack::{NetstackRequest, NetstackRequestGo};

mod error;
mod icmp;
mod netstack;
mod types;

pub use error::{Error, Result};
pub use types::{IpPingReplies, ProbeOutcome, ProbeResult};

#[derive(Args)]
pub struct NetstackArgs {
    #[arg(long, default_value_t = 180)]
    netstack_download_timeout_sec: u64,

    #[arg(long, default_value = "1.1.1.1")]
    netstack_v4_dns: String,

    #[arg(long, default_value = "2606:4700:4700::1111")]
    netstack_v6_dns: String,

    #[arg(long, default_value_t = 5)]
    netstack_num_ping: u8,

    #[arg(long, default_value_t = 3)]
    netstack_send_timeout_sec: u64,

    #[arg(long, default_value_t = 3)]
    netstack_recv_timeout_sec: u64,

    #[arg(long, default_values_t = vec!["nymtech.net".to_string()])]
    netstack_ping_hosts_v4: Vec<String>,

    #[arg(long, default_values_t = vec!["1.1.1.1".to_string()])]
    netstack_ping_ips_v4: Vec<String>,

    #[arg(long, default_values_t = vec!["ipv6.google.com".to_string()])]
    netstack_ping_hosts_v6: Vec<String>,

    #[arg(long, default_values_t = vec!["2001:4860:4860::8888".to_string(), "2606:4700:4700::1111".to_string(), "2620:fe::fe".to_string()])]
    netstack_ping_ips_v6: Vec<String>,
}

#[derive(Args)]
pub struct CredentialArgs {
    #[arg(long)]
    mnemonic: String,
}

#[derive(Default, Debug)]
pub enum TestedNode {
    #[default]
    SameAsEntry,
    Custom {
        identity: NodeIdentity,
    },
}

impl TestedNode {
    pub fn is_same_as_entry(&self) -> bool {
        matches!(self, TestedNode::SameAsEntry)
    }
}

#[derive(Debug)]
pub struct TestedNodeDetails {
    identity: NodeIdentity,
    exit_router_address: Option<IpPacketRouterAddress>,
    authenticator_address: Option<AuthAddress>,
    authenticator_version: AuthenticatorVersion,
    ip_address: Option<IpAddr>,
}

impl From<&NymNode> for TestedNodeDetails {
    fn from(node: &NymNode) -> Self {
        TestedNodeDetails {
            identity: node.identity,
            exit_router_address: node.ipr_address,
            authenticator_address: node.authenticator_address,
            authenticator_version: AuthenticatorVersion::from(node.version.as_ref()),
            ip_address: node.ips.first().copied(),
        }
    }
}

/// Obtain nym-node for testing
pub async fn get_nym_node(
    config: GatewayDirectoryConfig,
    identity: NodeIdentity,
) -> anyhow::Result<NymNode> {
    let user_agent = nym_bin_common::bin_info_local_vergen!().into();
    let nodes_client = GatewayDirectoryClient::new(config, user_agent)?;
    let nodes = nodes_client.lookup_all_nymnodes().await?;
    let node = nodes
        .node_with_identity(&identity)
        .ok_or_else(|| anyhow!("did not find the specified node"))?;
    Ok(node.clone())
}

pub async fn fetch_gateways(gateway_config: GatewayDirectoryConfig) -> anyhow::Result<GatewayList> {
    lookup_gateways(gateway_config).await
}

pub async fn fetch_gateways_with_ipr(
    gateway_config: GatewayDirectoryConfig,
) -> anyhow::Result<GatewayList> {
    Ok(lookup_gateways(gateway_config).await?.into_exit_gateways())
}

pub struct Probe {
    entrypoint: EntryPoint,
    tested_node: TestedNode,
    amnezia_args: String,
    netstack_args: NetstackArgs,
    credentials_args: CredentialArgs,
}

impl Probe {
    pub fn new(
        entrypoint: EntryPoint,
        tested_node: TestedNode,
        netstack_args: NetstackArgs,
        credentials_args: CredentialArgs,
    ) -> Self {
        Self {
            entrypoint,
            tested_node,
            amnezia_args: "".into(),
            netstack_args,
            credentials_args,
        }
    }
    pub fn with_amnezia(&mut self, args: &str) -> &Self {
        self.amnezia_args = args.to_string();
        self
    }

    async fn acquire_bandwidth(
        &self,
        disconnected_mixnet_client: &DisconnectedMixnetClient<Ephemeral>,
        ticketbook_type: TicketType,
    ) -> anyhow::Result<()> {
        // TODO: make it configurable
        const MAX_RETRIES: usize = 50;
        for i in 0..MAX_RETRIES {
            let attempt = i + 1; // since humans usually don't count from 0 in this instance
            info!(
                "attempt {attempt}/{MAX_RETRIES} for attempting to acquire {ticketbook_type} bandwidth"
            );
            let bw_client = disconnected_mixnet_client
                .create_bandwidth_client(self.credentials_args.mnemonic.clone(), ticketbook_type)
                .await?;
            info!("Calling bandwidth controller acquire() for {ticketbook_type}");
            match bw_client.acquire().await {
                Ok(_) => {
                    if i > 0 {
                        info!(
                            "managed to acquire {ticketbook_type} bandwidth after {attempt} attempts",
                        );
                    }
                    return Ok(());
                }
                Err(nym_sdk::Error::CredentialIssuanceError { source }) => match source {
                    nym_credential_utils::errors::Error::BandwidthControllerError(
                        BandwidthControllerError::Nyxd(nyxd_error),
                    ) => match nyxd_error {
                        // happens when sequence issue occurs during tx delivery
                        NyxdError::BroadcastTxErrorDeliverTx {
                            hash,
                            height,
                            code,
                            raw_log,
                        } => {
                            // unfortunately at this point we have to do string matching as the log
                            // is returned from the go nyxd binary
                            if raw_log.contains("account sequence mismatch") {
                                error!(
                                    "another process is using the same mnemonic. we failed to broadcast transaction {hash} due to mismatched sequence number"
                                )
                            } else {
                                return Err(NyxdError::BroadcastTxErrorDeliverTx {
                                    hash,
                                    height,
                                    code,
                                    raw_log,
                                }
                                .into());
                            }
                        }
                        // happens when sequence issue occurs during tx simulate
                        NyxdError::AbciError {
                            code,
                            log,
                            pretty_log,
                        } => {
                            // unfortunately at this point we have to do string matching as the log
                            // is returned from the go nyxd binary
                            if log.contains("account sequence mismatch") {
                                error!(
                                    "another process is using the same mnemonic. we failed to simulate transaction due to mismatched sequence number"
                                )
                            } else {
                                return Err(NyxdError::AbciError {
                                    code,
                                    log,
                                    pretty_log,
                                }
                                .into());
                            }
                        }
                        other => {
                            return Err(other)
                                .context("another nyxd failure during bandwidth acquisition");
                        }
                    },
                    other => {
                        return Err(other.into());
                    }
                },
                Err(other) => {
                    return Err(other.into());
                }
            }

            // add a bit of backoff as if the rpc node is slightly out of sync,
            // we might use our retry budget for abci queries to the simulate endpoint
            tokio::time::sleep(Duration::from_secs(1)).await;
        }

        bail!("failed to acquire bandwidth after {MAX_RETRIES} attempts")
    }

    pub async fn probe(
        self,
        gateway_config: GatewayDirectoryConfig,
        ignore_egress_epoch_role: bool,
        only_wireguard: bool,
    ) -> anyhow::Result<ProbeResult> {
        // Setup the entry gateways
        let gateways = lookup_gateways(gateway_config.clone()).await?;

        let entry_gateway = self.entrypoint.lookup_gateway(&gateways).await?;
        let tested_entry = self.tested_node.is_same_as_entry();

        let node_info: TestedNodeDetails = match self.tested_node {
            TestedNode::Custom { identity } => {
                let node = get_nym_node(gateway_config.clone(), identity).await?;
                info!(
                    "testing node {} (via entry {})",
                    node.identity, entry_gateway.identity
                );
                (&node).into()
            }
            TestedNode::SameAsEntry => (&entry_gateway).into(),
        };

        let mixnet_entry_gateway_id = entry_gateway.identity();

        info!("connecting to entry gateway: {entry_gateway:?}");
        debug!(
            "authenticator version: {:?}",
            node_info.authenticator_version
        );

        let storage = Ephemeral::default();

        // Connect to the mixnet via the entry gateway
        let disconnected_mixnet_client = MixnetClientBuilder::new_with_storage(storage.clone())
            .request_gateway(mixnet_entry_gateway_id.to_string())
            .network_details(NymNetworkDetails::new_from_env())
            .debug_config(mixnet_debug_config(
                gateway_config.min_gateway_performance,
                ignore_egress_epoch_role,
            ))
            .with_forget_me(ForgetMe::new_all())
            .credentials_mode(true)
            .build()?;

        for ticketbook_type in [
            TicketType::V1MixnetEntry,
            TicketType::V1WireguardEntry,
            TicketType::V1WireguardExit,
        ] {
            self.acquire_bandwidth(&disconnected_mixnet_client, ticketbook_type)
                .await?;
        }

        let mixnet_client = Box::pin(disconnected_mixnet_client.connect_to_mixnet()).await;

        let mixnet_client = match mixnet_client {
            Ok(mixnet_client) => mixnet_client,
            Err(err) => {
                error!("Failed to connect to mixnet: {err}");
                return Ok(ProbeResult {
                    node: node_info.identity.to_string(),
                    used_entry: mixnet_entry_gateway_id.to_string(),
                    outcome: ProbeOutcome {
                        as_entry: if tested_entry {
                            Entry::fail_to_connect()
                        } else {
                            Entry::EntryFailure
                        },
                        as_exit: None,
                        wg: None,
                    },
                });
            }
        };

        let nym_address = *mixnet_client.nym_address();
        let entry_gateway = nym_address.gateway().to_base58_string();

        info!("Successfully connected to entry gateway: {entry_gateway}");
        info!("Our nym address: {nym_address}");

        // Now that we have a connected mixnet client, we can start pinging
        let (outcome, mixnet_client) = if only_wireguard {
            (
                Ok(ProbeOutcome {
                    as_entry: if tested_entry {
                        Entry::success()
                    } else {
                        Entry::NotTested
                    },
                    as_exit: None,
                    wg: None,
                }),
                mixnet_client,
            )
        } else {
            do_ping(
                mixnet_client,
                nym_address,
                node_info.exit_router_address,
                tested_entry,
            )
            .await
        };

        let wg_outcome = if let (Some(authenticator), Some(ip_address)) =
            (node_info.authenticator_address, node_info.ip_address)
        {
            let cancel_token = CancellationToken::new();
            // Start the mixnet listener that the auth clients use to receive messages.
            let mixnet_listener_task =
                AuthClientMixnetListener::new(mixnet_client, cancel_token.child_token()).start();

            let auth_client = AuthenticatorClient::new_entry(
                &None,
                mixnet_listener_task.subscribe(),
                mixnet_listener_task.mixnet_sender(),
                nym_address,
                authenticator.into(),
                node_info.authenticator_version,
                ip_address,
            );
            let config = nym_validator_client::nyxd::Config::try_from_nym_network_details(
                &nym_config::defaults::NymNetworkDetails::new_from_env(),
            )?;
            let client = nym_validator_client::nyxd::NyxdClient::connect(
                config,
                gateway_config.nyxd_url().as_str(),
            )?;
            let bw_controller = nym_bandwidth_controller::BandwidthController::new(
                storage.credential_store().clone(),
                client,
            );
            let credential = bw_controller
                .prepare_ecash_ticket(
                    TicketType::V1WireguardEntry,
                    nym_address.gateway().to_bytes(),
                    1,
                )
                .await?
                .data;

            let outcome = wg_probe(
                auth_client,
                ip_address,
                node_info.authenticator_version,
                self.amnezia_args,
                self.netstack_args,
                credential,
            )
            .await
            .unwrap_or_default();

            cancel_token.cancel();
            let mixnet_client = mixnet_listener_task.wait().await?;
            mixnet_client.disconnect().await;

            outcome
        } else {
            mixnet_client.disconnect().await;
            WgProbeResults::default()
        };

        // Disconnect the mixnet client gracefully
        outcome.map(|mut outcome| {
            outcome.wg = Some(wg_outcome);
            ProbeResult {
                node: node_info.identity.to_string(),
                used_entry: mixnet_entry_gateway_id.to_string(),
                outcome,
            }
        })
    }
}

async fn wg_probe(
    mut auth_client: AuthenticatorClient,
    gateway_ip: IpAddr,
    auth_version: AuthenticatorVersion,
    awg_args: String,
    netstack_args: NetstackArgs,
    credential: CredentialSpendingData,
) -> anyhow::Result<WgProbeResults> {
    info!("attempting to use authenticator version {auth_version:?}");

    let mut rng = rand::thread_rng();
    let private_key = nym_crypto::asymmetric::encryption::PrivateKey::new(&mut rng);
    let public_key = private_key.public_key();

    let authenticator_pub_key = PeerPublicKey::new(public_key.to_bytes().into());
    let init_message = match auth_version {
        AuthenticatorVersion::V2 => ClientMessage::Initial(Box::new(
            v2::registration::InitMessage::new(authenticator_pub_key),
        )),
        AuthenticatorVersion::V3 => ClientMessage::Initial(Box::new(
            v3::registration::InitMessage::new(authenticator_pub_key),
        )),
        AuthenticatorVersion::V4 => ClientMessage::Initial(Box::new(
            v4::registration::InitMessage::new(authenticator_pub_key),
        )),
        AuthenticatorVersion::V5 => ClientMessage::Initial(Box::new(
            v5::registration::InitMessage::new(authenticator_pub_key),
        )),
        AuthenticatorVersion::V1 | AuthenticatorVersion::UNKNOWN => bail!("unknown version number"),
    };

    let mut wg_outcome = WgProbeResults::default();

    info!(
        "connecting to authenticator: {}...",
        auth_client.auth_recipient
    );
    let response = auth_client
        .send_and_wait_for_response(&init_message)
        .await?;

    let registered_data = match response {
        AuthenticatorResponse::PendingRegistration(pending_registration_response) => {
            // Unwrap since we have already checked that we have the keypair.
            debug!("Verifying data");
            pending_registration_response.verify(&private_key)?;

            let finalized_message = match auth_version {
                AuthenticatorVersion::V2 => {
                    ClientMessage::Final(Box::new(v2::registration::FinalMessage {
                        gateway_client: v2::registration::GatewayClient::new(
                            &private_key,
                            pending_registration_response.pub_key().inner(),
                            pending_registration_response.private_ips().ipv4.into(),
                            pending_registration_response.nonce(),
                        ),
                        credential: Some(credential),
                    }))
                }
                AuthenticatorVersion::V3 => {
                    ClientMessage::Final(Box::new(v3::registration::FinalMessage {
                        gateway_client: v3::registration::GatewayClient::new(
                            &private_key,
                            pending_registration_response.pub_key().inner(),
                            pending_registration_response.private_ips().ipv4.into(),
                            pending_registration_response.nonce(),
                        ),
                        credential: Some(credential),
                    }))
                }
                AuthenticatorVersion::V4 => {
                    ClientMessage::Final(Box::new(v4::registration::FinalMessage {
                        gateway_client: v4::registration::GatewayClient::new(
                            &private_key,
                            pending_registration_response.pub_key().inner(),
                            pending_registration_response.private_ips().into(),
                            pending_registration_response.nonce(),
                        ),
                        credential: Some(credential),
                    }))
                }
                AuthenticatorVersion::V5 => {
                    ClientMessage::Final(Box::new(v5::registration::FinalMessage {
                        gateway_client: v5::registration::GatewayClient::new(
                            &private_key,
                            pending_registration_response.pub_key().inner(),
                            pending_registration_response.private_ips(),
                            pending_registration_response.nonce(),
                        ),
                        credential: Some(credential),
                    }))
                }
                AuthenticatorVersion::V1 | AuthenticatorVersion::UNKNOWN => {
                    bail!("Unknown version number")
                }
            };
            let response = auth_client
                .send_and_wait_for_response(&finalized_message)
                .await?;
            let AuthenticatorResponse::Registered(registered_response) = response else {
                bail!("Unexpected response");
            };
            registered_response
        }
        AuthenticatorResponse::Registered(registered_response) => registered_response,
        _ => bail!("Unexpected response"),
    };

    let peer_public = registered_data.pub_key().inner();
    let static_private = x25519_dalek::StaticSecret::from(private_key.to_bytes());
    let public_key_bs64 = general_purpose::STANDARD.encode(peer_public.as_bytes());
    let private_key_hex = hex::encode(static_private.to_bytes());
    let public_key_hex = hex::encode(peer_public.as_bytes());

    info!("WG connection details");
    info!("Peer public key: {}", public_key_bs64);
    info!(
        "ips {}(v4) {}(v6), port {}",
        registered_data.private_ips().ipv4,
        registered_data.private_ips().ipv6,
        registered_data.wg_port(),
    );

    let wg_endpoint = format!("{gateway_ip}:{}", registered_data.wg_port());

    info!("Successfully registered with the gateway");

    wg_outcome.can_register = true;

    if wg_outcome.can_register {
        let netstack_request = NetstackRequest::new(
            &registered_data.private_ips().ipv4.to_string(),
            &registered_data.private_ips().ipv6.to_string(),
            &private_key_hex,
            &public_key_hex,
            &wg_endpoint,
            netstack_args.netstack_download_timeout_sec,
            &awg_args,
            netstack_args,
        );

        // Perform IPv4 ping test
        let ipv4_request = NetstackRequestGo::from_rust_v4(&netstack_request);

        match netstack::ping(&ipv4_request) {
            Ok(NetstackResult::Response(netstack_response_v4)) => {
                info!(
                    "Wireguard probe response for IPv4: {:#?}",
                    netstack_response_v4
                );
                wg_outcome.can_handshake_v4 = netstack_response_v4.can_handshake;
                wg_outcome.can_resolve_dns_v4 = netstack_response_v4.can_resolve_dns;
                wg_outcome.ping_hosts_performance_v4 = netstack_response_v4.received_hosts as f32
                    / netstack_response_v4.sent_hosts as f32;
                wg_outcome.ping_ips_performance_v4 =
                    netstack_response_v4.received_ips as f32 / netstack_response_v4.sent_ips as f32;

                wg_outcome.download_duration_sec_v4 = netstack_response_v4.download_duration_sec;
                wg_outcome.downloaded_file_v4 = netstack_response_v4.downloaded_file;
                wg_outcome.download_error_v4 = netstack_response_v4.download_error;
            }
            Ok(NetstackResult::Error { error }) => {
                error!("Netstack runtime error: {error}")
            }
            Err(error) => {
                error!("Internal error: {error}")
            }
        }

        // Perform IPv6 ping test
        let ipv6_request = NetstackRequestGo::from_rust_v6(&netstack_request);

        match netstack::ping(&ipv6_request) {
            Ok(NetstackResult::Response(netstack_response_v6)) => {
                info!(
                    "Wireguard probe response for IPv6: {:#?}",
                    netstack_response_v6
                );
                wg_outcome.can_handshake_v6 = netstack_response_v6.can_handshake;
                wg_outcome.can_resolve_dns_v6 = netstack_response_v6.can_resolve_dns;
                wg_outcome.ping_hosts_performance_v6 = netstack_response_v6.received_hosts as f32
                    / netstack_response_v6.sent_hosts as f32;
                wg_outcome.ping_ips_performance_v6 =
                    netstack_response_v6.received_ips as f32 / netstack_response_v6.sent_ips as f32;

                wg_outcome.download_duration_sec_v6 = netstack_response_v6.download_duration_sec;
                wg_outcome.downloaded_file_v6 = netstack_response_v6.downloaded_file;
                wg_outcome.download_error_v6 = netstack_response_v6.download_error;
            }
            Ok(NetstackResult::Error { error }) => {
                error!("Netstack runtime error: {error}")
            }
            Err(error) => {
                error!("Internal error: {error}")
            }
        }
    }

    Ok(wg_outcome)
}

async fn lookup_gateways(gateway_config: GatewayDirectoryConfig) -> anyhow::Result<GatewayList> {
    info!("nym-api: {}", gateway_config.api_url());
    info!(
        "nym-vpn-api: {}",
        gateway_config
            .nym_vpn_api_url()
            .map(|url| url.to_string())
            .unwrap_or("unavailable".to_string())
    );

    let user_agent = nym_bin_common::bin_info_local_vergen!().into();
    let gateway_client = GatewayDirectoryClient::new(gateway_config, user_agent)?;
    let gateways = gateway_client.lookup_all_gateways_from_nym_api().await?;
    Ok(gateways)
}

fn mixnet_debug_config(
    min_gateway_performance: Option<GatewayMinPerformance>,
    ignore_egress_epoch_role: bool,
) -> nym_client_core::config::DebugConfig {
    let mut debug_config = nym_client_core::config::DebugConfig::default();
    debug_config
        .traffic
        .disable_main_poisson_packet_distribution = true;
    debug_config.cover_traffic.disable_loop_cover_traffic_stream = true;
    if let Some(minimum_gateway_performance) =
        min_gateway_performance.and_then(|p| p.mixnet_min_performance)
    {
        debug_config.topology.minimum_gateway_performance =
            minimum_gateway_performance.round_to_integer();
    }
    if ignore_egress_epoch_role {
        debug_config.topology.ignore_egress_epoch_role = ignore_egress_epoch_role;
    }

    debug_config
}

async fn do_ping(
    mut mixnet_client: MixnetClient,
    our_address: Recipient,
    exit_router_address: Option<IpPacketRouterAddress>,
    tested_entry: bool,
) -> (anyhow::Result<ProbeOutcome>, MixnetClient) {
    let entry = do_ping_entry(&mut mixnet_client, our_address, tested_entry).await;

    let (exit_result, mixnet_client) = if let Some(exit_router_address) = exit_router_address {
        let (maybe_ip_pair, mut mixnet_client) =
            connect_exit(mixnet_client, exit_router_address).await;
        match maybe_ip_pair {
            Some(ip_pair) => (
                do_ping_exit(&mut mixnet_client, ip_pair, exit_router_address).await,
                mixnet_client,
            ),
            None => (Ok(Some(Exit::fail_to_connect())), mixnet_client),
        }
    } else {
        (Ok(None), mixnet_client)
    };

    (
        exit_result.map(|exit| ProbeOutcome {
            as_entry: entry,
            as_exit: exit,
            wg: None,
        }),
        mixnet_client,
    )
}

async fn do_ping_entry(
    mixnet_client: &mut MixnetClient,
    our_address: Recipient,
    tested_entry: bool,
) -> Entry {
    // Step 1: confirm that the entry gateway is routing our mixnet traffic
    info!("Sending mixnet ping to ourselves to verify mixnet connection");

    if self_ping_and_wait(our_address, mixnet_client)
        .await
        .is_err()
    {
        return if tested_entry {
            Entry::fail_to_connect()
        } else {
            Entry::EntryFailure
        };
    }
    info!("Successfully mixnet pinged ourselves");

    if tested_entry {
        Entry::success()
    } else {
        Entry::NotTested
    }
}

async fn connect_exit(
    mixnet_client: MixnetClient,
    exit_router_address: IpPacketRouterAddress,
) -> (Option<IpPair>, MixnetClient) {
    // Step 2: connect to the exit gateway
    info!(
        "Connecting to exit gateway: {}",
        exit_router_address.gateway().to_base58_string()
    );
    // The IPR supports cancellation, but it's unused in the gateway probe
    let cancel_token = CancellationToken::new();
    let mut ipr_client = IprClientConnect::new(mixnet_client, cancel_token).await;

    let maybe_ip_pair = ipr_client.connect(exit_router_address.into()).await;
    let mixnet_client = ipr_client.into_mixnet_client();

    if let Ok(our_ips) = maybe_ip_pair {
        info!("Successfully connected to exit gateway");
        info!("Using mixnet VPN IP addresses: {our_ips}");
        (Some(our_ips), mixnet_client)
    } else {
        (None, mixnet_client)
    }
}

async fn do_ping_exit(
    mixnet_client: &mut MixnetClient,
    our_ips: IpPair,
    exit_router_address: IpPacketRouterAddress,
) -> anyhow::Result<Option<Exit>> {
    // Step 3: perform ICMP connectivity checks for the exit gateway
    send_icmp_pings(mixnet_client, our_ips, exit_router_address).await?;
    listen_for_icmp_ping_replies(mixnet_client, our_ips).await
}

async fn send_icmp_pings(
    mixnet_client: &MixnetClient,
    our_ips: IpPair,
    exit_router_address: IpPacketRouterAddress,
) -> anyhow::Result<()> {
    // ipv4 addresses for testing
    let ipr_tun_ip_v4 = NYM_TUN_DEVICE_ADDRESS_V4;
    let external_ip_v4 = Ipv4Addr::new(8, 8, 8, 8);

    // ipv6 addresses for testing
    let ipr_tun_ip_v6 = NYM_TUN_DEVICE_ADDRESS_V6;
    let external_ip_v6 = Ipv6Addr::new(0x2001, 0x4860, 0x4860, 0, 0, 0, 0, 0x8888);

    info!(
        "Sending ICMP echo requests to: {ipr_tun_ip_v4}, {ipr_tun_ip_v6}, {external_ip_v4}, {external_ip_v6}"
    );

    // send ipv4 pings
    for ii in 0..10 {
        send_ping_v4(
            mixnet_client,
            our_ips,
            ii,
            ipr_tun_ip_v4,
            exit_router_address,
        )
        .await?;
        send_ping_v4(
            mixnet_client,
            our_ips,
            ii,
            external_ip_v4,
            exit_router_address,
        )
        .await?;
    }

    // send ipv6 pings
    for ii in 0..10 {
        send_ping_v6(
            mixnet_client,
            our_ips,
            ii,
            ipr_tun_ip_v6,
            exit_router_address,
        )
        .await?;
        send_ping_v6(
            mixnet_client,
            our_ips,
            ii,
            external_ip_v6,
            exit_router_address,
        )
        .await?;
    }
    Ok(())
}

async fn listen_for_icmp_ping_replies(
    mixnet_client: &mut MixnetClient,
    our_ips: IpPair,
) -> anyhow::Result<Option<Exit>> {
    let mut multi_ip_packet_decoder = MultiIpPacketCodec::new();
    let mut registered_replies = IpPingReplies::new();

    loop {
        tokio::select! {
            _ = tokio::time::sleep(Duration::from_secs(2)) => {
                info!("Finished waiting for ICMP echo reply from exit gateway");
                break;
            }
            Some(reconstructed_message) = mixnet_client.next() => {
                let Some(data_response) = unpack_data_response(&reconstructed_message) else {
                    continue;
                };

                // IP packets are bundled together in a mixnet message
                let mut bytes = BytesMut::from(&*data_response.ip_packet);
                while let Ok(Some(packet)) = multi_ip_packet_decoder.decode(&mut bytes) {
                    if let Some(event) = check_for_icmp_beacon_reply(&packet.into_bytes(), icmp_identifier(), our_ips) {
                        info!("Received ICMP echo reply from exit gateway");
                        info!("Connection event: {event:?}");
                        registered_replies.register_event(&event);
                    }
                }
            }
        }
    }

    Ok(Some(Exit {
        can_connect: true,
        can_route_ip_v4: registered_replies.ipr_tun_ip_v4,
        can_route_ip_external_v4: registered_replies.external_ip_v4,
        can_route_ip_v6: registered_replies.ipr_tun_ip_v6,
        can_route_ip_external_v6: registered_replies.external_ip_v6,
    }))
}

fn unpack_data_response(reconstructed_message: &ReconstructedMessage) -> Option<DataResponse> {
    match IpPacketResponse::from_reconstructed_message(reconstructed_message) {
        Ok(response) => match response.data {
            IpPacketResponseData::Data(data_response) => Some(data_response),
            IpPacketResponseData::Control(control) => match *control {
                ControlResponse::Info(info) => {
                    let msg = format!("Received info response from the mixnet: {}", info.reply);
                    match info.level {
                        InfoLevel::Info => info!("{msg}"),
                        InfoLevel::Warn => warn!("{msg}"),
                        InfoLevel::Error => error!("{msg}"),
                    }
                    None
                }
                _ => {
                    info!("Ignoring: {:?}", control);
                    None
                }
            },
        },
        Err(err) => {
            warn!("Failed to parse mixnet message: {err}");
            None
        }
    }
}
