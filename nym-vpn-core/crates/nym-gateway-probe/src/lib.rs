// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    sync::Arc,
    time::Duration,
};

use crate::netstack::NetstackRequest;
use anyhow::bail;
use base64::{engine::general_purpose, Engine as _};
use bytes::BytesMut;
use dns_lookup::lookup_host;
use futures::StreamExt;
use netstack::{NetstackCall as _, NetstackCallImpl};
use nym_authenticator_client::ClientMessage;
use nym_authenticator_requests::v4::{
    registration::{FinalMessage, GatewayClient, InitMessage, RegistrationData},
    response::{AuthenticatorResponseData, PendingRegistrationResponse, RegisteredResponse},
};
use nym_config::defaults::NymNetworkDetails;
use nym_connection_monitor::self_ping_and_wait;
use nym_gateway_directory::{
    AuthAddress, Config as GatewayDirectoryConfig, EntryPoint,
    GatewayClient as GatewayDirectoryClient, GatewayList, IpPacketRouterAddress,
};
use nym_ip_packet_client::{IprClientConnect, SharedMixnetClient};
use nym_ip_packet_requests::{
    codec::MultiIpPacketCodec,
    response::{DataResponse, InfoLevel, IpPacketResponse, IpPacketResponseData},
    IpPair,
};
use nym_sdk::mixnet::{MixnetClient, MixnetClientBuilder, ReconstructedMessage};
use nym_wireguard_types::PeerPublicKey;
use tokio::sync::Mutex;
use tokio_util::codec::Decoder;
use tracing::*;
use types::WgProbeResults;

use crate::{
    icmp::{check_for_icmp_beacon_reply, icmp_identifier, send_ping_v4, send_ping_v6},
    types::{Entry, Exit},
};

mod error;
mod icmp;
mod netstack;
mod types;

pub use error::{Error, Result};
pub use types::{IpPingReplies, ProbeOutcome, ProbeResult};

pub async fn fetch_gateways() -> anyhow::Result<GatewayList> {
    lookup_gateways().await
}

pub async fn fetch_gateways_with_ipr() -> anyhow::Result<GatewayList> {
    Ok(lookup_gateways().await?.into_exit_gateways())
}

pub async fn probe(entry_point: EntryPoint) -> anyhow::Result<ProbeResult> {
    // Setup the entry gateways
    let gateways = lookup_gateways().await?;
    let entry_gateway = entry_point.lookup_gateway(&gateways).await?;
    let exit_router_address = entry_gateway.ipr_address;
    let authenticator = entry_gateway.authenticator_address;
    let gateway_host = entry_gateway.host.clone().unwrap();
    println!("gateway_host: {}", gateway_host);
    let entry_gateway_id = entry_gateway.identity();

    info!("Probing gateway: {entry_gateway:?}");

    // Connect to the mixnet
    let mixnet_client = MixnetClientBuilder::new_ephemeral()
        .request_gateway(entry_gateway_id.to_string())
        .network_details(NymNetworkDetails::new_from_env())
        .debug_config(mixnet_debug_config())
        .build()?
        .connect_to_mixnet()
        .await;

    let mixnet_client = match mixnet_client {
        Ok(mixnet_client) => mixnet_client,
        Err(err) => {
            error!("Failed to connect to mixnet: {err}");
            return Ok(ProbeResult {
                gateway: entry_gateway_id.to_string(),
                outcome: ProbeOutcome {
                    as_entry: Entry::fail_to_connect(),
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

    let shared_client = Arc::new(tokio::sync::Mutex::new(Some(mixnet_client)));

    // Now that we have a connected mixnet client, we can start pinging
    let shared_mixnet_client = SharedMixnetClient::from_shared(&shared_client);
    let outcome = do_ping(shared_mixnet_client.clone(), exit_router_address).await;

    let wg_outcome = if let Some(authenticator) = authenticator {
        wg_probe(authenticator, shared_client, &gateway_host)
            .await
            .unwrap_or_default()
    } else {
        WgProbeResults::default()
    };

    let mixnet_client = shared_mixnet_client.lock().await.take().unwrap();
    mixnet_client.disconnect().await;

    // Disconnect the mixnet client gracefully
    outcome.map(|mut outcome| {
        outcome.wg = Some(wg_outcome);
        ProbeResult {
            gateway: entry_gateway.clone(),
            outcome,
        }
    })
}

async fn wg_probe(
    authenticator: AuthAddress,
    shared_mixnet_client: Arc<Mutex<Option<MixnetClient>>>,
    gateway_host: &nym_topology::NetworkAddress,
) -> anyhow::Result<WgProbeResults> {
    let auth_shared_client =
        nym_authenticator_client::SharedMixnetClient::from_shared(&shared_mixnet_client);
    let mut auth_client = nym_authenticator_client::AuthClient::new(auth_shared_client).await;

    let mut rng = rand::thread_rng();
    let private_key = nym_crypto::asymmetric::encryption::PrivateKey::new(&mut rng);
    let public_key = private_key.public_key();

    let init_message = ClientMessage::Initial(InitMessage {
        pub_key: PeerPublicKey::new(public_key.to_bytes().into()),
    });

    let mut wg_outcome = WgProbeResults::default();

    if let Some(authenticator_address) = authenticator.0 {
        let response = auth_client
            .send(init_message, authenticator_address)
            .await?;

        let registered_data = match response.data {
            AuthenticatorResponseData::PendingRegistration(PendingRegistrationResponse {
                reply:
                    RegistrationData {
                        nonce,
                        gateway_data,
                        ..
                    },
                ..
            }) => {
                // Unwrap since we have already checked that we have the keypair.
                debug!("Verifying data");
                gateway_data.verify(&private_key, nonce)?;

                let finalized_message = ClientMessage::Final(Box::new(FinalMessage {
                    gateway_client: GatewayClient::new(
                        &private_key,
                        gateway_data.pub_key().inner(),
                        gateway_data.private_ips,
                        nonce,
                    ),
                    credential: None,
                }));
                let response = auth_client
                    .send(finalized_message, authenticator_address)
                    .await?;
                let AuthenticatorResponseData::Registered(RegisteredResponse { reply, .. }) =
                    response.data
                else {
                    bail!("Unexpected response: {response:?}");
                };
                reply
            }
            AuthenticatorResponseData::Registered(RegisteredResponse { reply, .. }) => reply,
            _ => bail!("Unexpected response: {response:?}"),
        };

        println!("registered_data: {:?}", registered_data);

        let peer_public = registered_data.pub_key.inner();
        let static_private = x25519_dalek::StaticSecret::from(private_key.to_bytes());
        let public_key_bs64 = general_purpose::STANDARD.encode(peer_public.as_bytes());
        let private_key_hex = hex::encode(static_private.to_bytes());
        let public_key_hex = hex::encode(peer_public.as_bytes());

        info!("WG connection details");
        info!("Peer public key: {}", public_key_bs64);
        info!(
            "ips {}(v4) {}(v6), port {}",
            registered_data.private_ips.ipv4,
            registered_data.private_ips.ipv6,
            registered_data.wg_port,
        );

        let (gateway_ip, ip_version) = match gateway_host {
            nym_topology::NetworkAddress::Hostname(host) => lookup_host(host)?
                .first()
                .map(|ip| match ip {
                    IpAddr::V4(ip) => (ip.to_string(), 4),
                    IpAddr::V6(ip) => (format!("[{}]", ip), 6),
                })
                .unwrap_or_default(),
            nym_topology::NetworkAddress::IpAddr(ip) => match ip {
                IpAddr::V4(ip) => (ip.to_string(), 4),
                IpAddr::V6(ip) => (format!("[{}]", ip), 6),
            },
        };

        let wg_endpoint = format!("{}:{}", gateway_ip, registered_data.wg_port);

        info!("Successfully registered with the gateway");

        wg_outcome.can_register = true;

        if wg_outcome.can_register {
            let wg_ip = if ip_version == 4 {
                registered_data.private_ips.ipv4.to_string()
            } else {
                registered_data.private_ips.ipv6.to_string()
            };

            // Perform IPv4 ping test
            let ipv4_request = netstack::NetstackRequest {
                wg_ip: wg_ip.clone(),
                private_key: private_key_hex.clone(),
                public_key: public_key_hex.clone(),
                endpoint: wg_endpoint.clone(),
                ip_version: 4,
                ..NetstackRequest::with_ipv4_defaults()
            };

            let netstack_response_v4 = NetstackCallImpl::ping(&ipv4_request);
            info!(
                "Wireguard probe response for IPv4: {:?}",
                netstack_response_v4
            );
            wg_outcome.can_handshake_v4 = netstack_response_v4.can_handshake;
            wg_outcome.can_resolve_dns_v4 = netstack_response_v4.can_resolve_dns;
            wg_outcome.ping_hosts_performance_v4 =
                netstack_response_v4.received_hosts as f32 / netstack_response_v4.sent_hosts as f32;
            wg_outcome.ping_ips_performance_v4 =
                netstack_response_v4.received_ips as f32 / netstack_response_v4.sent_ips as f32;

            // Perform IPv6 ping test
            let ipv6_request = netstack::NetstackRequest {
                wg_ip,
                private_key: private_key_hex,
                public_key: public_key_hex,
                endpoint: wg_endpoint.clone(),
                dns: "2606:4700:4700::1111".to_string(), // cloudflare's IPv6 DNS
                ping_hosts: vec!["ipv6.google.com".to_string()],
                ping_ips: vec![
                    "2001:4860:4860::8888".to_string(), // google DNS
                    "2606:4700:4700::1111".to_string(), // cloudflare DNS
                    "2620:fe::fe".to_string(),          //Quad9 DNS
                ],
                ip_version: 6,
                ..NetstackRequest::default()
            };

            let netstack_response_v6 = NetstackCallImpl::ping(&ipv6_request);
            info!(
                "Wireguard probe response for IPv6: {:?}",
                netstack_response_v6
            );
            wg_outcome.can_handshake_v6 = netstack_response_v6.can_handshake;
            wg_outcome.can_resolve_dns_v6 = netstack_response_v6.can_resolve_dns;
            wg_outcome.ping_hosts_performance_v6 =
                netstack_response_v6.received_hosts as f32 / netstack_response_v6.sent_hosts as f32;
            wg_outcome.ping_ips_performance_v6 =
                netstack_response_v6.received_ips as f32 / netstack_response_v6.sent_ips as f32;
        }
    }

    Ok(wg_outcome)
}

async fn lookup_gateways() -> anyhow::Result<GatewayList> {
    let gateway_config = GatewayDirectoryConfig::new_from_env();
    info!("nym-api: {}", gateway_config.api_url());
    info!(
        "nym-vpn-api: {}",
        gateway_config
            .nym_vpn_api_url()
            .map(|url| url.to_string())
            .unwrap_or("unavailable".to_string())
    );

    let user_agent = nym_bin_common::bin_info_local_vergen!().into();
    let gateway_client = GatewayDirectoryClient::new(gateway_config.clone(), user_agent)?;
    let gateways = gateway_client.lookup_all_gateways_from_nym_api().await?;
    Ok(gateways)
}

fn mixnet_debug_config() -> nym_client_core::config::DebugConfig {
    let mut debug_config = nym_client_core::config::DebugConfig::default();
    debug_config
        .traffic
        .disable_main_poisson_packet_distribution = true;
    debug_config.cover_traffic.disable_loop_cover_traffic_stream = true;
    debug_config
}

async fn do_ping(
    shared_mixnet_client: SharedMixnetClient,
    exit_router_address: Option<IpPacketRouterAddress>,
) -> anyhow::Result<ProbeOutcome> {
    // Step 1: confirm that the entry gateway is routing our mixnet traffic
    info!("Sending mixnet ping to ourselves to verify mixnet connection");
    if self_ping_and_wait(
        shared_mixnet_client.nym_address().await,
        shared_mixnet_client.inner(),
    )
    .await
    .is_err()
    {
        return Ok(ProbeOutcome {
            as_entry: Entry::fail_to_route(),
            as_exit: None,
            wg: None,
        });
    }
    info!("Successfully mixnet pinged ourselves");

    let Some(exit_router_address) = exit_router_address else {
        return Ok(ProbeOutcome {
            as_entry: Entry::success(),
            as_exit: None,
            wg: None,
        });
    };

    // Step 2: connect to the exit gateway
    info!(
        "Connecting to exit gateway: {}",
        exit_router_address.gateway().to_base58_string()
    );
    let mut ipr_client = IprClientConnect::new(shared_mixnet_client.clone()).await;
    let Ok(our_ips) = ipr_client.connect(exit_router_address.0, None).await else {
        return Ok(ProbeOutcome {
            as_entry: Entry::success(),
            as_exit: Some(Exit::fail_to_connect()),
            wg: None,
        });
    };
    info!("Successfully connected to exit gateway");
    info!("Using mixnet VPN IP addresses: {our_ips}");

    // Step 3: perform ICMP connectivity checks for the exit gateway
    send_icmp_pings(shared_mixnet_client.clone(), our_ips, exit_router_address).await?;
    listen_for_icmp_ping_replies(shared_mixnet_client.clone(), our_ips).await
}

async fn send_icmp_pings(
    shared_mixnet_client: SharedMixnetClient,
    our_ips: IpPair,
    exit_router_address: IpPacketRouterAddress,
) -> anyhow::Result<()> {
    // ipv4 addresses for testing
    let ipr_tun_ip_v4 = Ipv4Addr::new(10, 0, 0, 1);
    let external_ip_v4 = Ipv4Addr::new(8, 8, 8, 8);

    // ipv6 addresses for testing
    let ipr_tun_ip_v6 = Ipv6Addr::new(0x2001, 0xdb8, 0xa160, 0, 0, 0, 0, 0x1);
    let external_ip_v6 = Ipv6Addr::new(0x2001, 0x4860, 0x4860, 0, 0, 0, 0, 0x8888);

    info!("Sending ICMP echo requests to: {ipr_tun_ip_v4}, {ipr_tun_ip_v6}, {external_ip_v4}, {external_ip_v6}");

    // send ipv4 pings
    for ii in 0..10 {
        send_ping_v4(
            shared_mixnet_client.clone(),
            our_ips,
            ii,
            ipr_tun_ip_v4,
            exit_router_address,
        )
        .await?;
        send_ping_v4(
            shared_mixnet_client.clone(),
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
            shared_mixnet_client.clone(),
            our_ips,
            ii,
            ipr_tun_ip_v6,
            exit_router_address,
        )
        .await?;
        send_ping_v6(
            shared_mixnet_client.clone(),
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
    shared_mixnet_client: SharedMixnetClient,
    our_ips: IpPair,
) -> anyhow::Result<ProbeOutcome> {
    // HACK: take it out of the shared mixnet client
    let mut mixnet_client = shared_mixnet_client.inner().lock().await.take().unwrap();
    let mut multi_ip_packet_decoder =
        MultiIpPacketCodec::new(nym_ip_packet_requests::codec::BUFFER_TIMEOUT);
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
                    if let Some(event) = check_for_icmp_beacon_reply(&packet, icmp_identifier(), our_ips) {
                        info!("Received ICMP echo reply from exit gateway");
                        info!("Connection event: {:?}", event);
                        registered_replies.register_event(&event);
                    }
                }
            }
        }
    }

    // HACK: put it back in the shared mixnet client, so it can be properly disconnected
    shared_mixnet_client
        .inner()
        .lock()
        .await
        .replace(mixnet_client);

    Ok(ProbeOutcome {
        as_entry: Entry::success(),
        as_exit: Some(Exit {
            can_connect: true,
            can_route_ip_v4: registered_replies.ipr_tun_ip_v4,
            can_route_ip_external_v4: registered_replies.external_ip_v4,
            can_route_ip_v6: registered_replies.ipr_tun_ip_v6,
            can_route_ip_external_v6: registered_replies.external_ip_v6,
        }),
        wg: None,
    })
}

fn unpack_data_response(reconstructed_message: &ReconstructedMessage) -> Option<DataResponse> {
    match IpPacketResponse::from_reconstructed_message(reconstructed_message) {
        Ok(response) => match response.data {
            IpPacketResponseData::Data(data_response) => Some(data_response),
            IpPacketResponseData::Info(info) => {
                let msg = format!("Received info response from the mixnet: {}", info.reply);
                match info.level {
                    InfoLevel::Info => info!("{msg}"),
                    InfoLevel::Warn => warn!("{msg}"),
                    InfoLevel::Error => error!("{msg}"),
                }
                None
            }
            _ => {
                info!("Ignoring: {:?}", response);
                None
            }
        },
        Err(err) => {
            warn!("Failed to parse mixnet message: {err}");
            None
        }
    }
}
