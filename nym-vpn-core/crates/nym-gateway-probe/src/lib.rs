use anyhow::bail;
use base64::{engine::general_purpose, Engine as _};
use bytes::BytesMut;
use futures::StreamExt;
use nym_authenticator_requests::v1::response::{
    AuthenticatorResponseData, PendingRegistrationResponse, RegisteredResponse,
};
use nym_bin_common::bin_info;
use nym_config::defaults::NymNetworkDetails;
use nym_connection_monitor::self_ping_and_wait;
use nym_gateway_directory::{
    AuthAddress, Config as GatewayDirectoryConfig, EntryPoint,
    GatewayClient as GatewayDirectoryClient, GatewayList, IpPacketRouterAddress,
};
use nym_ip_packet_client::{IprClient, SharedMixnetClient};
use nym_ip_packet_requests::{
    codec::MultiIpPacketCodec,
    response::{DataResponse, InfoLevel, IpPacketResponse, IpPacketResponseData},
    IpPair,
};
use nym_sdk::mixnet::{MixnetClient, MixnetClientBuilder, ReconstructedMessage};
use nym_wireguard_types::{
    registration::RegistrationData, ClientMessage, GatewayClient, InitMessage, PeerPublicKey,
};

use std::{
    net::{Ipv4Addr, Ipv6Addr},
    sync::Arc,
    time::Duration,
};
use tokio::{net::UdpSocket, sync::Mutex};
use tokio_util::codec::Decoder;
use tracing::*;
use types::Wg;
use wg::wireguard_test_peer;

use crate::{
    icmp::{check_for_icmp_beacon_reply, icmp_identifier, send_ping_v4, send_ping_v6},
    types::{Entry, Exit},
};

mod error;
mod icmp;
mod types;
mod wg;

const MAX_PACKET: usize = 65535;

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
    let entry_gateway = entry_point.lookup_gateway(&gateways)?;
    let exit_router_address = entry_gateway.ipr_address;
    let authenticator = entry_gateway.authenticator_address;
    let gateway_host = entry_gateway.host.clone().unwrap();
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
        Wg::default()
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
    gateway_host: &str,
) -> anyhow::Result<Wg> {
    let auth_shared_client =
        nym_authenticator_client::SharedMixnetClient::from_shared(&shared_mixnet_client);
    let mut auth_client = nym_authenticator_client::AuthClient::new(auth_shared_client).await;

    let mut rng = rand::thread_rng();
    let private_key = nym_crypto::asymmetric::encryption::PrivateKey::new(&mut rng);
    let public_key = private_key.public_key();

    let init_message = ClientMessage::Initial(InitMessage {
        pub_key: PeerPublicKey::new(public_key.to_bytes().into()),
    });

    let mut wg_outcome = Wg {
        can_register: false,
        can_handshake: false,
    };

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

                let finalized_message = ClientMessage::Final(GatewayClient::new(
                    &private_key,
                    gateway_data.pub_key().inner(),
                    gateway_data.private_ip,
                    nonce,
                ));
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

        let peer_public = registered_data.pub_key.inner();
        let static_private = x25519_dalek::StaticSecret::from(private_key.to_bytes());

        let private_key_bs64 = general_purpose::STANDARD.encode(static_private.as_bytes());
        let public_key_bs64 = general_purpose::STANDARD.encode(peer_public.as_bytes());

        info!("WG connection details");
        // info!("Our private key: {}", private_key_bs64);
        info!("Peer public key: {}", public_key_bs64);
        info!(
            "ip {}, port {}",
            registered_data.private_ip, registered_data.wg_port,
        );
        let wg_endpoint = format!("{}:{}", gateway_host, registered_data.wg_port);
        let socket = Arc::new(
            UdpSocket::bind(format!("{}:50000", "0.0.0.0"))
                .await
                .unwrap(),
        );
        socket.connect(wg_endpoint).await.unwrap();

        info!("Successfully registered with the gateway");

        wg_outcome.can_register = true;

        match wireguard_test_peer(socket, static_private, peer_public, gateway_host).await {
            Ok(can_handhshake) => {
                info!("Successfully connected to the gateway");
                wg_outcome.can_handshake = can_handhshake;
            }
            Err(err) => {
                error!("Failed to connect to the gateway: {err}");
            }
        };
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

    let user_agent = bin_info!().into();
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
    let mut ipr_client = IprClient::new(shared_mixnet_client.clone()).await;
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
    let ipr_tun_ip_v4 = Ipv4Addr::new(10, 0, 0, 1);
    let ipr_tun_ip_v6 = Ipv6Addr::new(0x2001, 0xdb8, 0xa160, 0, 0, 0, 0, 0x1);
    let external_ip_v4 = Ipv4Addr::new(8, 8, 8, 8);
    let external_ip_v6 = Ipv6Addr::new(0x2001, 0x4860, 0x4860, 0, 0, 0, 0, 0x8888);
    info!("Sending ICMP echo requests to: {ipr_tun_ip_v4}, {ipr_tun_ip_v6}, {external_ip_v4}, {external_ip_v6}");
    for ii in 0..10 {
        // HACK: there is hidden hardcoded assumption about these IPs inside
        // `check_for_icmp_beacon_reply`
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
