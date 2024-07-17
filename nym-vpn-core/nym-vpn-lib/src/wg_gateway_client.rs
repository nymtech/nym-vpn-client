// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::error::Result;
use crate::mixnet_connect::SharedMixnetClient;
use nym_authenticator_client::AuthClient;
use nym_authenticator_requests::v1::response::{
    AuthenticatorResponseData, PendingRegistrationResponse,
};
use nym_crypto::asymmetric::encryption;
use nym_crypto::asymmetric::x25519::KeyPair;
use nym_gateway_directory::Recipient;
use nym_node_requests::api::v1::gateway::client_interfaces::wireguard::models::{
    ClientMessage, InitMessage, PeerPublicKey,
};
use nym_pemstore::KeyPairPath;
use nym_wireguard_types::registration::RegistrationData;
use nym_wireguard_types::GatewayClient;
use rand::rngs::OsRng;
use rand::{CryptoRng, RngCore};
use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use std::str::FromStr;
use talpid_types::net::wireguard::PublicKey;
use tracing::debug;

const DEFAULT_PRIVATE_ENTRY_WIREGUARD_KEY_FILENAME: &str = "private_entry_wireguard.pem";
const DEFAULT_PUBLIC_ENTRY_WIREGUARD_KEY_FILENAME: &str = "public_entry_wireguard.pem";
const DEFAULT_PRIVATE_EXIT_WIREGUARD_KEY_FILENAME: &str = "private_exit_wireguard.pem";
const DEFAULT_PUBLIC_EXIT_WIREGUARD_KEY_FILENAME: &str = "public_exit_wireguard.pem";

#[derive(Clone, Debug)]
pub struct GatewayData {
    pub(crate) public_key: PublicKey,
    pub(crate) endpoint: SocketAddr,
    pub(crate) private_ip: IpAddr,
}

pub struct WgGatewayClient {
    keypair: encryption::KeyPair,
    mixnet_client: SharedMixnetClient,
}

impl WgGatewayClient {
    fn new_type(
        data_path: &Option<PathBuf>,
        mixnet_client: SharedMixnetClient,
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
                mixnet_client,
            }
        } else {
            WgGatewayClient {
                keypair: KeyPair::new(&mut rng),
                mixnet_client,
            }
        }
    }

    pub fn new_entry(data_path: &Option<PathBuf>, mixnet_client: SharedMixnetClient) -> Self {
        Self::new_type(
            data_path,
            mixnet_client,
            DEFAULT_PRIVATE_ENTRY_WIREGUARD_KEY_FILENAME,
            DEFAULT_PUBLIC_ENTRY_WIREGUARD_KEY_FILENAME,
        )
    }

    pub fn new_exit(data_path: &Option<PathBuf>, mixnet_client: SharedMixnetClient) -> Self {
        Self::new_type(
            data_path,
            mixnet_client,
            DEFAULT_PRIVATE_EXIT_WIREGUARD_KEY_FILENAME,
            DEFAULT_PUBLIC_EXIT_WIREGUARD_KEY_FILENAME,
        )
    }

    pub fn new(keypair: encryption::KeyPair, mixnet_client: SharedMixnetClient) -> Self {
        WgGatewayClient {
            keypair,
            mixnet_client,
        }
    }

    pub fn keypair(&self) -> &encryption::KeyPair {
        &self.keypair
    }

    pub async fn register_wireguard(
        &self,
        auth_recipient: Recipient,
        gateway_host: IpAddr,
    ) -> Result<GatewayData> {
        let mut auth_client = AuthClient::new_from_inner(self.mixnet_client.inner()).await;

        debug!("Registering with the wg gateway...");
        let init_message = ClientMessage::Initial(InitMessage {
            pub_key: PeerPublicKey::new(self.keypair.public_key().to_bytes().into()),
        });
        let response = auth_client.send(init_message, auth_recipient).await?;
        let AuthenticatorResponseData::PendingRegistration(PendingRegistrationResponse {
            reply:
                RegistrationData {
                    nonce,
                    gateway_data,
                    wg_port,
                },
            ..
        }) = response.data
        else {
            return Err(crate::error::Error::InvalidGatewayAPIResponse);
        };
        debug!("Received nonce: {}", nonce);
        debug!("Received wg_port: {}", wg_port);
        debug!("Received gateway data: {:?}", gateway_data);

        // Unwrap since we have already checked that we have the keypair.
        debug!("Verifying data");
        gateway_data.verify(self.keypair.private_key(), nonce)?;

        let finalized_message = ClientMessage::Final(GatewayClient::new(
            self.keypair.private_key(),
            gateway_data.pub_key().inner(),
            gateway_data.private_ip,
            nonce,
        ));
        let response = auth_client.send(finalized_message, auth_recipient).await?;
        let AuthenticatorResponseData::Registered(_) = response.data else {
            return Err(crate::error::Error::InvalidGatewayAPIResponse);
        };
        let gateway_data = GatewayData {
            public_key: PublicKey::from(gateway_data.pub_key().to_bytes()),
            endpoint: SocketAddr::from_str(&format!("{}:{}", gateway_host, wg_port))?,
            private_ip: gateway_data.private_ip,
        };

        Ok(gateway_data)
    }
}

fn load_or_generate_keypair<R: RngCore + CryptoRng>(rng: &mut R, paths: KeyPairPath) -> KeyPair {
    match nym_pemstore::load_keypair(&paths) {
        Ok(keypair) => keypair,
        Err(_) => {
            let keypair = KeyPair::new(rng);
            if let Err(e) = nym_pemstore::store_keypair(&keypair, &paths) {
                log::error!(
                    "could not store generated keypair at {:?} - {:?}; will use ephemeral keys",
                    paths,
                    e
                );
            }
            keypair
        }
    }
}
