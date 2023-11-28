use std::{net::IpAddr, time::Duration};

use nym_ip_packet_requests::{
    IpPacketRequest, IpPacketResponse, IpPacketResponseData, StaticConnectResponse,
};
use nym_sdk::mixnet::{MixnetClient, MixnetMessageSender};
use tracing::{error, info, debug};

use crate::{
    error::{Error, Result},
    mixnet_processor::IpPacketRouterAddress,
};

async fn send_connect_to_ip_packet_router(
    mixnet_client: &mut MixnetClient,
    ip_packet_router_address: IpPacketRouterAddress,
    ip: Option<IpAddr>,
) -> Result<u64> {
    let (request, request_id) = if let Some(ip) = ip {
        IpPacketRequest::new_static_connect_request(ip, *mixnet_client.nym_address(), None, None)
    } else {
        IpPacketRequest::new_dynamic_connect_request(*mixnet_client.nym_address(), None, None)
    };

    mixnet_client
        .send(nym_sdk::mixnet::InputMessage::new_regular(
            ip_packet_router_address.0,
            request.to_bytes().unwrap(),
            nym_task::connections::TransmissionLane::General,
            None,
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
    if response.reply_to != *mixnet_client.nym_address() {
        error!("Got reply intended for wrong address");
        return Err(Error::GotReplyIntendedForWrongAddress);
    }
    match response.reply {
        nym_ip_packet_requests::StaticConnectResponseReply::Success => Ok(()),
        nym_ip_packet_requests::StaticConnectResponseReply::Failure(reason) => {
            Err(Error::ConnectRequestDenied {
                reason: Some(reason),
            })
        }
    }
}

pub async fn connect_to_ip_packet_router(
    mixnet_client: &mut MixnetClient,
    ip_packet_router_address: IpPacketRouterAddress,
    ip: IpAddr,
) -> Result<()> {
    info!("Sending static connect request");
    let request_id =
        send_connect_to_ip_packet_router(mixnet_client, ip_packet_router_address, Some(ip)).await?;

    info!("Waiting for reply...");
    let response = wait_for_connect_response(mixnet_client, request_id).await?;

    match response.data {
        IpPacketResponseData::StaticConnect(resp) => {
            handle_static_connect_response(mixnet_client, resp).await
        }
        IpPacketResponseData::DynamicConnect(_) => {
            error!("Requested static connect, but got dynamic connect response!");
            Err(Error::UnexpectedConnectResponse)
        }
        IpPacketResponseData::Data(_) => {
            error!("Requested static connect, but got data response!");
            Err(Error::UnexpectedConnectResponse)
        }
    }
}
