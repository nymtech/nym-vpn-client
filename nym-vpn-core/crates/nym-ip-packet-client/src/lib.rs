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
    MixnetClient, MixnetClientSender, MixnetMessageSender, Recipient, ReconstructedMessage,
    TransmissionLane,
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

    pub async fn send(&self, msg: nym_sdk::mixnet::InputMessage) -> Result<()> {
        self.lock().await.as_mut().unwrap().send(msg).await?;
        Ok(())
    }

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
    // During connection we need the mixnet client, but once connected we expect to setup a channel
    // from the main mixnet listener at the top-level.
    // As such, we drop the shared mixnet client once we're connected.
    mixnet_client: SharedMixnetClient,
    mixnet_sender: MixnetClientSender,
    nym_address: Recipient,
    connected: ConnectionState,
}

impl IprClient {
    pub async fn new(mixnet_client: SharedMixnetClient) -> Self {
        let mixnet_sender = mixnet_client.lock().await.as_ref().unwrap().split_sender();
        let nym_address = *mixnet_client
            .inner()
            .lock()
            .await
            .as_ref()
            .unwrap()
            .nym_address();
        Self {
            mixnet_client,
            mixnet_sender,
            nym_address,
            connected: ConnectionState::Disconnected,
        }
    }

    // A workaround until we can extract SharedMixnetClient to a common crate
    pub async fn new_from_inner(
        mixnet_client: Arc<tokio::sync::Mutex<Option<MixnetClient>>>,
    ) -> Self {
        let mixnet_client = SharedMixnetClient(mixnet_client);
        Self::new(mixnet_client).await
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

        debug!("Waiting for reply...");
        self.listen_for_connect_response(request_id, ips).await
    }

    async fn send_connect_request(
        &self,
        ip_packet_router_address: &IpPacketRouterAddress,
        ips: Option<IpPair>,
        enable_two_hop: bool,
    ) -> Result<u64> {
        let hops = enable_two_hop.then_some(0);
        let (request, request_id) = if let Some(ips) = ips {
            debug!("Sending static connect request with ips: {ips}");
            IpPacketRequest::new_static_connect_request(ips, self.nym_address, hops, None, None)
        } else {
            debug!("Sending dynamic connect request");
            IpPacketRequest::new_dynamic_connect_request(self.nym_address, hops, None, None)
        };
        debug!("Sent connect request with version v{}", request.version);

        self.mixnet_sender
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

    async fn listen_for_connect_response(
        &self,
        request_id: u64,
        ips: Option<IpPair>,
    ) -> Result<IpPair> {
        // Connecting is basically synchronous from the perspective of the mixnet client, so it's safe
        // to just grab ahold of the mutex and keep it until we get the response.
        let mut mixnet_client_handle = self.mixnet_client.lock().await;
        let mixnet_client = mixnet_client_handle.as_mut().unwrap();

        let timeout = tokio::time::sleep(Duration::from_secs(5));
        tokio::pin!(timeout);

        loop {
            tokio::select! {
                _ = &mut timeout => {
                    error!("Timed out waiting for reply to connect request");
                    return Err(Error::TimeoutWaitingForConnectResponse);
                }
                msgs = mixnet_client.wait_for_messages() => match msgs {
                    None => {
                        return Err(Error::NoMixnetMessagesReceived);
                    }
                    Some(msgs) => {
                        for msg in msgs {
                            // Confirm that the version is correct
                            check_ipr_message_version(&msg)?;

                            // Then we deserialize the message
                            debug!("IprClient: got message while waiting for connect response");
                            let Ok(response) = IpPacketResponse::from_reconstructed_message(&msg) else {
                                // This is ok, it's likely just one of our self-pings
                                debug!("Failed to deserialize reconstructed message");
                                continue;
                            };

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
