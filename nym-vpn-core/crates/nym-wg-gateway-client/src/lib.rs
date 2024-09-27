// Copyright 2023-2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

mod error;

use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::PathBuf,
    str::FromStr,
};

pub use error::{Error, ErrorMessage};
use nym_authenticator_client::AuthClient;
use nym_authenticator_client::ClientMessage;
use nym_authenticator_requests::v1::{
    registration::{GatewayClient, InitMessage, RegistrationData},
    response::{
        AuthenticatorResponseData, PendingRegistrationResponse, RegisteredResponse,
        RemainingBandwidthResponse,
    },
};
use nym_credentials_interface::CredentialSpendingData;
use nym_crypto::asymmetric::{encryption, x25519::KeyPair};
use nym_gateway_directory::Recipient;
use nym_node_requests::api::v1::gateway::client_interfaces::wireguard::models::PeerPublicKey;
use nym_pemstore::KeyPairPath;
use rand::{rngs::OsRng, CryptoRng, RngCore};
use talpid_types::net::wireguard::PublicKey; // TODO: this is a type we should provide instead
use tracing::{debug, error, info, warn};

use crate::error::Result;

const DEFAULT_PRIVATE_ENTRY_WIREGUARD_KEY_FILENAME: &str = "private_entry_wireguard.pem";
const DEFAULT_PUBLIC_ENTRY_WIREGUARD_KEY_FILENAME: &str = "public_entry_wireguard.pem";
const DEFAULT_PRIVATE_EXIT_WIREGUARD_KEY_FILENAME: &str = "private_exit_wireguard.pem";
const DEFAULT_PUBLIC_EXIT_WIREGUARD_KEY_FILENAME: &str = "public_exit_wireguard.pem";

#[derive(Clone, Debug)]
pub struct GatewayData {
    pub public_key: PublicKey,
    pub endpoint: SocketAddr,
    pub private_ipv4: Ipv4Addr,
}

pub struct WgGatewayClient {
    keypair: encryption::KeyPair,
    auth_client: AuthClient,
    auth_recipient: Recipient,
}

impl WgGatewayClient {
    fn new_type(
        data_path: &Option<PathBuf>,
        auth_client: AuthClient,
        auth_recipient: Recipient,
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
            }
        } else {
            WgGatewayClient {
                keypair: KeyPair::new(&mut rng),
                auth_client,
                auth_recipient,
            }
        }
    }

    pub fn new_entry(
        data_path: &Option<PathBuf>,
        auth_client: AuthClient,
        auth_recipient: Recipient,
    ) -> Self {
        Self::new_type(
            data_path,
            auth_client,
            auth_recipient,
            DEFAULT_PRIVATE_ENTRY_WIREGUARD_KEY_FILENAME,
            DEFAULT_PUBLIC_ENTRY_WIREGUARD_KEY_FILENAME,
        )
    }

    pub fn new_exit(
        data_path: &Option<PathBuf>,
        auth_client: AuthClient,
        auth_recipient: Recipient,
    ) -> Self {
        Self::new_type(
            data_path,
            auth_client,
            auth_recipient,
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

    pub async fn register_wireguard(
        &mut self,
        gateway_host: IpAddr,
        _credential: Option<CredentialSpendingData>,
    ) -> Result<GatewayData> {
        debug!("Registering with the wg gateway...");
        let init_message = ClientMessage::Initial(InitMessage {
            pub_key: PeerPublicKey::new(self.keypair.public_key().to_bytes().into()),
        });
        let response = self
            .auth_client
            .send(init_message, self.auth_recipient)
            .await?;
        let registred_data = match response.data {
            AuthenticatorResponseData::PendingRegistration(PendingRegistrationResponse {
                reply:
                    RegistrationData {
                        nonce,
                        gateway_data,
                        ..
                    },
                ..
            }) => {
                // Unwrap since we have already checked that we have the keypair.
                debug!("Verifying data");
                gateway_data
                    .verify(self.keypair.private_key(), nonce)
                    .map_err(Error::VerificationFailed)?;

                let finalized_message = ClientMessage::Final(GatewayClient::new(
                    self.keypair.private_key(),
                    gateway_data.pub_key().inner(),
                    gateway_data.private_ip,
                    nonce,
                ));
                let response = self
                    .auth_client
                    .send(finalized_message, self.auth_recipient)
                    .await?;
                let AuthenticatorResponseData::Registered(RegisteredResponse { reply, .. }) =
                    response.data
                else {
                    return Err(Error::InvalidGatewayAuthResponse);
                };
                reply
            }
            AuthenticatorResponseData::Registered(RegisteredResponse { reply, .. }) => reply,
            _ => return Err(Error::InvalidGatewayAuthResponse),
        };

        let IpAddr::V4(private_ipv4) = registred_data.private_ip else {
            return Err(Error::InvalidGatewayAuthResponse);
        };
        let gateway_data = GatewayData {
            public_key: PublicKey::from(registred_data.pub_key.to_bytes()),
            endpoint: SocketAddr::from_str(&format!("{}:{}", gateway_host, registred_data.wg_port))
                .map_err(Error::FailedToParseEntryGatewaySocketAddr)?,
            private_ipv4,
        };

        Ok(gateway_data)
    }

    pub async fn query_bandwidth(&mut self) -> Result<Option<u64>> {
        let query_message = ClientMessage::Query(PeerPublicKey::new(
            self.keypair.public_key().to_bytes().into(),
        ));
        let response = self
            .auth_client
            .send(query_message, self.auth_recipient)
            .await?;

        let remaining_bandwidth_data = match response.data {
            AuthenticatorResponseData::RemainingBandwidth(RemainingBandwidthResponse {
                reply: Some(remaining_bandwidth_data),
                ..
            }) => remaining_bandwidth_data,
            AuthenticatorResponseData::RemainingBandwidth(RemainingBandwidthResponse {
                reply: None,
                ..
            }) => return Ok(Some(0)),
            _ => return Err(Error::InvalidGatewayAuthResponse),
        };

        let remaining_pretty = if remaining_bandwidth_data.available_bandwidth > 1024 * 1024 {
            format!(
                "{:.2} MB",
                remaining_bandwidth_data.available_bandwidth as f64 / 1024.0 / 1024.0
            )
        } else {
            format!("{} KB", remaining_bandwidth_data.available_bandwidth / 1024)
        };
        info!(
            "Remaining wireguard bandwidth with gateway {} for today: {}",
            self.auth_recipient.gateway(),
            remaining_pretty
        );
        if remaining_bandwidth_data.available_bandwidth < 1024 * 1024 {
            warn!("Remaining bandwidth is under 1 MB. The wireguard mode will get suspended after that until tomorrow, UTC time. The client might shutdown with timeout soon");
        }
        Ok(Some(remaining_bandwidth_data.available_bandwidth))
    }

    pub async fn suspended(&mut self) -> Result<bool> {
        Ok(self.query_bandwidth().await?.is_none())
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
