// Copyright 2023-2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::time::Duration;

use nym_ip_packet_requests::IpPair;
use nym_mixnet_client::SharedMixnetClient;
use nym_sdk::mixnet::{MixnetClientSender, MixnetMessageSender, Recipient, TransmissionLane};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error};

use crate::{
    error::{Error, Result},
    helpers::check_ipr_message_version,
    nym_ip_packet_requests_current::{
        request::IpPacketRequest,
        response::{
            DynamicConnectResponse, DynamicConnectResponseReply, IpPacketResponse,
            IpPacketResponseData, StaticConnectResponse, StaticConnectResponseReply,
        },
    },
};

const IPR_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, PartialEq, Eq)]
enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    #[allow(unused)]
    Disconnecting,
}

pub struct IprClientConnect {
    // During connection we need the mixnet client, but once connected we expect to setup a channel
    // from the main mixnet listener at the top-level.
    // As such, we drop the shared mixnet client once we're connected.
    mixnet_client: SharedMixnetClient,
    mixnet_sender: MixnetClientSender,
    nym_address: Recipient,
    connected: ConnectionState,
    cancel_token: CancellationToken,
}

impl IprClientConnect {
    pub async fn new(mixnet_client: SharedMixnetClient, cancel_token: CancellationToken) -> Self {
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
            cancel_token,
        }
    }

    pub async fn connect(
        &mut self,
        ip_packet_router_address: Recipient,
        ips: Option<IpPair>,
    ) -> Result<IpPair> {
        if self.connected != ConnectionState::Disconnected {
            return Err(Error::AlreadyConnected);
        }

        debug!("Sending connect request");
        self.connected = ConnectionState::Connecting;
        match self.connect_inner(ip_packet_router_address, ips).await {
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
        ip_packet_router_address: Recipient,
        ips: Option<IpPair>,
    ) -> Result<IpPair> {
        let request_id = self
            .send_connect_request(ip_packet_router_address, ips)
            .await?;

        debug!("Waiting for reply...");
        self.listen_for_connect_response(request_id, ips).await
    }

    async fn send_connect_request(
        &self,
        ip_packet_router_address: Recipient,
        ips: Option<IpPair>,
    ) -> Result<u64> {
        let (mut request, request_id) = if let Some(ips) = ips {
            debug!("Sending static connect request with ips: {ips}");
            IpPacketRequest::new_static_connect_request(ips, self.nym_address, None, None, None)
        } else {
            debug!("Sending dynamic connect request");
            IpPacketRequest::new_dynamic_connect_request(self.nym_address, None, None, None)
        };
        debug!("Sent connect request with version v{}", request.version);

        // With the request constructed, we need to sign it
        if let Some(Ok(data_to_sign)) = request.data.signable_request() {
            let signature = self.mixnet_client.sign(&data_to_sign).await;
            request.data.add_signature(signature);
        } else {
            error!("Failed to add signature to connect the request");
        }

        self.mixnet_sender
            .send(nym_sdk::mixnet::InputMessage::new_regular(
                ip_packet_router_address,
                request.to_bytes().unwrap(),
                TransmissionLane::General,
                None,
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

        let timeout = tokio::time::sleep(IPR_CONNECT_TIMEOUT);
        tokio::pin!(timeout);

        loop {
            tokio::select! {
                _ = self.cancel_token.cancelled() => {
                    error!("Cancelled while waiting for reply to connect request");
                    return Err(Error::Cancelled);
                },
                _ = &mut timeout => {
                    error!("Timed out waiting for reply to connect request");
                    return Err(Error::TimeoutWaitingForConnectResponse);
                },
                msgs = mixnet_client.wait_for_messages() => match msgs {
                    None => {
                        return Err(Error::NoMixnetMessagesReceived);
                    }
                    Some(msgs) => {
                        for msg in msgs {
                            // Confirm that the version is correct
                            if let Err(err) = check_ipr_message_version(&msg) {
                                tracing::info!("Mixnet message version mismatch: {err}");
                                continue;
                            }

                            // Then we deserialize the message
                            tracing::debug!("IprClient: got message while waiting for connect response");
                            let Ok(response) = IpPacketResponse::from_reconstructed_message(&msg) else {
                                // This is ok, it's likely just one of our self-pings
                                tracing::debug!("Failed to deserialize mixnet message");
                                continue;
                            };

                            if response.id() == Some(request_id) {
                                tracing::debug!("Got response with matching id");
                                return self.handle_ip_packet_router_response(response, ips).await;
                            }
                        }
                    }
                }
            }
        }
    }
}
