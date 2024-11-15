use std::{
    net::{Ipv4Addr, Ipv6Addr},
    sync::Arc,
    time::Duration,
};

use bytes::BytesMut;
use futures::StreamExt as _;
use nym_bin_common::bin_info_local_vergen;
use nym_client_core::config::DebugConfig;
use nym_connection_monitor::self_ping_and_wait;
use nym_gateway_directory::{
    Config as GatewayDirectoryConfig, EntryPoint, GatewayClient as GatewayDirectoryClient,
    GatewayList, IpPacketRouterAddress,
};
use nym_ip_packet_client::{IprClientConnect, SharedMixnetClient};
use nym_ip_packet_requests::{
    codec::{MultiIpPacketCodec, BUFFER_TIMEOUT},
    response::{DataResponse, InfoLevel, IpPacketResponse, IpPacketResponseData},
    IpPair,
};
use nym_sdk::{
    mixnet::{MixnetClientBuilder, ReconstructedMessage},
    NymNetworkDetails,
};
use tokio_util::codec::Decoder;
use tracing::{error, info, warn};
use types::{Entry, Exit, IpPingReplies, ProbeOutcome, ProbeResult, WgProbeResults};

use icmp::{check_for_icmp_beacon_reply, icmp_identifier, send_ping_v4, send_ping_v6};

mod icmp;
pub mod run;
mod types;
mod wg;

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
        wg::probe(authenticator, shared_client, gateway_host)
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

    let user_agent = bin_info_local_vergen!().into();
    let gateway_client = GatewayDirectoryClient::new(gateway_config.clone(), user_agent)?;
    let gateways = gateway_client.lookup_all_gateways_from_nym_api().await?;
    Ok(gateways)
}

fn mixnet_debug_config() -> DebugConfig {
    let mut debug_config = DebugConfig::default();
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
    let mut multi_ip_packet_decoder = MultiIpPacketCodec::new(BUFFER_TIMEOUT);
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
