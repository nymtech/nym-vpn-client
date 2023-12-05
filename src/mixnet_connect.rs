use nym_config::defaults::NymNetworkDetails;
use std::path::PathBuf;
use std::{
    net::{IpAddr, Ipv4Addr},
    time::Duration,
};

use nym_ip_packet_requests::{
    DynamicConnectResponse, IpPacketRequest, IpPacketResponse, IpPacketResponseData,
    StaticConnectResponse,
};
use nym_sdk::mixnet::{MixnetClient, MixnetClientBuilder, MixnetMessageSender, StoragePaths};
use tracing::{debug, error, info};

use crate::{
    error::{Error, Result},
    mixnet_processor::IpPacketRouterAddress,
};

async fn send_connect_to_ip_packet_router(
    mixnet_client: &mut MixnetClient,
    ip_packet_router_address: IpPacketRouterAddress,
    ip: Option<Ipv4Addr>,
    enable_two_hop: bool,
) -> Result<u64> {
    let hops = enable_two_hop.then_some(0);
    let (request, request_id) = if let Some(ip) = ip {
        debug!("Sending static connect request with ip: {ip}");
        IpPacketRequest::new_static_connect_request(
            ip.into(),
            *mixnet_client.nym_address(),
            hops,
            None,
        )
    } else {
        debug!("Sending dynamic connect request");
        IpPacketRequest::new_dynamic_connect_request(*mixnet_client.nym_address(), hops, None)
    };

    mixnet_client
        .send(nym_sdk::mixnet::InputMessage::new_regular_with_custom_hops(
            ip_packet_router_address.0,
            request.to_bytes().unwrap(),
            nym_task::connections::TransmissionLane::General,
            None,
            hops,
        ))
        .await?;

    Ok(request_id)
}

async fn wait_for_connect_response(
    mixnet_client: &mut MixnetClient,
    request_id: u64,
) -> Result<IpPacketResponse> {
    let timeout = tokio::time::sleep(Duration::from_secs(5));
    tokio::pin!(timeout);

    loop {
        tokio::select! {
            _ = &mut timeout => {
                error!("Timed out waiting for reply to connect request");
                return Err(Error::TimeoutWaitingForConnectResponse);
            }
            msgs = mixnet_client.wait_for_messages() => {
                if let Some(msgs) = msgs {
                    for msg in msgs {
                        debug!("MixnetProcessor: Got message while waiting for connect response");
                        let Ok(response) = IpPacketResponse::from_reconstructed_message(&msg) else {
                            error!("Failed to deserialize reconstructed message");
                            continue;
                        };
                        if response.id() == Some(request_id) {
                            info!("Got response with matching id");
                            return Ok(response);
                        }
                    }
                } else {
                    return Err(Error::NoMixnetMessagesReceived);
                }
            }
        }
    }
}

async fn handle_static_connect_response(
    mixnet_client: &mut MixnetClient,
    response: StaticConnectResponse,
) -> Result<()> {
    debug!("Handling static connect response");
    if response.reply_to != *mixnet_client.nym_address() {
        error!("Got reply intended for wrong address");
        return Err(Error::GotReplyIntendedForWrongAddress);
    }
    match response.reply {
        nym_ip_packet_requests::StaticConnectResponseReply::Success => Ok(()),
        nym_ip_packet_requests::StaticConnectResponseReply::Failure(reason) => {
            Err(Error::StaticConnectRequestDenied { reason })
        }
    }
}

async fn handle_dynamic_connect_response(
    mixnet_client: &mut MixnetClient,
    response: DynamicConnectResponse,
) -> Result<IpAddr> {
    debug!("Handling dynamic connect response");
    if response.reply_to != *mixnet_client.nym_address() {
        error!("Got reply intended for wrong address");
        return Err(Error::GotReplyIntendedForWrongAddress);
    }
    match response.reply {
        nym_ip_packet_requests::DynamicConnectResponseReply::Success(r) => Ok(r.ip),
        nym_ip_packet_requests::DynamicConnectResponseReply::Failure(reason) => {
            Err(Error::DynamicConnectRequestDenied { reason })
        }
    }
}

pub async fn connect_to_ip_packet_router(
    mixnet_client: &mut MixnetClient,
    ip_packet_router_address: IpPacketRouterAddress,
    ip: Option<Ipv4Addr>,
    enable_two_hop: bool,
) -> Result<IpAddr> {
    info!("Sending connect request");
    let request_id = send_connect_to_ip_packet_router(
        mixnet_client,
        ip_packet_router_address,
        ip,
        enable_two_hop,
    )
    .await?;

    info!("Waiting for reply...");
    let response = wait_for_connect_response(mixnet_client, request_id).await?;

    match response.data {
        IpPacketResponseData::StaticConnect(resp) if ip.is_some() => {
            handle_static_connect_response(mixnet_client, resp).await?;
            Ok(ip.unwrap().into())
        }
        IpPacketResponseData::DynamicConnect(resp) if ip.is_none() => {
            handle_dynamic_connect_response(mixnet_client, resp).await
        }
        response => {
            error!("Unexpected response: {:?}", response);
            Err(Error::UnexpectedConnectResponse)
        }
    }
}

pub(crate) async fn setup_mixnet_client(
    mixnet_entry_gateway: &str,
    mixnet_client_key_storage_path: &Option<PathBuf>,
    task_client: nym_task::TaskClient,
    enable_wireguard: bool,
    enable_two_hop: bool,
    enable_poisson_rate: bool,
) -> Result<MixnetClient> {
    // Disable Poisson rate limiter by default
    let mut debug_config = nym_client_core::config::DebugConfig::default();

    info!("mixnet client has Poisson rate limiting enabled: {enable_poisson_rate}");
    debug_config
        .traffic
        .disable_main_poisson_packet_distribution = !enable_poisson_rate;

    info!("mixnet client setup to send with two hops: {enable_two_hop}");
    // TODO: add support for two-hop mixnet traffic as a setting on the mixnet_client.
    // For now it's something we explicitly set on each set InputMessage.

    debug!("mixnet client has wireguard_mode={enable_wireguard}");
    let mixnet_client = if let Some(path) = mixnet_client_key_storage_path {
        debug!("Using custom key storage path: {:?}", path);
        let key_storage_path = StoragePaths::new_from_dir(path)?;
        MixnetClientBuilder::new_with_default_storage(key_storage_path)
            .await?
            .with_wireguard_mode(enable_wireguard)
            .request_gateway(mixnet_entry_gateway.to_string())
            .network_details(NymNetworkDetails::new_from_env())
            .debug_config(debug_config)
            .custom_shutdown(task_client)
            .build()?
            .connect_to_mixnet()
            .await?
    } else {
        debug!("Using ephemeral key storage");
        MixnetClientBuilder::new_ephemeral()
            .with_wireguard_mode(enable_wireguard)
            .request_gateway(mixnet_entry_gateway.to_string())
            .network_details(NymNetworkDetails::new_from_env())
            .debug_config(debug_config)
            .custom_shutdown(task_client)
            .build()?
            .connect_to_mixnet()
            .await?
    };

    Ok(mixnet_client)
}
