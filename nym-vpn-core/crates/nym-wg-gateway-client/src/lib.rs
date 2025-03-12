// Copyright 2023-2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

mod error;

use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    path::PathBuf,
    str::FromStr,
    time::Duration,
};

pub use error::{Error, ErrorMessage};
use nym_authenticator_client::{
    AuthClient, AuthenticatorResponse, AuthenticatorVersion, ClientMessage, QueryMessageImpl,
};
use nym_authenticator_requests::{v2, v3, v4, v5};
use nym_bandwidth_controller::PreparedCredential;
use nym_credentials_interface::{CredentialSpendingData, TicketType};
use nym_crypto::asymmetric::{encryption, x25519::KeyPair};
use nym_gateway_directory::Recipient;
use nym_node_requests::api::v1::gateway::client_interfaces::wireguard::models::PeerPublicKey;
use nym_pemstore::KeyPairPath;
use nym_sdk::mixnet::{ClientStatsEvents, CredentialStorage};
use nym_validator_client::QueryHttpRpcNyxdClient;
use nym_wg_go::PublicKey;
use rand::{rngs::OsRng, CryptoRng, RngCore};
use tracing::{debug, error, info, warn};

use crate::error::Result;

pub const DEFAULT_PRIVATE_ENTRY_WIREGUARD_KEY_FILENAME: &str = "private_entry_wireguard.pem";
pub const DEFAULT_PUBLIC_ENTRY_WIREGUARD_KEY_FILENAME: &str = "public_entry_wireguard.pem";
pub const DEFAULT_PRIVATE_EXIT_WIREGUARD_KEY_FILENAME: &str = "private_exit_wireguard.pem";
pub const DEFAULT_PUBLIC_EXIT_WIREGUARD_KEY_FILENAME: &str = "public_exit_wireguard.pem";

pub const DEFAULT_FREE_PRIVATE_ENTRY_WIREGUARD_KEY_FILENAME: &str =
    "free_private_entry_wireguard.pem";
pub const DEFAULT_FREE_PUBLIC_ENTRY_WIREGUARD_KEY_FILENAME: &str =
    "free_public_entry_wireguard.pem";
pub const DEFAULT_FREE_PRIVATE_EXIT_WIREGUARD_KEY_FILENAME: &str =
    "free_private_exit_wireguard.pem";
pub const DEFAULT_FREE_PUBLIC_EXIT_WIREGUARD_KEY_FILENAME: &str = "free_public_exit_wireguard.pem";

pub const TICKETS_TO_SPEND: u32 = 1;
const RETRY_PERIOD: Duration = Duration::from_secs(30);

#[derive(Clone, Debug)]
pub struct GatewayData {
    pub public_key: PublicKey,
    pub endpoint: SocketAddr,
    pub private_ipv4: Ipv4Addr,
    pub private_ipv6: Ipv6Addr,
}
#[derive(Clone)]
pub struct WgGatewayLightClient {
    public_key: encryption::PublicKey,
    auth_client: AuthClient,
    auth_recipient: Recipient,
    auth_version: AuthenticatorVersion,
}

impl WgGatewayLightClient {
    pub fn auth_recipient(&self) -> Recipient {
        self.auth_recipient
    }

    pub fn auth_client(&self) -> &AuthClient {
        &self.auth_client
    }

    pub fn set_auth_client(&mut self, auth_client: AuthClient) {
        self.auth_client = auth_client;
    }

    pub async fn query_bandwidth(&mut self) -> Result<Option<i64>> {
        let query_message = match self.auth_version {
            AuthenticatorVersion::V2 => ClientMessage::Query(Box::new(QueryMessageImpl {
                pub_key: PeerPublicKey::new(self.public_key.to_bytes().into()),
                version: AuthenticatorVersion::V2,
            })),
            AuthenticatorVersion::V3 => ClientMessage::Query(Box::new(QueryMessageImpl {
                pub_key: PeerPublicKey::new(self.public_key.to_bytes().into()),
                version: AuthenticatorVersion::V3,
            })),
            AuthenticatorVersion::V4 => ClientMessage::Query(Box::new(QueryMessageImpl {
                pub_key: PeerPublicKey::new(self.public_key.to_bytes().into()),
                version: AuthenticatorVersion::V4,
            })),
            AuthenticatorVersion::V5 => ClientMessage::Query(Box::new(QueryMessageImpl {
                pub_key: PeerPublicKey::new(self.public_key.to_bytes().into()),
                version: AuthenticatorVersion::V5,
            })),
            AuthenticatorVersion::UNKNOWN => return Err(Error::UnsupportedAuthenticatorVersion),
        };
        let response = self
            .auth_client
            .send(&query_message, self.auth_recipient)
            .await?;

        let available_bandwidth = match response {
            nym_authenticator_client::AuthenticatorResponse::RemainingBandwidth(
                remaining_bandwidth_response,
            ) => remaining_bandwidth_response
                .available_bandwidth()
                .unwrap_or_default(),
            _ => return Err(Error::InvalidGatewayAuthResponse),
        };

        let remaining_pretty = if available_bandwidth > 1024 * 1024 {
            format!("{:.2} MB", available_bandwidth as f64 / 1024.0 / 1024.0)
        } else {
            format!("{} KB", available_bandwidth / 1024)
        };
        info!(
            "Remaining wireguard bandwidth with gateway {} for today: {}",
            self.auth_recipient.gateway(),
            remaining_pretty
        );
        if available_bandwidth < 1024 * 1024 {
            warn!("Remaining bandwidth is under 1 MB. The wireguard mode will get suspended after that until tomorrow, UTC time. The client might shutdown with timeout soon");
        }
        Ok(Some(available_bandwidth))
    }

    pub async fn suspended(&mut self) -> Result<bool> {
        Ok(self.query_bandwidth().await?.is_none())
    }

    async fn send(&mut self, msg: ClientMessage) -> Result<AuthenticatorResponse> {
        let now = std::time::Instant::now();
        while now.elapsed() < RETRY_PERIOD {
            match self.auth_client.send(&msg, self.auth_recipient).await {
                Ok(response) => return Ok(response),
                Err(nym_authenticator_client::Error::TimeoutWaitingForConnectResponse) => continue,
                Err(source) => {
                    if msg.is_wasteful() {
                        return Err(Error::NoRetry { source });
                    } else {
                        return Err(Error::AuthenticatorClientError(source));
                    }
                }
            }
        }
        if msg.is_wasteful() {
            Err(Error::NoRetry {
                source: nym_authenticator_client::Error::TimeoutWaitingForConnectResponse,
            })
        } else {
            Err(Error::AuthenticatorClientError(
                nym_authenticator_client::Error::TimeoutWaitingForConnectResponse,
            ))
        }
    }

    pub async fn top_up(&mut self, credential: CredentialSpendingData) -> Result<i64> {
        let top_up_message = match self.auth_version {
            AuthenticatorVersion::V3 => ClientMessage::TopUp(Box::new(v3::topup::TopUpMessage {
                pub_key: PeerPublicKey::new(self.public_key.to_bytes().into()),
                credential,
            })),
            // NOTE: looks like a bug here using v3. But we're leaving it as is since it's working
            // and V4 is deprecated in favour of V5
            AuthenticatorVersion::V4 => ClientMessage::TopUp(Box::new(v3::topup::TopUpMessage {
                pub_key: PeerPublicKey::new(self.public_key.to_bytes().into()),
                credential,
            })),
            AuthenticatorVersion::V5 => ClientMessage::TopUp(Box::new(v5::topup::TopUpMessage {
                pub_key: PeerPublicKey::new(self.public_key.to_bytes().into()),
                credential,
            })),
            AuthenticatorVersion::V2 | AuthenticatorVersion::UNKNOWN => {
                return Err(Error::UnsupportedAuthenticatorVersion)
            }
        };
        let response = self.send(top_up_message).await?;

        let remaining_bandwidth = match response {
            AuthenticatorResponse::TopUpBandwidth(top_up_bandwidth_response) => {
                top_up_bandwidth_response.available_bandwidth()
            }
            _ => return Err(Error::InvalidGatewayAuthResponse),
        };

        Ok(remaining_bandwidth)
    }

    pub fn send_stats_event(&self, event: ClientStatsEvents) {
        self.auth_client.send_stats_event(event);
    }
}

pub struct WgGatewayClient {
    keypair: encryption::KeyPair,
    auth_client: AuthClient,
    auth_recipient: Recipient,
    auth_version: AuthenticatorVersion,
}

impl WgGatewayClient {
    pub fn light_client(&self) -> WgGatewayLightClient {
        WgGatewayLightClient {
            public_key: *self.keypair.public_key(),
            auth_client: self.auth_client.clone(),
            auth_recipient: self.auth_recipient,
            auth_version: self.auth_version,
        }
    }

    fn new_type(
        data_path: &Option<PathBuf>,
        auth_client: AuthClient,
        auth_recipient: Recipient,
        auth_version: AuthenticatorVersion,
        private_file_name: &str,
        public_file_name: &str,
    ) -> Self {
        let mut rng = OsRng;
        if let Some(data_path) = data_path {
            let paths = KeyPairPath::new(
                data_path.join(private_file_name),
                data_path.join(public_file_name),
            );
            let keypair = load_or_generate_keypair(&mut rng, paths);
            WgGatewayClient {
                keypair,
                auth_client,
                auth_recipient,
                auth_version,
            }
        } else {
            WgGatewayClient {
                keypair: KeyPair::new(&mut rng),
                auth_client,
                auth_recipient,
                auth_version,
            }
        }
    }

    pub fn new_free_entry(
        data_path: &Option<PathBuf>,
        auth_client: AuthClient,
        auth_recipient: Recipient,
        auth_version: AuthenticatorVersion,
    ) -> Self {
        Self::new_type(
            data_path,
            auth_client,
            auth_recipient,
            auth_version,
            DEFAULT_FREE_PRIVATE_ENTRY_WIREGUARD_KEY_FILENAME,
            DEFAULT_FREE_PUBLIC_ENTRY_WIREGUARD_KEY_FILENAME,
        )
    }

    pub fn new_free_exit(
        data_path: &Option<PathBuf>,
        auth_client: AuthClient,
        auth_recipient: Recipient,
        auth_version: AuthenticatorVersion,
    ) -> Self {
        Self::new_type(
            data_path,
            auth_client,
            auth_recipient,
            auth_version,
            DEFAULT_FREE_PRIVATE_EXIT_WIREGUARD_KEY_FILENAME,
            DEFAULT_FREE_PUBLIC_EXIT_WIREGUARD_KEY_FILENAME,
        )
    }

    pub fn new_entry(
        data_path: &Option<PathBuf>,
        auth_client: AuthClient,
        auth_recipient: Recipient,
        auth_version: AuthenticatorVersion,
    ) -> Self {
        Self::new_type(
            data_path,
            auth_client,
            auth_recipient,
            auth_version,
            DEFAULT_PRIVATE_ENTRY_WIREGUARD_KEY_FILENAME,
            DEFAULT_PUBLIC_ENTRY_WIREGUARD_KEY_FILENAME,
        )
    }

    pub fn new_exit(
        data_path: &Option<PathBuf>,
        auth_client: AuthClient,
        auth_recipient: Recipient,
        auth_version: AuthenticatorVersion,
    ) -> Self {
        Self::new_type(
            data_path,
            auth_client,
            auth_recipient,
            auth_version,
            DEFAULT_PRIVATE_EXIT_WIREGUARD_KEY_FILENAME,
            DEFAULT_PUBLIC_EXIT_WIREGUARD_KEY_FILENAME,
        )
    }

    pub fn keypair(&self) -> &encryption::KeyPair {
        &self.keypair
    }

    pub fn auth_recipient(&self) -> Recipient {
        self.auth_recipient
    }

    pub fn auth_version(&self) -> AuthenticatorVersion {
        self.auth_version
    }

    pub async fn request_bandwidth<St: CredentialStorage>(
        wg_gateway_client: &mut WgGatewayLightClient,
        controller: &nym_bandwidth_controller::BandwidthController<QueryHttpRpcNyxdClient, St>,
        ticketbook_type: TicketType,
    ) -> Result<PreparedCredential>
    where
        <St as CredentialStorage>::StorageError: Send + Sync + 'static,
    {
        let credential = controller
            .prepare_ecash_ticket(
                ticketbook_type,
                wg_gateway_client.auth_recipient().gateway().to_bytes(),
                TICKETS_TO_SPEND,
            )
            .await
            .map_err(|source| Error::GetTicket {
                ticketbook_type,
                source,
            })?;
        Ok(credential)
    }

    pub async fn register_wireguard<St: CredentialStorage>(
        &mut self,
        gateway_host: IpAddr,
        controller: &nym_bandwidth_controller::BandwidthController<QueryHttpRpcNyxdClient, St>,
        enable_credentials_mode: bool,
        ticketbook_type: TicketType,
    ) -> Result<GatewayData>
    where
        <St as CredentialStorage>::StorageError: Send + Sync + 'static,
    {
        debug!("Registering with the wg gateway...");
        let init_message = match self.auth_version {
            AuthenticatorVersion::V2 => {
                ClientMessage::Initial(Box::new(v2::registration::InitMessage {
                    pub_key: PeerPublicKey::new(self.keypair.public_key().to_bytes().into()),
                }))
            }
            AuthenticatorVersion::V3 => {
                ClientMessage::Initial(Box::new(v3::registration::InitMessage {
                    pub_key: PeerPublicKey::new(self.keypair.public_key().to_bytes().into()),
                }))
            }
            AuthenticatorVersion::V4 => {
                ClientMessage::Initial(Box::new(v4::registration::InitMessage {
                    pub_key: PeerPublicKey::new(self.keypair.public_key().to_bytes().into()),
                }))
            }
            AuthenticatorVersion::V5 => {
                ClientMessage::Initial(Box::new(v5::registration::InitMessage {
                    pub_key: PeerPublicKey::new(self.keypair.public_key().to_bytes().into()),
                }))
            }
            AuthenticatorVersion::UNKNOWN => return Err(Error::UnsupportedAuthenticatorVersion),
        };
        let response = self
            .auth_client
            .send(&init_message, self.auth_recipient)
            .await?;
        let registered_data = match response {
            AuthenticatorResponse::PendingRegistration(pending_registration_response) => {
                // Unwrap since we have already checked that we have the keypair.
                debug!("Verifying data");
                if let Err(e) = pending_registration_response.verify(self.keypair.private_key()) {
                    return Err(Error::VerificationFailed(e));
                }

                let credential = if enable_credentials_mode {
                    let cred = Self::request_bandwidth(
                        &mut self.light_client(),
                        controller,
                        ticketbook_type,
                    )
                    .await?;
                    Some(cred.data)
                } else {
                    None
                };

                let finalized_message = match self.auth_version {
                    AuthenticatorVersion::V2 => {
                        ClientMessage::Final(Box::new(v2::registration::FinalMessage {
                            gateway_client: v2::registration::GatewayClient::new(
                                self.keypair.private_key(),
                                pending_registration_response.pub_key().inner(),
                                pending_registration_response.private_ips().ipv4.into(),
                                pending_registration_response.nonce(),
                            ),
                            credential,
                        }))
                    }
                    AuthenticatorVersion::V3 => {
                        ClientMessage::Final(Box::new(v3::registration::FinalMessage {
                            gateway_client: v3::registration::GatewayClient::new(
                                self.keypair.private_key(),
                                pending_registration_response.pub_key().inner(),
                                pending_registration_response.private_ips().ipv4.into(),
                                pending_registration_response.nonce(),
                            ),
                            credential,
                        }))
                    }
                    AuthenticatorVersion::V4 => {
                        ClientMessage::Final(Box::new(v4::registration::FinalMessage {
                            gateway_client: v4::registration::GatewayClient::new(
                                self.keypair.private_key(),
                                pending_registration_response.pub_key().inner(),
                                pending_registration_response.private_ips().into(),
                                pending_registration_response.nonce(),
                            ),
                            credential,
                        }))
                    }
                    AuthenticatorVersion::V5 => {
                        ClientMessage::Final(Box::new(v5::registration::FinalMessage {
                            gateway_client: v5::registration::GatewayClient::new(
                                self.keypair.private_key(),
                                pending_registration_response.pub_key().inner(),
                                pending_registration_response.private_ips(),
                                pending_registration_response.nonce(),
                            ),
                            credential,
                        }))
                    }
                    AuthenticatorVersion::UNKNOWN => {
                        return Err(Error::UnsupportedAuthenticatorVersion)
                    }
                };
                let response = self.light_client().send(finalized_message).await?;
                let AuthenticatorResponse::Registered(registered_response) = response else {
                    return Err(Error::InvalidGatewayAuthResponse);
                };
                registered_response
            }
            AuthenticatorResponse::Registered(registered_response) => registered_response,
            _ => return Err(Error::InvalidGatewayAuthResponse),
        };

        let gateway_data = GatewayData {
            public_key: PublicKey::from(registered_data.pub_key().to_bytes()),
            endpoint: SocketAddr::from_str(&format!(
                "{}:{}",
                gateway_host,
                registered_data.wg_port()
            ))
            .map_err(Error::FailedToParseEntryGatewaySocketAddr)?,
            private_ipv4: registered_data.private_ips().ipv4,
            private_ipv6: registered_data.private_ips().ipv6,
        };

        Ok(gateway_data)
    }

    pub async fn top_up_wireguard<St: CredentialStorage>(
        wg_gateway_client: &mut WgGatewayLightClient,
        controller: &nym_bandwidth_controller::BandwidthController<QueryHttpRpcNyxdClient, St>,
        ticketbook_type: TicketType,
    ) -> Result<i64>
    where
        <St as CredentialStorage>::StorageError: Send + Sync + 'static,
    {
        let credential =
            Self::request_bandwidth(wg_gateway_client, controller, ticketbook_type).await?;
        let remaining_bandwidth = wg_gateway_client.top_up(credential.data).await?;

        Ok(remaining_bandwidth)
    }
}

fn load_or_generate_keypair<R: RngCore + CryptoRng>(rng: &mut R, paths: KeyPairPath) -> KeyPair {
    match nym_pemstore::load_keypair(&paths) {
        Ok(keypair) => keypair,
        Err(_) => {
            let keypair = KeyPair::new(rng);
            if let Err(e) = nym_pemstore::store_keypair(&keypair, &paths) {
                error!(
                    "could not store generated keypair at {:?} - {:?}; will use ephemeral keys",
                    paths, e
                );
            }
            keypair
        }
    }
}
