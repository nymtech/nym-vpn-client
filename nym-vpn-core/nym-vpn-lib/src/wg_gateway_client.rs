// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::error::Result;
use crate::mixnet_connect::SharedMixnetClient;
use nym_authenticator_client::AuthClient;
use nym_authenticator_requests::v1::response::{AuthenticatorResponse, AuthenticatorResponseData};
use nym_crypto::asymmetric::encryption;
use nym_crypto::asymmetric::x25519::KeyPair;
use nym_gateway_directory::Recipient;
use nym_node_requests::api::v1::gateway::client_interfaces::wireguard::models::{
    ClientMessage, ClientRegistrationResponse, InitMessage, PeerPublicKey,
};
use nym_wireguard_types::registration::RegistrationData;
use nym_wireguard_types::GatewayClient;
use rand::{CryptoRng, RngCore};
use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;
use talpid_types::net::wireguard::PublicKey;
use tracing::debug;

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
    pub fn new<R: RngCore + CryptoRng>(rng: &mut R, mixnet_client: SharedMixnetClient) -> Self {
        let keypair = KeyPair::new(rng);

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
        let mixnet_client = self.mixnet_client.lock().await.unwrap();
        let response = auth_client.send(init_message, auth_recipient).await?;
        let AuthenticatorResponseData::PendingRegistration(RegistrationData {
            nonce,
            gateway_data,
            wg_port,
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
        let AuthenticatorResponseData::Registered =
            auth_client.send(finalized_message, auth_recipient).await?
        else {
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
