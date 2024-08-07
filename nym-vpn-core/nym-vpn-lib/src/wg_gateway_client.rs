// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::error::*;
use nym_authenticator_client::AuthClient;
use nym_authenticator_requests::v1::response::{
    AuthenticatorResponseData, PendingRegistrationResponse, RegisteredResponse,
    RemainingBandwidthResponse,
};
use nym_crypto::asymmetric::encryption;
use nym_crypto::asymmetric::x25519::KeyPair;
use nym_gateway_directory::Recipient;
use nym_node_requests::api::v1::gateway::client_interfaces::wireguard::models::{
    ClientMessage, InitMessage, PeerPublicKey,
};
use nym_pemstore::KeyPairPath;
use nym_sdk::TaskClient;
use nym_wireguard_types::registration::RegistrationData;
use nym_wireguard_types::{GatewayClient, DEFAULT_PEER_TIMEOUT_CHECK};
use rand::rngs::OsRng;
use rand::{CryptoRng, RngCore};
use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;
use talpid_types::net::wireguard::PublicKey;
use tokio_stream::wrappers::IntervalStream;
use tokio_stream::StreamExt;
use tracing::*;

const DEFAULT_PRIVATE_ENTRY_WIREGUARD_KEY_FILENAME: &str = "private_entry_wireguard.pem";
const DEFAULT_PUBLIC_ENTRY_WIREGUARD_KEY_FILENAME: &str = "public_entry_wireguard.pem";
const DEFAULT_PRIVATE_EXIT_WIREGUARD_KEY_FILENAME: &str = "private_exit_wireguard.pem";
const DEFAULT_PUBLIC_EXIT_WIREGUARD_KEY_FILENAME: &str = "public_exit_wireguard.pem";
const DEFAULT_BANDWIDTH_CHECK: Duration = Duration::from_secs(10); // 10 seconds
const ASSUMED_BANDWIDTH_DEPLETION_RATE: u64 = 10 * 1024 * 1024; // 10 MB/s

#[derive(Clone, Debug)]
pub struct GatewayData {
    pub(crate) public_key: PublicKey,
    pub(crate) endpoint: SocketAddr,
    pub(crate) private_ip: IpAddr,
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

    pub async fn register_wireguard(&mut self, gateway_host: IpAddr) -> Result<GatewayData> {
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
                gateway_data.verify(self.keypair.private_key(), nonce)?;

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

        let gateway_data = GatewayData {
            public_key: PublicKey::from(registred_data.pub_key.to_bytes()),
            endpoint: SocketAddr::from_str(&format!(
                "{}:{}",
                gateway_host, registred_data.wg_port
            ))?,
            private_ip: registred_data.private_ip,
        };

        Ok(gateway_data)
    }

    async fn query_bandwidth(&mut self) -> Result<Option<u64>> {
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

        if remaining_bandwidth_data.suspended {
            warn!("Wireguard access to gateway {} is suspended until tomorrow, UTC time. The client will shutdown", self.auth_recipient.gateway());
            Ok(None)
        } else {
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
    }

    pub async fn suspended(&mut self) -> Result<bool> {
        Ok(self.query_bandwidth().await?.is_none())
    }

    pub async fn run(mut self, mut shutdown: TaskClient) {
        let mut timeout_check_interval =
            IntervalStream::new(tokio::time::interval(DEFAULT_BANDWIDTH_CHECK));
        // Skip the first, immediate tick
        timeout_check_interval.next().await;
        while !shutdown.is_shutdown() {
            tokio::select! {
                _ = shutdown.recv_with_delay() => {
                    trace!("WgGatewayClient: Received shutdown");
                }
                _ = timeout_check_interval.next() => {
                    match self.query_bandwidth().await {
                        Err(e) => warn!("Error querying remaining bandwidth {:?}", e),
                        Ok(Some(remaining_bandwidth)) => {
                            match update_dynamic_check_interval(remaining_bandwidth) {
                                Some(new_interval) => {
                                    timeout_check_interval = new_interval;
                                    // skip the next beat, which takes place immediately
                                    timeout_check_interval.next().await;
                                }
                                None => {
                                    shutdown.send_we_stopped(Box::new(Error::OutOfBandwidth));
                                }
                            }
                        },
                        Ok(None) => {},
                    }
                }
            }
        }
    }
}

fn update_dynamic_check_interval(remaining_bandwidth: u64) -> Option<IntervalStream> {
    let estimated_depletion_secs = remaining_bandwidth / ASSUMED_BANDWIDTH_DEPLETION_RATE;
    // try and have 10 logs before depletion...
    let next_timeout_secs = estimated_depletion_secs / 10;
    if next_timeout_secs == 0 {
        return None;
    }
    // ... but not faster then the gateway bandwidth refresh
    if next_timeout_secs > DEFAULT_PEER_TIMEOUT_CHECK.as_secs() {
        Some(IntervalStream::new(tokio::time::interval(
            Duration::from_secs(next_timeout_secs),
        )))
    } else {
        Some(IntervalStream::new(tokio::time::interval(
            DEFAULT_PEER_TIMEOUT_CHECK,
        )))
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
