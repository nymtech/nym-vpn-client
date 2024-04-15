use bytes::{Bytes, BytesMut};
use futures::StreamExt;
use nym_connection_monitor::{
    packet_helpers::{
        create_icmpv4_echo_request, create_icmpv6_echo_request, wrap_icmp_in_ipv4,
        wrap_icmp_in_ipv6,
    },
    self_ping_and_wait,
};
use nym_gateway_directory::{
    DescribedGatewayWithLocation, GatewayClient as GatewayDirectoryClient, IpPacketRouterAddress,
};
use nym_ip_packet_requests::{
    codec::MultiIpPacketCodec,
    response::{DataResponse, InfoLevel, IpPacketResponse, IpPacketResponseData},
    IpPair,
};
use nym_sdk::mixnet::{InputMessage, MixnetClientBuilder, Recipient, ReconstructedMessage};
use nym_task::connections::TransmissionLane;
use nym_vpn_lib::{
    error::*,
    gateway_directory::{Config as GatewayDirectoryConfig, EntryPoint, ExitPoint},
    mixnet_connect::{connect_to_ip_packet_router, SharedMixnetClient},
    mixnet_processor::check_for_icmp_beacon_reply,
    nym_config::{
        defaults::{
            setup_env,
            var_names::{EXPLORER_API, NYM_API},
        },
        OptionalSet,
    },
};
use pnet_packet::Packet;
use std::{
    net::{Ipv4Addr, Ipv6Addr},
    path::PathBuf,
    time::Duration,
};
use tokio_util::codec::Decoder;
use tracing::*;
use types::PingResult;

use crate::types::{IpPingReplies, PingOutcome};

mod types;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if let Err(err) = run().await {
        error!("Exit with error: {err}");
        eprintln!("An error occurred: {err}");
        std::process::exit(1)
    }
    Ok(())
}

async fn run() -> anyhow::Result<PingResult> {
    setup_logging();
    debug!("{:?}", nym_vpn_lib::nym_bin_common::bin_info!());
    // mainnet by default
    setup_env::<PathBuf>(None);
    let result = ping(EntryPoint::Random, ExitPoint::Random).await;
    match result {
        Ok(ref result) => {
            println!("{:#?}", result);
        }
        Err(ref err) => {
            println!("Error: {err}");
        }
    };
    result
}

async fn lookup_gateways() -> anyhow::Result<Vec<DescribedGatewayWithLocation>> {
    let gateway_config = GatewayDirectoryConfig::default()
        .with_optional_env(GatewayDirectoryConfig::with_custom_api_url, None, NYM_API)
        .with_optional_env(
            GatewayDirectoryConfig::with_custom_explorer_url,
            None,
            EXPLORER_API,
        );
    info!("nym-api: {}", gateway_config.api_url());
    info!(
        "explorer-api: {}",
        gateway_config
            .explorer_url()
            .map(|url| url.to_string())
            .unwrap_or("unavailable".to_string())
    );

    let gateway_client = GatewayDirectoryClient::new(gateway_config.clone())?;
    Ok(gateway_client
        .lookup_described_gateways_with_location()
        .await?)
}

async fn exit_gateways(
    gateways: Vec<DescribedGatewayWithLocation>,
) -> Vec<DescribedGatewayWithLocation> {
    gateways
        .into_iter()
        .filter(|gateway| gateway.is_current_build())
        .collect()
}

fn mixnet_debug_config() -> nym_client_core::config::DebugConfig {
    let mut debug_config = nym_client_core::config::DebugConfig::default();
    debug_config
        .traffic
        .disable_main_poisson_packet_distribution = true;
    debug_config.cover_traffic.disable_loop_cover_traffic_stream = true;
    debug_config
}

async fn ping(entry_point: EntryPoint, exit_point: ExitPoint) -> anyhow::Result<PingResult> {
    // Setup the entry gateways
    let gateways = lookup_gateways().await?;
    let (entry_gateway_id, _) = entry_point.lookup_gateway_identity(&gateways).await?;

    // Setup the exit gateway
    let exit_gateways = exit_gateways(gateways.clone()).await;
    let (exit_router_address, _) = exit_point.lookup_router_address(&exit_gateways)?;

    // Connect to the mixnet
    let mixnet_client = MixnetClientBuilder::new_ephemeral()
        .request_gateway(entry_gateway_id.to_string())
        .network_details(nym_vpn_lib::nym_config::defaults::NymNetworkDetails::new_from_env())
        .debug_config(mixnet_debug_config())
        .build()?
        .connect_to_mixnet()
        .await;

    let Ok(mixnet_client) = mixnet_client else {
        return Ok(PingResult {
            entry_gateway: entry_gateway_id.to_string(),
            exit_gateway: exit_router_address.gateway().to_base58_string(),
            outcome: PingOutcome::EntryGatewayNotConnected,
        });
    };

    let nym_address = *mixnet_client.nym_address();
    let entry_gateway = nym_address.gateway().to_base58_string();
    let exit_gateway = exit_router_address.gateway().to_base58_string();

    info!("Successfully connected to entry gateway: {entry_gateway}");
    info!("Our nym address: {nym_address}");

    // Now that we have a connected mixnet client, we can start pinging
    let shared_mixnet_client = SharedMixnetClient::new(mixnet_client);
    let outcome = do_ping(shared_mixnet_client.clone(), exit_router_address).await;

    // Disconnect the mixnet client gracefully
    let mixnet_client = shared_mixnet_client.lock().await.take().unwrap();
    mixnet_client.disconnect().await;

    outcome.map(|outcome| PingResult {
        entry_gateway: entry_gateway.clone(),
        exit_gateway: exit_gateway.clone(),
        outcome,
    })
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

fn unpack_data_response(reconstructed_message: &ReconstructedMessage) -> Option<DataResponse> {
    match IpPacketResponse::from_reconstructed_message(&reconstructed_message) {
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

async fn listen_for_icmp_ping_replies(
    shared_mixnet_client: SharedMixnetClient,
    our_ips: IpPair,
) -> anyhow::Result<PingOutcome> {
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
    Ok(PingOutcome::IpPingReplies(registered_replies))
}

async fn do_ping(
    shared_mixnet_client: SharedMixnetClient,
    exit_router_address: IpPacketRouterAddress,
) -> anyhow::Result<PingOutcome> {
    // Step 1: confirm that the entry gateway is routing our mixnet traffic
    info!("Sending mixnet ping to ourselves to verify mixnet connection");
    if self_ping_and_wait(
        shared_mixnet_client.nym_address().await,
        shared_mixnet_client.inner(),
    )
    .await
    .is_err()
    {
        return Ok(PingOutcome::EntryGatewayNotRouting);
    }
    info!("Successfully mixnet pinged ourselves");

    // Step 2: connect to the exit gateway
    info!(
        "Connecting to exit gateway: {}",
        exit_router_address.gateway().to_base58_string()
    );
    let Ok(our_ips) = connect_to_ip_packet_router(
        shared_mixnet_client.clone(),
        &exit_router_address,
        None,
        false,
    )
    .await
    else {
        return Ok(PingOutcome::ExitGatewayNotConnected);
    };
    info!("Successfully connected to exit gateway");
    info!("Using mixnet VPN IP addresses: {our_ips}");

    // Step 3: perform ICMP connectivity checks for the exit gateway
    send_icmp_pings(shared_mixnet_client.clone(), our_ips, exit_router_address).await?;
    listen_for_icmp_ping_replies(shared_mixnet_client.clone(), our_ips).await
}

async fn send_ping_v4(
    shared_mixnet_client: SharedMixnetClient,
    our_ips: IpPair,
    sequence_number: u16,
    destination: Ipv4Addr,
    exit_router_address: IpPacketRouterAddress,
) -> anyhow::Result<()> {
    let icmp_identifier = icmp_identifier();
    let icmp_echo_request = create_icmpv4_echo_request(sequence_number, icmp_identifier)?;
    let ipv4_packet = wrap_icmp_in_ipv4(icmp_echo_request, our_ips.ipv4, destination)?;

    // Wrap the IPv4 packet in a MultiIpPacket
    let bundled_packet =
        MultiIpPacketCodec::bundle_one_packet(ipv4_packet.packet().to_vec().into());

    // Wrap into a mixnet input message addressed to the IPR
    let two_hop = true;
    let mixnet_message = create_input_message(exit_router_address.0, bundled_packet, two_hop)?;

    shared_mixnet_client.send(mixnet_message).await?;
    Ok(())
}

async fn send_ping_v6(
    shared_mixnet_client: SharedMixnetClient,
    our_ips: IpPair,
    sequence_number: u16,
    destination: Ipv6Addr,
    exit_router_address: IpPacketRouterAddress,
) -> anyhow::Result<()> {
    let icmp_identifier = icmp_identifier();
    let icmp_echo_request = create_icmpv6_echo_request(
        sequence_number,
        icmp_identifier,
        &our_ips.ipv6,
        &destination,
    )?;
    let ipv6_packet = wrap_icmp_in_ipv6(icmp_echo_request, our_ips.ipv6, destination)?;

    // Wrap the IPv6 packet in a MultiIpPacket
    let bundled_packet =
        MultiIpPacketCodec::bundle_one_packet(ipv6_packet.packet().to_vec().into());

    // Wrap into a mixnet input message addressed to the IPR
    let two_hop = true;
    let mixnet_message = create_input_message(exit_router_address.0, bundled_packet, two_hop)?;

    // Send across the mixnet
    shared_mixnet_client.send(mixnet_message).await?;
    Ok(())
}

fn icmp_identifier() -> u16 {
    8475
}

fn create_input_message(
    recipient: Recipient,
    bundled_packets: Bytes,
    enable_two_hop: bool,
) -> Result<InputMessage> {
    let packet =
        nym_ip_packet_requests::request::IpPacketRequest::new_data_request(bundled_packets)
            .to_bytes()?;

    let lane = TransmissionLane::General;
    let packet_type = None;
    let hops = enable_two_hop.then_some(0);
    Ok(InputMessage::new_regular_with_custom_hops(
        recipient,
        packet,
        lane,
        packet_type,
        hops,
    ))
}

fn setup_logging() {
    let filter = tracing_subscriber::EnvFilter::builder()
        .with_default_directive(tracing_subscriber::filter::LevelFilter::INFO.into())
        .from_env()
        .unwrap()
        .add_directive("hyper::proto=info".parse().unwrap())
        .add_directive("netlink_proto=info".parse().unwrap());

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .compact()
        .init();
}
