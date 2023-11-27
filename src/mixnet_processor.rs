// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>

use std::{
    net::{IpAddr, Ipv4Addr},
    time::Duration,
};

use futures::{SinkExt, StreamExt};
use nym_ip_packet_requests::{
    IpPacketRequest, IpPacketResponse, IpPacketResponseData, StaticConnectResponse,
};
use nym_sdk::mixnet::{IncludedSurbs, MixnetClient, MixnetMessageSender, Recipient};
use nym_task::{TaskClient, TaskManager};
use tracing::{debug, error, info, trace, warn};
use tun::{AsyncDevice, Device, TunPacket};

use crate::error::{Error, Result};

#[derive(Debug)]
pub struct Config {
    pub ip_packet_router_address: IpPacketRouterAddress,
}

impl Config {
    pub fn new(ip_packet_router_address: IpPacketRouterAddress) -> Self {
        Config {
            ip_packet_router_address,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct IpPacketRouterAddress(pub Recipient);

impl IpPacketRouterAddress {
    pub fn try_from_base58_string(ip_packet_router_nym_address: &str) -> Result<Self> {
        Ok(Self(
            Recipient::try_from_base58_string(ip_packet_router_nym_address)
                .map_err(|_| Error::RecipientFormattingError)?,
        ))
    }
}

impl std::fmt::Display for IpPacketRouterAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub struct MixnetProcessor {
    device: AsyncDevice,
    mixnet_client: MixnetClient,
    ip_packet_router_address: IpPacketRouterAddress,
}

impl MixnetProcessor {
    pub fn new(
        device: AsyncDevice,
        mixnet_client: MixnetClient,
        ip_packet_router_address: IpPacketRouterAddress,
    ) -> Self {
        MixnetProcessor {
            device,
            mixnet_client,
            ip_packet_router_address,
        }
    }

    async fn send_connect_to_ip_packet_router(&mut self, ip: IpAddr) -> Result<u64> {
        let (request, request_id) = IpPacketRequest::new_static_connect_request(
            ip,
            *self.mixnet_client.nym_address(),
            None,
            None,
        );
        self.mixnet_client
            .send(nym_sdk::mixnet::InputMessage::new_regular(
                self.ip_packet_router_address.0,
                request.to_bytes().unwrap(),
                nym_task::connections::TransmissionLane::General,
                None,
            ))
            .await?;
        Ok(request_id)
    }

    async fn wait_for_connect_response(&mut self, request_id: u64) -> Result<IpPacketResponse> {
        let timeout = tokio::time::sleep(Duration::from_secs(5));
        tokio::pin!(timeout);

        loop {
            tokio::select! {
                _ = &mut timeout => {
                    error!("MixnetProcessor: Timed out waiting for reply");
                    return Err(Error::TimeoutWaitingForConnectResponse);
                }
                msgs = self.mixnet_client.wait_for_messages() => {
                    if let Some(msgs) = msgs {
                        for msg in msgs {
                            debug!("MixnetProcessor: Got message while waiting for connect response");
                            let Ok(response) = IpPacketResponse::from_reconstructed_message(&msg) else {
                                error!("MixnetProcessor: Failed to deserialize reconstructed message");
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
        &self,
        response: StaticConnectResponse,
    ) -> Result<bool> {
        if response.reply_to != *self.mixnet_client.nym_address() {
            error!("Got reply intended for wrong address");
            return Err(Error::InvalidGatewayAPIResponse);
        }
        Ok(response.reply.is_success())
    }

    async fn connect_to_ip_packet_router(&mut self, ip: IpAddr) -> Result<()> {
        info!("Sending static connect request");
        let request_id = self.send_connect_to_ip_packet_router(ip).await?;

        info!("Waiting for reply...");
        let response = self.wait_for_connect_response(request_id).await?;

        match response.data {
            IpPacketResponseData::StaticConnect(resp) => {
                if self.handle_static_connect_response(resp).await? {
                    debug!("Static connect successful");
                    Ok(())
                } else {
                    debug!("Static connect denied");
                    Err(Error::ConnectDenied)
                }
            }
            IpPacketResponseData::DynamicConnect(_) => {
                error!("Requested static connect, but got dynamic connect response!");
                Err(Error::UnexpectedConnectResponse)
            }
            IpPacketResponseData::Data(_) => {
                unreachable!()
            }
        }
    }

    pub async fn run(mut self, mut shutdown: TaskClient) {
        info!("Connecting to IP packet router");
        let ip = Ipv4Addr::new(10, 0, 0, 2).into();
        if let Err(_err) = self.connect_to_ip_packet_router(ip).await {
            // It is not yet implemented on the server to return anything but deny, so we just
            // ignore it for now....

            //error!("Failed to connect to IP packet router: {err}");
            //debug!("{err:?}");
            //TODO: signal back to the main task to shutdown cleanly
            //return;
        }

        info!(
            "Opened mixnet processor on tun device {}",
            self.device.get_ref().name()
        );
        let (mut sink, mut stream) = self.device.into_framed().split();
        let sender = self.mixnet_client.split_sender();
        let recipient = self.ip_packet_router_address;

        let mixnet_stream = self
            .mixnet_client
            .filter_map(|reconstructed_message| async move {
                match IpPacketResponse::from_reconstructed_message(&reconstructed_message) {
                    Ok(response) => match response.data {
                        IpPacketResponseData::StaticConnect(_) => {
                            info!("Received static connect response when already connected - ignoring");
                            None
                        },
                        IpPacketResponseData::DynamicConnect(_) => {
                            info!("Received dynamic connect response when already connected - ignoring");
                            None
                        },
                        IpPacketResponseData::Data(data_response) => {
                            Some(Ok(TunPacket::new(data_response.ip_packet.into())))
                        }
                    },
                    Err(err) => {
                        error!("failed to deserialize reconstructed message: {err}");
                        None
                    }
                }
            });
        tokio::pin!(mixnet_stream);

        while !shutdown.is_shutdown() {
            tokio::select! {
                _ = shutdown.recv_with_delay() => {
                    trace!("MixnetProcessor: Received shutdown");
                }
                Some(Ok(packet)) = stream.next() => {
                    // TODO: properly investigate the binary format here and the overheard
                    let Ok(packet) = IpPacketRequest::new_ip_packet(packet.into_bytes()).to_bytes() else {
                        error!("Failed to serialize packet");
                        continue;
                    };

                    // The enum here about IncludedSurbs and ExposeSelfAddress is misleading. It is
                    // not being used. Basically IncludedSurbs::ExposeSelfAddress just omits the
                    // surbs, assuming that it is exposed inside the message. (This is the case
                    // for SOCKS5 too).
                    let ret = sender.send_message(recipient.0, &packet, IncludedSurbs::ExposeSelfAddress).await;
                    if ret.is_err() && !shutdown.is_shutdown_poll() {
                        error!("Could not forward IP packet to the mixnet. The packet will be dropped.");
                    }
                }
                res = sink.send_all(&mut mixnet_stream) => {
                    warn!("Mixnet stream finished. This may mean that the gateway was shut down");
                    if let Err(e) = res {
                        error!("Could not forward mixnet traffic to the client - {:?}", e);
                    }
                    break;
                }
            }
        }
        debug!("MixnetProcessor: Exiting");
    }
}

pub async fn start_processor(
    config: Config,
    dev: tun::AsyncDevice,
    mixnet_client: MixnetClient,
    task_manager: &TaskManager,
) -> Result<()> {
    info!("Creating mixnet processor");
    let processor = MixnetProcessor::new(dev, mixnet_client, config.ip_packet_router_address);
    let shutdown_listener = task_manager.subscribe();
    tokio::spawn(processor.run(shutdown_listener));
    Ok(())
}
