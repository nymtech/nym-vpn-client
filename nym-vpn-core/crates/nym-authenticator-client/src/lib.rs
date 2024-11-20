use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    sync::Arc,
    time::Duration,
};

use nym_authenticator_requests::{v2, v3};

use nym_credentials_interface::CredentialSpendingData;
use nym_crypto::asymmetric::x25519::PrivateKey;
use nym_sdk::mixnet::{
    MixnetClient, MixnetClientSender, MixnetMessageSender, Recipient, ReconstructedMessage,
    TransmissionLane,
};
use nym_service_provider_requests_common::ServiceProviderType;
use nym_wireguard_types::PeerPublicKey;
use tracing::{debug, error};

mod error;

pub use crate::error::{Error, Result};

pub trait Versionable {
    fn version(&self) -> AuthenticatorVersion;
}

impl Versionable for v2::registration::InitMessage {
    fn version(&self) -> AuthenticatorVersion {
        AuthenticatorVersion::V2
    }
}

impl Versionable for v2::registration::FinalMessage {
    fn version(&self) -> AuthenticatorVersion {
        AuthenticatorVersion::V2
    }
}

impl Versionable for v3::registration::InitMessage {
    fn version(&self) -> AuthenticatorVersion {
        AuthenticatorVersion::V3
    }
}

impl Versionable for v3::registration::FinalMessage {
    fn version(&self) -> AuthenticatorVersion {
        AuthenticatorVersion::V3
    }
}

impl Versionable for PeerPublicKey {
    fn version(&self) -> AuthenticatorVersion {
        AuthenticatorVersion::V3
    }
}

impl Versionable for v3::topup::TopUpMessage {
    fn version(&self) -> AuthenticatorVersion {
        AuthenticatorVersion::V3
    }
}

pub trait InitMessage: Versionable {
    fn pub_key(&self) -> PeerPublicKey;
}

impl InitMessage for v2::registration::InitMessage {
    fn pub_key(&self) -> PeerPublicKey {
        self.pub_key
    }
}

impl InitMessage for v3::registration::InitMessage {
    fn pub_key(&self) -> PeerPublicKey {
        self.pub_key
    }
}

pub trait FinalMessage: Versionable {
    fn gateway_client_pub_key(&self) -> PeerPublicKey;
    fn gateway_client_ipv4(&self) -> Option<Ipv4Addr>;
    fn gateway_client_ipv6(&self) -> Option<Ipv6Addr>;
    fn gateway_client_mac(&self) -> Vec<u8>;
    fn credential(&self) -> Option<CredentialSpendingData>;
}

impl FinalMessage for v2::registration::FinalMessage {
    fn gateway_client_pub_key(&self) -> PeerPublicKey {
        self.gateway_client.pub_key
    }

    fn gateway_client_ipv4(&self) -> Option<Ipv4Addr> {
        match self.gateway_client.private_ip {
            std::net::IpAddr::V4(ipv4_addr) => Some(ipv4_addr),
            std::net::IpAddr::V6(_) => None,
        }
    }

    fn gateway_client_ipv6(&self) -> Option<Ipv6Addr> {
        None
    }

    fn gateway_client_mac(&self) -> Vec<u8> {
        self.gateway_client.mac.to_vec()
    }

    fn credential(&self) -> Option<CredentialSpendingData> {
        self.credential.clone()
    }
}

impl FinalMessage for v3::registration::FinalMessage {
    fn gateway_client_pub_key(&self) -> PeerPublicKey {
        self.gateway_client.pub_key
    }

    fn gateway_client_ipv4(&self) -> Option<Ipv4Addr> {
        match self.gateway_client.private_ip {
            std::net::IpAddr::V4(ipv4_addr) => Some(ipv4_addr),
            std::net::IpAddr::V6(_) => None,
        }
    }

    fn gateway_client_ipv6(&self) -> Option<Ipv6Addr> {
        None
    }

    fn gateway_client_mac(&self) -> Vec<u8> {
        self.gateway_client.mac.to_vec()
    }

    fn credential(&self) -> Option<CredentialSpendingData> {
        self.credential.clone()
    }
}

// Temporary solution for lacking a query message wrapper in monorepo
pub struct QueryMessageImpl {
    pub pub_key: PeerPublicKey,
    pub version: AuthenticatorVersion,
}

impl Versionable for QueryMessageImpl {
    fn version(&self) -> AuthenticatorVersion {
        self.version
    }
}

pub trait QueryMessage: Versionable {
    fn pub_key(&self) -> PeerPublicKey;
}

impl QueryMessage for QueryMessageImpl {
    fn pub_key(&self) -> PeerPublicKey {
        self.pub_key
    }
}

pub trait TopUpMessage: Versionable {
    fn pub_key(&self) -> PeerPublicKey;
    fn credential(&self) -> CredentialSpendingData;
}

impl TopUpMessage for v3::topup::TopUpMessage {
    fn pub_key(&self) -> PeerPublicKey {
        self.pub_key
    }

    fn credential(&self) -> CredentialSpendingData {
        self.credential.clone()
    }
}

pub enum ClientMessage {
    Initial(Box<dyn InitMessage + Send + Sync + 'static>),
    Final(Box<dyn FinalMessage + Send + Sync + 'static>),
    Query(Box<dyn QueryMessage + Send + Sync + 'static>),
    TopUp(Box<dyn TopUpMessage + Send + Sync + 'static>),
}

impl ClientMessage {
    fn version(&self) -> AuthenticatorVersion {
        match self {
            ClientMessage::Initial(msg) => msg.version(),
            ClientMessage::Final(msg) => msg.version(),
            ClientMessage::Query(msg) => msg.version(),
            ClientMessage::TopUp(msg) => msg.version(),
        }
    }

    pub fn bytes(&self, reply_to: Recipient) -> Result<(Vec<u8>, u64)> {
        match self.version() {
            AuthenticatorVersion::V2 => {
                use v2::{
                    registration::{ClientMac, FinalMessage, GatewayClient, InitMessage},
                    request::AuthenticatorRequest,
                };
                match self {
                    ClientMessage::Initial(init_message) => {
                        let (req, id) = AuthenticatorRequest::new_initial_request(
                            InitMessage {
                                pub_key: init_message.pub_key(),
                            },
                            reply_to,
                        );
                        Ok((req.to_bytes()?, id))
                    }
                    ClientMessage::Final(final_message) => {
                        let (req, id) = AuthenticatorRequest::new_final_request(
                            FinalMessage {
                                gateway_client: GatewayClient {
                                    pub_key: final_message.gateway_client_pub_key(),
                                    private_ip: final_message
                                        .gateway_client_ipv4()
                                        .ok_or(Error::UnsupportedMessage)?
                                        .into(),
                                    mac: ClientMac::new(final_message.gateway_client_mac()),
                                },
                                credential: final_message.credential(),
                            },
                            reply_to,
                        );
                        Ok((req.to_bytes()?, id))
                    }
                    ClientMessage::Query(query_message) => {
                        let (req, id) = AuthenticatorRequest::new_query_request(
                            query_message.pub_key(),
                            reply_to,
                        );
                        Ok((req.to_bytes()?, id))
                    }
                    _ => Err(Error::UnsupportedMessage),
                }
            }
            AuthenticatorVersion::V3 | AuthenticatorVersion::UNKNOWN => {
                use v3::{
                    registration::{ClientMac, FinalMessage, GatewayClient, InitMessage},
                    request::AuthenticatorRequest,
                    topup::TopUpMessage,
                };
                match self {
                    ClientMessage::Initial(init_message) => {
                        let (req, id) = AuthenticatorRequest::new_initial_request(
                            InitMessage {
                                pub_key: init_message.pub_key(),
                            },
                            reply_to,
                        );
                        Ok((req.to_bytes()?, id))
                    }
                    ClientMessage::Final(final_message) => {
                        let (req, id) = AuthenticatorRequest::new_final_request(
                            FinalMessage {
                                gateway_client: GatewayClient {
                                    pub_key: final_message.gateway_client_pub_key(),
                                    private_ip: final_message
                                        .gateway_client_ipv4()
                                        .ok_or(Error::UnsupportedMessage)?
                                        .into(),
                                    mac: ClientMac::new(final_message.gateway_client_mac()),
                                },
                                credential: final_message.credential(),
                            },
                            reply_to,
                        );
                        Ok((req.to_bytes()?, id))
                    }
                    ClientMessage::Query(query_message) => {
                        let (req, id) = AuthenticatorRequest::new_query_request(
                            query_message.pub_key(),
                            reply_to,
                        );
                        Ok((req.to_bytes()?, id))
                    }
                    ClientMessage::TopUp(top_up_message) => {
                        let (req, id) = AuthenticatorRequest::new_topup_request(
                            TopUpMessage {
                                pub_key: top_up_message.pub_key(),
                                credential: top_up_message.credential(),
                            },
                            reply_to,
                        );
                        Ok((req.to_bytes()?, id))
                    }
                }
            }
        }
    }
}

pub trait Id {
    fn id(&self) -> u64;
}

impl Id for v2::response::PendingRegistrationResponse {
    fn id(&self) -> u64 {
        self.request_id
    }
}

impl Id for v3::response::PendingRegistrationResponse {
    fn id(&self) -> u64 {
        self.request_id
    }
}

impl Id for v2::response::RegisteredResponse {
    fn id(&self) -> u64 {
        self.request_id
    }
}

impl Id for v3::response::RegisteredResponse {
    fn id(&self) -> u64 {
        self.request_id
    }
}

impl Id for v2::response::RemainingBandwidthResponse {
    fn id(&self) -> u64 {
        self.request_id
    }
}

impl Id for v3::response::RemainingBandwidthResponse {
    fn id(&self) -> u64 {
        self.request_id
    }
}

impl Id for v3::response::TopUpBandwidthResponse {
    fn id(&self) -> u64 {
        self.request_id
    }
}

pub trait PendingRegistrationResponse: Id {
    fn nonce(&self) -> u64;
    fn verify(
        &self,
        gateway_key: &PrivateKey,
    ) -> std::result::Result<(), nym_authenticator_requests::Error>;
    fn pub_key(&self) -> PeerPublicKey;
    fn private_ip(&self) -> IpAddr;
}

impl PendingRegistrationResponse for v2::response::PendingRegistrationResponse {
    fn nonce(&self) -> u64 {
        self.reply.nonce
    }

    fn verify(
        &self,
        gateway_key: &PrivateKey,
    ) -> std::result::Result<(), nym_authenticator_requests::Error> {
        self.reply.gateway_data.verify(gateway_key, self.nonce())
    }

    fn pub_key(&self) -> PeerPublicKey {
        self.reply.gateway_data.pub_key
    }

    fn private_ip(&self) -> IpAddr {
        self.reply.gateway_data.private_ip
    }
}

impl PendingRegistrationResponse for v3::response::PendingRegistrationResponse {
    fn nonce(&self) -> u64 {
        self.reply.nonce
    }

    fn verify(
        &self,
        gateway_key: &PrivateKey,
    ) -> std::result::Result<(), nym_authenticator_requests::Error> {
        self.reply.gateway_data.verify(gateway_key, self.nonce())
    }

    fn pub_key(&self) -> PeerPublicKey {
        self.reply.gateway_data.pub_key
    }

    fn private_ip(&self) -> IpAddr {
        self.reply.gateway_data.private_ip
    }
}

pub trait RegisteredResponse: Id {
    fn private_ip(&self) -> IpAddr;
    fn pub_key(&self) -> PeerPublicKey;
    fn wg_port(&self) -> u16;
}

impl RegisteredResponse for v2::response::RegisteredResponse {
    fn private_ip(&self) -> IpAddr {
        self.reply.private_ip
    }

    fn pub_key(&self) -> PeerPublicKey {
        self.reply.pub_key
    }

    fn wg_port(&self) -> u16 {
        self.reply.wg_port
    }
}

impl RegisteredResponse for v3::response::RegisteredResponse {
    fn private_ip(&self) -> IpAddr {
        self.reply.private_ip
    }

    fn pub_key(&self) -> PeerPublicKey {
        self.reply.pub_key
    }

    fn wg_port(&self) -> u16 {
        self.reply.wg_port
    }
}

pub trait RemainingBandwidthResponse: Id {
    fn available_bandwidth(&self) -> Option<i64>;
}

impl RemainingBandwidthResponse for v2::response::RemainingBandwidthResponse {
    fn available_bandwidth(&self) -> Option<i64> {
        self.reply.as_ref().map(|r| r.available_bandwidth)
    }
}

impl RemainingBandwidthResponse for v3::response::RemainingBandwidthResponse {
    fn available_bandwidth(&self) -> Option<i64> {
        self.reply.as_ref().map(|r| r.available_bandwidth)
    }
}

pub trait TopUpBandwidthResponse: Id {
    fn available_bandwidth(&self) -> i64;
}

impl TopUpBandwidthResponse for v3::response::TopUpBandwidthResponse {
    fn available_bandwidth(&self) -> i64 {
        self.reply.available_bandwidth
    }
}

pub enum AuthenticatorResponse {
    PendingRegistration(Box<dyn PendingRegistrationResponse + Send + Sync + 'static>),
    Registered(Box<dyn RegisteredResponse + Send + Sync + 'static>),
    RemainingBandwidth(Box<dyn RemainingBandwidthResponse + Send + Sync + 'static>),
    TopUpBandwidth(Box<dyn TopUpBandwidthResponse + Send + Sync + 'static>),
}

impl Id for AuthenticatorResponse {
    fn id(&self) -> u64 {
        match self {
            AuthenticatorResponse::PendingRegistration(pending_registration_response) => {
                pending_registration_response.id()
            }
            AuthenticatorResponse::Registered(registered_response) => registered_response.id(),
            AuthenticatorResponse::RemainingBandwidth(remaining_bandwidth_response) => {
                remaining_bandwidth_response.id()
            }
            AuthenticatorResponse::TopUpBandwidth(top_up_bandwidth_response) => {
                top_up_bandwidth_response.id()
            }
        }
    }
}

impl From<v2::response::AuthenticatorResponse> for AuthenticatorResponse {
    fn from(value: v2::response::AuthenticatorResponse) -> Self {
        match value.data {
            v2::response::AuthenticatorResponseData::PendingRegistration(
                pending_registration_response,
            ) => Self::PendingRegistration(Box::new(pending_registration_response)),
            v2::response::AuthenticatorResponseData::Registered(registered_response) => {
                Self::Registered(Box::new(registered_response))
            }
            v2::response::AuthenticatorResponseData::RemainingBandwidth(
                remaining_bandwidth_response,
            ) => Self::RemainingBandwidth(Box::new(remaining_bandwidth_response)),
        }
    }
}

impl From<v3::response::AuthenticatorResponse> for AuthenticatorResponse {
    fn from(value: v3::response::AuthenticatorResponse) -> Self {
        match value.data {
            v3::response::AuthenticatorResponseData::PendingRegistration(
                pending_registration_response,
            ) => Self::PendingRegistration(Box::new(pending_registration_response)),
            v3::response::AuthenticatorResponseData::Registered(registered_response) => {
                Self::Registered(Box::new(registered_response))
            }
            v3::response::AuthenticatorResponseData::RemainingBandwidth(
                remaining_bandwidth_response,
            ) => Self::RemainingBandwidth(Box::new(remaining_bandwidth_response)),
            v3::response::AuthenticatorResponseData::TopUpBandwidth(top_up_bandwidth_response) => {
                Self::TopUpBandwidth(Box::new(top_up_bandwidth_response))
            }
        }
    }
}

#[derive(Clone)]
pub struct SharedMixnetClient(Arc<tokio::sync::Mutex<Option<MixnetClient>>>);

impl SharedMixnetClient {
    pub fn from_shared(mixnet_client: &Arc<tokio::sync::Mutex<Option<MixnetClient>>>) -> Self {
        Self(Arc::clone(mixnet_client))
    }

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
        self.lock()
            .await
            .as_mut()
            .unwrap()
            .send(msg)
            .await
            .map_err(Error::SendMixnetMessage)?;
        Ok(())
    }

    pub fn inner(&self) -> Arc<tokio::sync::Mutex<Option<MixnetClient>>> {
        self.0.clone()
    }
}

#[derive(Copy, Clone, Debug)]
pub enum AuthenticatorVersion {
    V2,
    V3,
    UNKNOWN,
}

impl From<u8> for AuthenticatorVersion {
    fn from(value: u8) -> Self {
        if value == 2 {
            Self::V2
        } else if value == 3 {
            Self::V3
        } else {
            Self::UNKNOWN
        }
    }
}

impl From<String> for AuthenticatorVersion {
    fn from(value: String) -> Self {
        if value.contains("1.1.9") {
            Self::V2
        } else if value.contains("1.1.10") {
            Self::V3
        } else {
            Self::UNKNOWN
        }
    }
}

impl From<Option<String>> for AuthenticatorVersion {
    fn from(value: Option<String>) -> Self {
        let Some(value) = value else {
            return Self::UNKNOWN;
        };
        if value.contains("1.1.9") {
            Self::V2
        } else if value.contains("1.1.10") {
            Self::V3
        } else {
            Self::UNKNOWN
        }
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
        message: &ClientMessage,
        authenticator_address: Recipient,
    ) -> Result<AuthenticatorResponse> {
        self.send_inner(message, authenticator_address).await
    }

    async fn send_inner(
        &mut self,
        message: &ClientMessage,
        authenticator_address: Recipient,
    ) -> Result<AuthenticatorResponse> {
        // Connecting is basically synchronous from the perspective of the mixnet client, so it's safe
        // to just grab ahold of the mutex and keep it until we get the response.
        // This needs to sit here, before sending the request and dropped after getting the response,
        // so that it doesn't interfere with message to the other gateway (entry/exit).
        let mut mixnet_client_handle = self.mixnet_client.lock().await;
        if mixnet_client_handle.is_none() {
            return Err(Error::UnableToGetMixnetHandle);
        }
        let request_id = self
            .send_connect_request(message, authenticator_address)
            .await?;

        debug!("Waiting for reply...");
        self.listen_for_connect_response(request_id, mixnet_client_handle.as_mut().unwrap())
            .await
    }

    async fn send_connect_request(
        &self,
        message: &ClientMessage,
        authenticator_address: Recipient,
    ) -> Result<u64> {
        let (data, request_id) = message.bytes(self.nym_address)?;

        self.mixnet_sender
            .send(nym_sdk::mixnet::InputMessage::new_regular(
                authenticator_address,
                data,
                TransmissionLane::General,
                None,
            ))
            .await
            .map_err(Error::SendMixnetMessage)?;

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
                            let version = check_auth_message_version(&msg)?;

                            // Then we deserialize the message
                            debug!("AuthClient: got message while waiting for connect response with version {version:?}");
                            let ret: Result<AuthenticatorResponse> = match version {
                                AuthenticatorVersion::V2 => v2::response::AuthenticatorResponse::from_reconstructed_message(&msg).map(Into::into).map_err(Into::into),
                                AuthenticatorVersion::V3 => v3::response::AuthenticatorResponse::from_reconstructed_message(&msg).map(Into::into).map_err(Into::into),
                                AuthenticatorVersion::UNKNOWN => Err(Error::UnknownVersion),
                            };
                            let Ok(response) = ret else {
                                // This is ok, it's likely just one of our self-pings
                                debug!("Failed to deserialize reconstructed message");
                                continue;
                            };

                            if response.id() == request_id {
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
    if let Some(msg_type) = message.message.get(1) {
        ServiceProviderType::Authenticator as u8 == *msg_type
    } else {
        false
    }
}

fn check_auth_message_version(message: &ReconstructedMessage) -> Result<AuthenticatorVersion> {
    // Assuing it's an Authenticator message, it will have a version as its first byte
    if let Some(&version) = message.message.first() {
        Ok(version.into())
    } else {
        Err(Error::NoVersionInMessage)
    }
}
