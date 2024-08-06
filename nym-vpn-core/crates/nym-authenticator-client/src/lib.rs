use std::{cmp::Ordering, sync::Arc, time::Duration};

use nym_authenticator_requests::v1::{
    request::AuthenticatorRequest, response::AuthenticatorResponse,
};
use nym_sdk::mixnet::{
    MixnetClient, MixnetClientSender, MixnetMessageSender, Recipient, ReconstructedMessage,
    TransmissionLane,
};
use nym_wireguard_types::ClientMessage;
use tracing::{debug, error};

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

#[derive(Clone)]
pub struct AuthClient {
    mixnet_client: SharedMixnetClient,
    mixnet_sender: MixnetClientSender,
    nym_address: Recipient,
}

impl AuthClient {
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
        }
    }

    // A workaround until we can extract SharedMixnetClient to a common crate
    pub async fn new_from_inner(
        mixnet_client: Arc<tokio::sync::Mutex<Option<MixnetClient>>>,
    ) -> Self {
        let mixnet_client = SharedMixnetClient(mixnet_client);
        Self::new(mixnet_client).await
    }

    pub async fn send(
        &mut self,
        message: ClientMessage,
        authenticator_address: Recipient,
    ) -> Result<AuthenticatorResponse> {
        self.send_inner(message, authenticator_address).await
    }

    async fn send_inner(
        &mut self,
        message: ClientMessage,
        authenticator_address: Recipient,
    ) -> Result<AuthenticatorResponse> {
        // Connecting is basically synchronous from the perspective of the mixnet client, so it's safe
        // to just grab ahold of the mutex and keep it until we get the response.
        let mut mixnet_client_handle = self.mixnet_client.lock().await;
        let request_id = self
            .send_connect_request(message, authenticator_address)
            .await?;

        debug!("Waiting for reply...");
        self.listen_for_connect_response(request_id, mixnet_client_handle.as_mut().unwrap())
            .await
    }

    async fn send_connect_request(
        &self,
        message: ClientMessage,
        authenticator_address: Recipient,
    ) -> Result<u64> {
        let (request, request_id) = match message {
            ClientMessage::Initial(init_message) => {
                AuthenticatorRequest::new_initial_request(init_message, self.nym_address)
            }
            ClientMessage::Final(gateway_client) => {
                AuthenticatorRequest::new_final_request(gateway_client, self.nym_address)
            }
            ClientMessage::Query(peer_public_key) => {
                AuthenticatorRequest::new_query_request(peer_public_key, self.nym_address)
            }
        };
        debug!("Sent connect request with version v{}", request.version);

        self.mixnet_sender
            .send(nym_sdk::mixnet::InputMessage::new_regular(
                authenticator_address,
                request.to_bytes().unwrap(),
                TransmissionLane::General,
                None,
            ))
            .await?;

        Ok(request_id)
    }

    async fn listen_for_connect_response(
        &self,
        request_id: u64,
        mixnet_client: &mut MixnetClient,
    ) -> Result<AuthenticatorResponse> {
        let timeout = tokio::time::sleep(Duration::from_secs(10));
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
                            if !check_if_authenticator_message(&msg) {
                                debug!("Received non-authenticator message while waiting for connect response");
                                continue;
                            }
                            // Confirm that the version is correct
                            check_auth_message_version(&msg)?;

                            // Then we deserialize the message
                            debug!("AuthClient: got message while waiting for connect response");
                            let Ok(response) = AuthenticatorResponse::from_reconstructed_message(&msg) else {
                                // This is ok, it's likely just one of our self-pings
                                debug!("Failed to deserialize reconstructed message");
                                continue;
                            };

                            if response.id() == Some(request_id) {
                                debug!("Got response with matching id");
                                return Ok(response);
                            }
                        }
                    }
                }
            }
        }
    }
}

fn check_if_authenticator_message(message: &ReconstructedMessage) -> bool {
    // TODO: switch version number so that they have their own reserved range, like 50-100 for the
    // authenticator messages
    if let Some(version) = message.message.first() {
        // Temporary constant, see above TODO note
        *version < 6
    } else {
        false
    }
}

fn check_auth_message_version(message: &ReconstructedMessage) -> Result<()> {
    // Assuing it's an Authenticator message, it will have a version as its first byte
    if let Some(version) = message.message.first() {
        match version.cmp(&nym_authenticator_requests::CURRENT_VERSION) {
            Ordering::Greater => Err(Error::ReceivedResponseWithNewVersion {
                expected: nym_authenticator_requests::CURRENT_VERSION,
                received: *version,
            }),
            Ordering::Less => Err(Error::ReceivedResponseWithOldVersion {
                expected: nym_authenticator_requests::CURRENT_VERSION,
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
