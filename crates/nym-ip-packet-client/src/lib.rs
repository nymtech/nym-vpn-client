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
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::oneshot;
use tracing::{debug, error, info};

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

#[derive(Debug, PartialEq, Eq)]
enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    #[allow(unused)]
    Disconnecting,
}

pub struct IprClient {
    mixnet_client: SharedMixnetClient,
    nym_address: Recipient,
    connected: ConnectionState,
}

impl IprClient {
    pub async fn new(mixnet_client: SharedMixnetClient) -> Self {
        let nym_address = *mixnet_client
            .inner()
            .lock()
            .await
            .as_ref()
            .unwrap()
            .nym_address();
        Self {
            mixnet_client,
            nym_address,
            connected: ConnectionState::Disconnected,
        }
    }

    pub async fn connect(
        &mut self,
        ip_packet_router_address: &IpPacketRouterAddress,
        ips: Option<IpPair>,
        enable_two_hop: bool,
    ) -> Result<IpPair> {
        if self.connected != ConnectionState::Disconnected {
            return Err(Error::AlreadyConnected);
        }

        debug!("Sending connect request");
        self.connected = ConnectionState::Connecting;
        match self
            .connect_inner(ip_packet_router_address, ips, enable_two_hop)
            .await
        {
            Ok(ips) => {
                debug!("Successfully connected to the ip-packet-router");
                self.connected = ConnectionState::Connected;
                Ok(ips)
            }
            Err(err) => {
                error!("Failed to connect to the ip-packet-router: {:?}", err);
                self.connected = ConnectionState::Disconnected;
                Err(err)
            }
        }
    }

    async fn connect_inner(
        &mut self,
        ip_packet_router_address: &IpPacketRouterAddress,
        ips: Option<IpPair>,
        enable_two_hop: bool,
    ) -> Result<IpPair> {
        let request_id = self
            .send_connect_request(ip_packet_router_address, ips, enable_two_hop)
            .await?;

        // WIP(JON): we spawn a short lived mixnet listener task here while we gradually implement
        // all aspects of the IPR client. The correct thing is for the top-level application to start
        // the listener and use the same one for the duration of the application.
        debug!("Waiting for reply...");
        let mixnet_client = self.mixnet_client.clone();
        let (outbound_mix_message_tx, outbound_mix_message_rx) = tokio::sync::mpsc::channel(16);
        let (should_stop_tx, should_stop_rx) = tokio::sync::oneshot::channel();
        tokio::task::spawn(async move {
            start_mixnet_listener(mixnet_client, outbound_mix_message_tx, should_stop_rx)
                .await
                .unwrap();
        });

        let response = self
            .listen_for_connect_response(outbound_mix_message_rx, request_id, ips)
            .await;
        should_stop_tx.send(()).unwrap();
        response
    }

    async fn send_connect_request(
        &self,
        ip_packet_router_address: &IpPacketRouterAddress,
        ips: Option<IpPair>,
        enable_two_hop: bool,
    ) -> Result<u64> {
        let hops = enable_two_hop.then_some(0);
        // let mixnet_client_address = self.mixnet_client.nym_address().await;
        let (request, request_id) = if let Some(ips) = ips {
            debug!("Sending static connect request with ips: {ips}");
            IpPacketRequest::new_static_connect_request(ips, self.nym_address, hops, None, None)
        } else {
            debug!("Sending dynamic connect request");
            // IpPacketRequest::new_dynamic_connect_request(mixnet_client_address, hops, None, None)
            IpPacketRequest::new_dynamic_connect_request(self.nym_address, hops, None, None)
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

    async fn handle_static_connect_response(&self, response: StaticConnectResponse) -> Result<()> {
        debug!("Handling static connect response");
        // let mixnet_client_address = self.mixnet_client.nym_address().await;
        if response.reply_to != self.nym_address {
            error!("Got reply intended for wrong address");
            return Err(Error::GotReplyIntendedForWrongAddress);
        }
        match response.reply {
            StaticConnectResponseReply::Success => Ok(()),
            StaticConnectResponseReply::Failure(reason) => {
                Err(Error::StaticConnectRequestDenied { reason })
            }
        }
    }

    async fn handle_dynamic_connect_response(
        &self,
        response: DynamicConnectResponse,
    ) -> Result<IpPair> {
        debug!("Handling dynamic connect response");
        // let mixnet_client_address = self.mixnet_client.nym_address().await;
        if response.reply_to != self.nym_address {
            error!("Got reply intended for wrong address");
            return Err(Error::GotReplyIntendedForWrongAddress);
        }
        match response.reply {
            DynamicConnectResponseReply::Success(r) => Ok(r.ips),
            DynamicConnectResponseReply::Failure(reason) => {
                Err(Error::DynamicConnectRequestDenied { reason })
            }
        }
    }

    async fn handle_ip_packet_router_response(
        &self,
        response: IpPacketResponse,
        ips: Option<IpPair>,
    ) -> Result<IpPair> {
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

    pub async fn listen_for_connect_response(
        &self,
        mut outbound_mix_message_rx: Receiver<IpPacketResponse>,
        request_id: u64,
        ips: Option<IpPair>,
    ) -> Result<IpPair> {
        let timeout = tokio::time::sleep(Duration::from_secs(5));
        tokio::pin!(timeout);

        loop {
            tokio::select! {
                _ = &mut timeout => {
                    error!("Timed out waiting for reply to connect request");
                    return Err(Error::TimeoutWaitingForConnectResponse);
                }
                response = outbound_mix_message_rx.recv() => {
                    match response {
                        None => {
                            error!("Channel closed while waiting for response");
                            panic!();
                        }
                        Some(response) => {
                            if response.id() == Some(request_id) {
                                debug!("Got response with matching id");
                                return self.handle_ip_packet_router_response(response, ips).await;
                            }
                        }
                    }
                }
            }
        }
    }
}

async fn start_mixnet_listener(
    mixnet_client: SharedMixnetClient,
    outbound_mix_message_tx: Sender<IpPacketResponse>,
    mut should_stop: oneshot::Receiver<()>,
) -> Result<()> {
    // Connecting is basically synchronous from the perspective of the mixnet client, so it's safe
    // to just grab ahold of the mutex and keep it until we get the response.
    let mut mixnet_client_handle = mixnet_client.lock().await;
    let mixnet_client = mixnet_client_handle.as_mut().unwrap();

    loop {
        tokio::select! {
            _ = &mut should_stop => {
                info!("Instructed to stop the mixnet listener");
                return Ok(());
            }
            msgs = mixnet_client.wait_for_messages() => {
                if let Some(msgs) = msgs {
                    for msg in msgs {
                        // Confirm that the version is correct
                        check_ipr_message_version(&msg)?;

                        // Then we deserialize the message
                        debug!("MixnetProcessor: Got message while waiting for connect response");
                        let Ok(response) = IpPacketResponse::from_reconstructed_message(&msg) else {
                            // This is ok, it's likely just one of our self-pings
                            debug!("Failed to deserialize reconstructed message");
                            continue;
                        };

                        // The we forward it to the IPR client
                        outbound_mix_message_tx.send(response).await.unwrap();
                    }
                } else {
                    return Err(Error::NoMixnetMessagesReceived);
                }
            }
        }
    }
}

fn check_ipr_message_version(message: &ReconstructedMessage) -> Result<()> {
    // Assuing it's a IPR message, it will have a version as its first byte
    if let Some(version) = message.message.first() {
        match version.cmp(&nym_ip_packet_requests::CURRENT_VERSION) {
            Ordering::Greater => Err(Error::ReceivedResponseWithNewVersion {
                expected: nym_ip_packet_requests::CURRENT_VERSION,
                received: *version,
            }),
            Ordering::Less => Err(Error::ReceivedResponseWithOldVersion {
                expected: nym_ip_packet_requests::CURRENT_VERSION,
                received: *version,
            }),
            Ordering::Equal => {
                // We're good
                Ok(())
            }
        }
    } else {
        Err(Error::NoVersionInMessage)
    }
}
