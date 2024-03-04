// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_config::defaults::NymNetworkDetails;
use std::os::fd::RawFd;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use nym_ip_packet_requests::IpPair;
use nym_ip_packet_requests::{
    request::IpPacketRequest,
    response::{
        DynamicConnectResponse, IpPacketResponse, IpPacketResponseData, StaticConnectResponse,
    },
};
use nym_sdk::mixnet::{
    MixnetClient, MixnetClientBuilder, MixnetMessageSender, NodeIdentity, Recipient, StoragePaths,
};
use tracing::{debug, error, info};

use crate::{
    error::{Error, Result},
    mixnet_processor::IpPacketRouterAddress,
};

#[derive(Clone)]
pub struct SharedMixnetClient(Arc<tokio::sync::Mutex<Option<MixnetClient>>>);

impl SharedMixnetClient {
    pub fn new(mixnet_client: MixnetClient) -> Self {
        Self(Arc::new(tokio::sync::Mutex::new(Some(mixnet_client))))
    }

    pub async fn lock(&self) -> tokio::sync::MutexGuard<'_, Option<MixnetClient>> {
        self.0.lock().await
    }

    pub async fn nym_address(&self) -> Recipient {
        *self.lock().await.as_ref().unwrap().nym_address()
    }

    pub async fn gateway_ws_fd(&self) -> Option<RawFd> {
        self.lock()
            .await
            .as_ref()
            .unwrap()
            .gateway_connection()
            .gateway_ws_fd
    }

    pub async fn send(&self, msg: nym_sdk::mixnet::InputMessage) -> Result<()> {
        self.lock().await.as_mut().unwrap().send(msg).await?;
        Ok(())
    }

    pub async fn disconnect(self) -> Self {
        let handle = self.lock().await.take().unwrap();
        handle.disconnect().await;
        self
    }
}

async fn send_connect_to_ip_packet_router(
    mixnet_client: &SharedMixnetClient,
    ip_packet_router_address: &IpPacketRouterAddress,
    ips: Option<IpPair>,
    enable_two_hop: bool,
) -> Result<u64> {
    let hops = enable_two_hop.then_some(0);
    let mixnet_client_address = mixnet_client.nym_address().await;
    let (request, request_id) = if let Some(ips) = ips {
        debug!("Sending static connect request with ips: {ips}");
        IpPacketRequest::new_static_connect_request(ips, mixnet_client_address, hops, None, None)
    } else {
        debug!("Sending dynamic connect request");
        IpPacketRequest::new_dynamic_connect_request(mixnet_client_address, hops, None, None)
    };
    debug!("Sent connect request with version v{}", request.version);

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
    mixnet_client: &SharedMixnetClient,
    request_id: u64,
) -> Result<IpPacketResponse> {
    let timeout = tokio::time::sleep(Duration::from_secs(5));
    tokio::pin!(timeout);

    // Connecting is basically synchronous from the perspective of the mixnet client, so it's safe
    // to just grab ahold of the mutex and keep it until we get the response.
    let mut mixnet_client_handle = mixnet_client.lock().await;
    let mixnet_client = mixnet_client_handle.as_mut().unwrap();

    loop {
        tokio::select! {
            _ = &mut timeout => {
                error!("Timed out waiting for reply to connect request");
                return Err(Error::TimeoutWaitingForConnectResponse);
            }
            msgs = mixnet_client.wait_for_messages() => {
                if let Some(msgs) = msgs {
                    for msg in msgs {

                        // Handle if the response is from an IPR running an older or newer version
                        if let Some(version) = msg.message.first() {
                            if *version != nym_ip_packet_requests::CURRENT_VERSION {
                                log::error!("Received packet with invalid version: v{version}, is your client up to date?");
                                return Err(Error::InvalidVersion {
                                    expected: nym_ip_packet_requests::CURRENT_VERSION,
                                    received: *version,
                                });
                            }
                        }

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
    mixnet_client_address: &Recipient,
    response: StaticConnectResponse,
) -> Result<()> {
    debug!("Handling static connect response");
    if response.reply_to != *mixnet_client_address {
        error!("Got reply intended for wrong address");
        return Err(Error::GotReplyIntendedForWrongAddress);
    }
    match response.reply {
        nym_ip_packet_requests::response::StaticConnectResponseReply::Success => Ok(()),
        nym_ip_packet_requests::response::StaticConnectResponseReply::Failure(reason) => {
            Err(Error::StaticConnectRequestDenied { reason })
        }
    }
}

async fn handle_dynamic_connect_response(
    mixnet_client_address: &Recipient,
    response: DynamicConnectResponse,
) -> Result<IpPair> {
    debug!("Handling dynamic connect response");
    if response.reply_to != *mixnet_client_address {
        error!("Got reply intended for wrong address");
        return Err(Error::GotReplyIntendedForWrongAddress);
    }
    match response.reply {
        nym_ip_packet_requests::response::DynamicConnectResponseReply::Success(r) => Ok(r.ips),
        nym_ip_packet_requests::response::DynamicConnectResponseReply::Failure(reason) => {
            Err(Error::DynamicConnectRequestDenied { reason })
        }
    }
}

pub async fn connect_to_ip_packet_router(
    mixnet_client: SharedMixnetClient,
    ip_packet_router_address: &IpPacketRouterAddress,
    ips: Option<IpPair>,
    enable_two_hop: bool,
) -> Result<IpPair> {
    info!("Sending connect request");
    let request_id = send_connect_to_ip_packet_router(
        &mixnet_client,
        ip_packet_router_address,
        ips,
        enable_two_hop,
    )
    .await?;

    info!("Waiting for reply...");
    let response = wait_for_connect_response(&mixnet_client, request_id).await?;

    let mixnet_client_address = mixnet_client.nym_address().await;
    match response.data {
        IpPacketResponseData::StaticConnect(resp) if ips.is_some() => {
            handle_static_connect_response(&mixnet_client_address, resp).await?;
            Ok(ips.unwrap())
        }
        IpPacketResponseData::DynamicConnect(resp) if ips.is_none() => {
            handle_dynamic_connect_response(&mixnet_client_address, resp).await
        }
        response => {
            error!("Unexpected response: {:?}", response);
            Err(Error::UnexpectedConnectResponse)
        }
    }
}

fn true_to_enabled(val: bool) -> &'static str {
    if val {
        "enabled"
    } else {
        "disabled"
    }
}

fn true_to_disabled(val: bool) -> &'static str {
    if val {
        "disabled"
    } else {
        "enabled"
    }
}

pub(crate) async fn setup_mixnet_client(
    mixnet_entry_gateway: &NodeIdentity,
    mixnet_client_key_storage_path: &Option<PathBuf>,
    task_client: nym_task::TaskClient,
    enable_wireguard: bool,
    enable_two_hop: bool,
    enable_poisson_rate: bool,
    disable_background_cover_traffic: bool,
) -> Result<SharedMixnetClient> {
    // Disable Poisson rate limiter by default
    let mut debug_config = nym_client_core::config::DebugConfig::default();

    info!(
        "mixnet client poisson rate limiting: {}",
        true_to_enabled(enable_poisson_rate)
    );
    debug_config
        .traffic
        .disable_main_poisson_packet_distribution = !enable_poisson_rate;

    info!(
        "mixnet client background loop cover traffic stream: {}",
        true_to_disabled(disable_background_cover_traffic)
    );
    debug_config.cover_traffic.disable_loop_cover_traffic_stream = disable_background_cover_traffic;

    info!(
        "mixnet client two hop traffic: {}",
        true_to_enabled(enable_two_hop)
    );
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

    Ok(SharedMixnetClient::new(mixnet_client))
}
