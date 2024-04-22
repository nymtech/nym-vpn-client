use bytes::BytesMut;
use futures::StreamExt;
use nym_config::{
    defaults::{
        var_names::{EXPLORER_API, NYM_API},
        NymNetworkDetails,
    },
    OptionalSet,
};
use nym_connection_monitor::self_ping_and_wait;
use nym_gateway_directory::{
    Config as GatewayDirectoryConfig, DescribedGatewayWithLocation, EntryPoint, ExitPoint,
    GatewayClient as GatewayDirectoryClient, IpPacketRouterAddress,
};
use nym_ip_packet_requests::{
    codec::MultiIpPacketCodec,
    response::{DataResponse, InfoLevel, IpPacketResponse, IpPacketResponseData},
    IpPair,
};
use nym_sdk::mixnet::{MixnetClientBuilder, ReconstructedMessage};
use std::{
    net::{Ipv4Addr, Ipv6Addr},
    time::Duration,
};
use tokio_util::codec::Decoder;
use tracing::*;

use crate::{
    icmp::{check_for_icmp_beacon_reply, icmp_identifier, send_ping_v4, send_ping_v6},
    ipr_connect::{connect_to_ip_packet_router, SharedMixnetClient},
    types::{Entry, Exit},
};

mod error;
mod icmp;
mod ipr_connect;
mod types;

pub use error::{Error, Result};
pub use types::{IpPingReplies, ProbeOutcome, ProbeResult};

pub async fn fetch_gateways() -> anyhow::Result<Vec<DescribedGatewayWithLocation>> {
    lookup_gateways().await
}

pub async fn fetch_gateways_with_ipr() -> anyhow::Result<Vec<DescribedGatewayWithLocation>> {
    let gateways = lookup_gateways().await?;
    Ok(extract_out_exit_gateways(gateways).await)
}

pub async fn probe(entry_point: EntryPoint) -> anyhow::Result<ProbeResult> {
    // Setup the entry gateways
    let gateways = lookup_gateways().await?;
    let (entry_gateway_id, _) = entry_point.lookup_gateway_identity(&gateways).await?;

    // Setup the exit gateway to be the same as entry gateway.
    let exit_point = ExitPoint::Gateway {
        identity: entry_gateway_id,
    };
    let exit_gateways = extract_out_exit_gateways(gateways.clone()).await;
    let exit_router_address = exit_point
        .lookup_router_address(&exit_gateways)
        .map(|(address, _)| address)
        .ok();

    // Connect to the mixnet
    let mixnet_client = MixnetClientBuilder::new_ephemeral()
        .request_gateway(entry_gateway_id.to_string())
        .network_details(NymNetworkDetails::new_from_env())
        .debug_config(mixnet_debug_config())
        .build()?
        .connect_to_mixnet()
        .await;

    let Ok(mixnet_client) = mixnet_client else {
        return Ok(ProbeResult {
            gateway: entry_gateway_id.to_string(),
            outcome: ProbeOutcome {
                as_entry: Entry::fail_to_connect(),
                as_exit: None,
            },
        });
    };

    let nym_address = *mixnet_client.nym_address();
    let entry_gateway = nym_address.gateway().to_base58_string();

    info!("Successfully connected to entry gateway: {entry_gateway}");
    info!("Our nym address: {nym_address}");

    // Now that we have a connected mixnet client, we can start pinging
    let shared_mixnet_client = SharedMixnetClient::new(mixnet_client);
    let outcome = do_ping(shared_mixnet_client.clone(), exit_router_address).await;

    // Disconnect the mixnet client gracefully
    let mixnet_client = shared_mixnet_client.lock().await.take().unwrap();
    mixnet_client.disconnect().await;

    outcome.map(|outcome| ProbeResult {
        gateway: entry_gateway.clone(),
        outcome,
    })
}

async fn lookup_gateways() -> anyhow::Result<Vec<DescribedGatewayWithLocation>> {
    let gateway_config = GatewayDirectoryConfig::new_from_env();
    info!("nym-api: {}", gateway_config.api_url());
    info!(
        "explorer-api: {}",
        gateway_config
            .explorer_url()
            .map(|url| url.to_string())
            .unwrap_or("unavailable".to_string())
    );
    info!(
        "harbour-master: {}",
        gateway_config
            .harbour_master_url()
            .map(|url| url.to_string())
            .unwrap_or("unavailable".to_string())
    );

    let gateway_client = GatewayDirectoryClient::new(gateway_config.clone())?;
    Ok(gateway_client
        .lookup_described_gateways_with_location()
        .await?)
}

async fn extract_out_exit_gateways(
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
        });
    }
    info!("Successfully mixnet pinged ourselves");

    let Some(exit_router_address) = exit_router_address else {
        return Ok(ProbeOutcome {
            as_entry: Entry::success(),
            as_exit: None,
        });
    };

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
        return Ok(ProbeOutcome {
            as_entry: Entry::success(),
            as_exit: Some(Exit::fail_to_connect()),
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
