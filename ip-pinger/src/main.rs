use bytes::{Bytes, BytesMut};
use futures::StreamExt;
use nym_gateway_directory::{DescribedGatewayWithLocation, GatewayClient};
use nym_ip_packet_requests::{
    codec::MultiIpPacketCodec,
    response::{InfoLevel, IpPacketResponse, IpPacketResponseData},
};
use nym_sdk::mixnet::{InputMessage, MixnetClientBuilder, Recipient};
use nym_task::{connections::TransmissionLane, TaskManager};
use nym_vpn_lib::{
    error::*,
    gateway_directory::{Config as GatewayConfig, EntryPoint, ExitPoint},
    nym_config::{
        defaults::{
            setup_env,
            var_names::{EXPLORER_API, NYM_API},
        },
        OptionalSet,
    },
};
use pnet_packet::Packet;
use std::{net::Ipv4Addr, path::PathBuf, time::Duration};
use tokio_util::codec::Decoder;
use tracing::*;

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
    // setup_env(args.config_env_file.as_ref());
    setup_env::<PathBuf>(None);
    let result = do_ping(EntryPoint::Random, ExitPoint::Random).await;
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

#[derive(Debug)]
struct PingResult {
    entry_gateway: String,
    exit_gateway: String,
    outcome: PingOutcome,
}

#[derive(Debug)]
enum PingOutcome {
    EntryGatewayNotConnected,
    EntryGatewayNotRouting,
    ExitGatewayNotConnected,
    ExitGatewayTunNotResponding,
    ExitGatewayNotRoutingExternal,
    FullyOperational,
}

async fn do_ping(entry_point: EntryPoint, exit_point: ExitPoint) -> anyhow::Result<PingResult> {
    let gateway_config = GatewayConfig::default()
        .with_optional_env(GatewayConfig::with_custom_api_url, None, NYM_API)
        .with_optional_env(GatewayConfig::with_custom_explorer_url, None, EXPLORER_API);
    info!("nym-api: {}", gateway_config.api_url());
    info!(
        "explorer-api: {}",
        gateway_config
            .explorer_url()
            .map(|url| url.to_string())
            .unwrap_or("unavailable".to_string())
    );

    // let (vpn_ctrl_tx, vpn_ctrl_rx) =
    //     futures::channel::mpsc::unbounded::<nym_vpn_lib::NymVpnCtrlMessage>();
    // let (vpn_status_tx, vpn_status_rx) =
    //     futures::channel::mpsc::channel::<nym_vpn_lib::SentStatus>(128);
    // let (vpn_exit_tx, vpn_exit_rx) =
    //     futures::channel::oneshot::channel::<nym_vpn_lib::NymVpnExitStatusMessage>();

    let gateway_client = GatewayClient::new(gateway_config.clone())?;
    let gateways = gateway_client
        .lookup_described_gateways_with_location()
        .await?;
    let working_exit_gateways: Vec<DescribedGatewayWithLocation> = gateways
        .clone()
        .into_iter()
        .filter(|gateway| gateway.is_current_build())
        .collect();

    let (entry_gateway_id, _entry_location) =
        entry_point.lookup_gateway_identity(&gateways).await?;
    let (exit_router_address, _exit_location) =
        exit_point.lookup_router_address(&working_exit_gateways)?;

    let task_manager = TaskManager::new(10).named("ip_pinger");

    let mut debug_config = nym_client_core::config::DebugConfig::default();
    debug_config
        .traffic
        .disable_main_poisson_packet_distribution = true;
    debug_config.cover_traffic.disable_loop_cover_traffic_stream = true;

    let mixnet_client = MixnetClientBuilder::new_ephemeral()
        .request_gateway(entry_gateway_id.to_string())
        .network_details(nym_vpn_lib::nym_config::defaults::NymNetworkDetails::new_from_env())
        .debug_config(debug_config)
        .custom_shutdown(task_manager.subscribe_named("mixnet_client_main"))
        // .credentials_mode(enable_credentials_mode)
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

    info!("Successfully connected to entry gateway: {entry_gateway}");
    info!("Our nym address: {nym_address}");

    info!("Sending mixnet ping to ourselves to verify mixnet connection");
    let shared_mixnet_client = nym_vpn_lib::mixnet_connect::SharedMixnetClient::new(mixnet_client);
    if nym_connection_monitor::self_ping_and_wait(nym_address, shared_mixnet_client.inner())
        .await
        .is_err()
    {
        shared_mixnet_client.disconnect().await;
        return Ok(PingResult {
            entry_gateway,
            exit_gateway: exit_router_address.gateway().to_base58_string(),
            outcome: PingOutcome::EntryGatewayNotRouting,
        });
    }
    info!("Successfully mixnet pinged ourselves");

    let exit_gateway = exit_router_address.gateway().to_base58_string();
    info!("Connecting to exit gateway: {exit_gateway}");
    let enable_two_hop = false;
    let Ok(our_ips) = nym_vpn_lib::mixnet_connect::connect_to_ip_packet_router(
        shared_mixnet_client.clone(),
        &exit_router_address,
        None,
        enable_two_hop,
    )
    .await
    else {
        shared_mixnet_client.disconnect().await;
        return Ok(PingResult {
            entry_gateway,
            exit_gateway,
            outcome: PingOutcome::ExitGatewayNotConnected,
        });
    };
    info!("Successfully connected to exit gateway");
    info!("Using mixnet VPN IP addresses: {our_ips}");

    // Perform ICMP connectivity check for exit gateway

    // Create ICMP/IPv4 echo request packet
    let sequence_number = 1;
    let icmp_identifier = icmp_identifier();
    let icmp_echo_request = nym_connection_monitor::packet_helpers::create_icmpv4_echo_request(
        sequence_number,
        icmp_identifier,
    )?;
    let destination = "10.0.0.1".parse::<Ipv4Addr>()?;
    let ipv4_packet = nym_connection_monitor::packet_helpers::wrap_icmp_in_ipv4(
        icmp_echo_request,
        our_ips.ipv4,
        destination,
    )?;

    // Wrap the IPv4 packet in a MultiIpPacket
    let bundled_packet =
        MultiIpPacketCodec::bundle_one_packet(ipv4_packet.packet().to_vec().into());

    // Wrap into a mixnet input message addressed to the IPR
    let two_hop = true;
    let mixnet_message = create_input_message(exit_router_address.0, bundled_packet, two_hop)?;

    shared_mixnet_client.send(mixnet_message).await?;

    // Listen for reply
    let mut mixnet_client = shared_mixnet_client.inner().lock().await.take().unwrap();
    let mut multi_ip_packet_decoder =
        MultiIpPacketCodec::new(nym_ip_packet_requests::codec::BUFFER_TIMEOUT);

    tokio::select! {
        _ = tokio::time::sleep(Duration::from_secs(10)) => {
            error!("Timeout waiting for ICMP echo reply from exit gateway");
        }
        Some(reconstructed_message) = mixnet_client.next() => {
            match IpPacketResponse::from_reconstructed_message(&reconstructed_message) {
                Ok(response) => match response.data {
                    IpPacketResponseData::Data(data_response) => {
                        let mut bytes = BytesMut::from(&*data_response.ip_packet);
                        while let Ok(Some(packet)) = multi_ip_packet_decoder.decode(&mut bytes) {
                            if let Some(connection_event) = nym_vpn_lib::mixnet_processor::check_for_icmp_beacon_reply(
                                &packet,
                                icmp_identifier,
                                our_ips,
                            ) {
                                info!("Received ICMP echo reply from exit gateway");
                                info!("Connection event: {:?}", connection_event);
                                return Ok(PingResult {
                                    entry_gateway,
                                    exit_gateway,
                                    outcome: PingOutcome::FullyOperational,
                                });
                            }
                        }
                    }
                    IpPacketResponseData::Info(info) => {
                        let msg = format!("Received info response from the mixnet: {}", info.reply);
                        match info.level {
                            InfoLevel::Info => info!("{msg}"),
                            InfoLevel::Warn => warn!("{msg}"),
                            InfoLevel::Error => error!("{msg}"),
                        }
                    }
                    _ => {
                        info!("Ignoring: {:?}", response);
                    }
                },
                Err(err) => {
                    warn!("Failed to parse mixnet message: {err}");
                }
            }
        }
    }

    mixnet_client.disconnect().await;

    Ok(PingResult {
        entry_gateway,
        exit_gateway,
        outcome: PingOutcome::ExitGatewayTunNotResponding,
    })
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
