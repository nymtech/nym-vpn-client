use std::cmp::Ordering;
use std::sync::Arc;
use std::time::Duration;

use nym_ip_packet_requests::response::{DynamicConnectResponseReply, StaticConnectResponseReply};
use nym_ip_packet_requests::IpPair;
use nym_ip_packet_requests::{
    request::IpPacketRequest,
    response::{
        DynamicConnectResponse, IpPacketResponse, IpPacketResponseData, StaticConnectResponse,
    },
};
use nym_sdk::mixnet::{
    MixnetClient, MixnetMessageSender, Recipient, ReconstructedMessage, TransmissionLane,
};
use tracing::{debug, error};

use nym_gateway_directory::IpPacketRouterAddress;

mod error;

pub use crate::error::{Error, Result};

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

    // pub async fn split_sender(&self) -> MixnetClientSender {
    //     self.lock().await.as_ref().unwrap().split_sender()
    // }

    // pub async fn gateway_ws_fd(&self) -> Option<RawFd> {
    //     self.lock()
    //         .await
    //         .as_ref()
    //         .unwrap()
    //         .gateway_connection()
    //         .gateway_ws_fd
    // }

    pub async fn send(&self, msg: nym_sdk::mixnet::InputMessage) -> Result<()> {
        self.lock().await.as_mut().unwrap().send(msg).await?;
        Ok(())
    }

    // pub async fn disconnect(self) -> Self {
    //     let handle = self.lock().await.take().unwrap();
    //     handle.disconnect().await;
    //     self
    // }

    pub fn inner(&self) -> Arc<tokio::sync::Mutex<Option<MixnetClient>>> {
        self.0.clone()
    }
}

pub struct IprClient {
    mixnet_client: SharedMixnetClient,
    connected: bool,
    // incoming_messages: Receiver<ReconstructedMessage>,
}

impl IprClient {
    pub fn new(mixnet_client: SharedMixnetClient) -> Self {
        Self {
            mixnet_client,
            connected: false,
            // incoming_messages: todo!(),
        }
    }

    pub async fn connect(
        &mut self,
        ip_packet_router_address: &IpPacketRouterAddress,
        ips: Option<IpPair>,
        enable_two_hop: bool,
    ) -> Result<IpPair> {
        if self.connected {
            return Err(Error::AlreadyConnected);
        }

        debug!("Sending connect request");
        let request_id = self
            .send_connect_request(ip_packet_router_address, ips, enable_two_hop)
            .await?;

        debug!("Waiting for reply...");
        let response = self.wait_for_connect_response(request_id).await?;

        match response.data {
            IpPacketResponseData::StaticConnect(resp) if ips.is_some() => {
                self.handle_static_connect_response(resp).await?;
                Ok(ips.unwrap())
            }
            IpPacketResponseData::DynamicConnect(resp) if ips.is_none() => {
                self.handle_dynamic_connect_response(resp).await
            }
            response => {
                error!("Unexpected response: {:?}", response);
                Err(Error::UnexpectedConnectResponse)
            }
        }
    }

    async fn send_connect_request(
        &self,
        ip_packet_router_address: &IpPacketRouterAddress,
        ips: Option<IpPair>,
        enable_two_hop: bool,
    ) -> Result<u64> {
        let hops = enable_two_hop.then_some(0);
        let mixnet_client_address = self.mixnet_client.nym_address().await;
        let (request, request_id) = if let Some(ips) = ips {
            debug!("Sending static connect request with ips: {ips}");
            IpPacketRequest::new_static_connect_request(
                ips,
                mixnet_client_address,
                hops,
                None,
                None,
            )
        } else {
            debug!("Sending dynamic connect request");
            IpPacketRequest::new_dynamic_connect_request(mixnet_client_address, hops, None, None)
        };
        debug!("Sent connect request with version v{}", request.version);

        self.mixnet_client
            .send(nym_sdk::mixnet::InputMessage::new_regular_with_custom_hops(
                ip_packet_router_address.0,
                request.to_bytes().unwrap(),
                TransmissionLane::General,
                None,
                hops,
            ))
            .await?;

        Ok(request_id)
    }

    fn check_ipr_message_version(&self, message: &ReconstructedMessage) -> Result<()> {
        // Assuing it's a IPR message, it will have a version as its first byte
        if let Some(version) = message.message.first() {
            match version.cmp(&nym_ip_packet_requests::CURRENT_VERSION) {
                Ordering::Greater => {
                    error!(
                        "Received packet with newer version: v{version}, \
                        is your client up to date?"
                    );
                    Err(Error::ReceivedResponseWithNewVersion {
                        expected: nym_ip_packet_requests::CURRENT_VERSION,
                        received: *version,
                    })
                }
                Ordering::Less => {
                    error!(
                        "Received packet with older version: v{version}, you client appears \
                        to be too new for the exit gateway or exit ip-packet-router?"
                    );
                    Err(Error::ReceivedResponseWithOldVersion {
                        expected: nym_ip_packet_requests::CURRENT_VERSION,
                        received: *version,
                    })
                }
                Ordering::Equal => {
                    // We're good
                    Ok(())
                }
            }
        } else {
            Err(Error::NoVersionInMessage)
        }
    }

    async fn wait_for_connect_response(&self, request_id: u64) -> Result<IpPacketResponse> {
        let timeout = tokio::time::sleep(Duration::from_secs(5));
        tokio::pin!(timeout);

        // Connecting is basically synchronous from the perspective of the mixnet client, so it's safe
        // to just grab ahold of the mutex and keep it until we get the response.
        let mut mixnet_client_handle = self.mixnet_client.lock().await;
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
                            self.check_ipr_message_version(&msg)?;

                            debug!("MixnetProcessor: Got message while waiting for connect response");
                            let Ok(response) = IpPacketResponse::from_reconstructed_message(&msg) else {
                                // This is ok, it's likely just one of our self-pings
                                debug!("Failed to deserialize reconstructed message");
                                continue;
                            };
                            if response.id() == Some(request_id) {
                                debug!("Got response with matching id");
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
        &mut self,
        response: StaticConnectResponse,
    ) -> Result<()> {
        debug!("Handling static connect response");
        let mixnet_client_address = self.mixnet_client.nym_address().await;
        if response.reply_to != mixnet_client_address {
            error!("Got reply intended for wrong address");
            return Err(Error::GotReplyIntendedForWrongAddress);
        }
        match response.reply {
            StaticConnectResponseReply::Success => {
                self.connected = true;
                Ok(())
            }
            StaticConnectResponseReply::Failure(reason) => {
                Err(Error::StaticConnectRequestDenied { reason })
            }
        }
    }

    async fn handle_dynamic_connect_response(
        &mut self,
        response: DynamicConnectResponse,
    ) -> Result<IpPair> {
        debug!("Handling dynamic connect response");
        let mixnet_client_address = self.mixnet_client.nym_address().await;
        if response.reply_to != mixnet_client_address {
            error!("Got reply intended for wrong address");
            return Err(Error::GotReplyIntendedForWrongAddress);
        }
        match response.reply {
            DynamicConnectResponseReply::Success(r) => {
                self.connected = true;
                Ok(r.ips)
            }
            DynamicConnectResponseReply::Failure(reason) => {
                Err(Error::DynamicConnectRequestDenied { reason })
            }
        }
    }

    #[allow(dead_code)]
    pub async fn listen_for_ip_packet_router_responses(&self) -> Result<()> {
        todo!()
    }
}
