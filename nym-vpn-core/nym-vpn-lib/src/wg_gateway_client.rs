// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::error::Result;
use nym_config::defaults::DEFAULT_NYM_NODE_HTTP_PORT;
use nym_crypto::asymmetric::encryption;
use nym_crypto::asymmetric::x25519::KeyPair;
use nym_node_requests::api::client::NymNodeApiClientExt;
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
}

impl WgGatewayClient {
    pub fn new<R: RngCore + CryptoRng>(rng: &mut R) -> Self {
        let keypair = KeyPair::new(rng);

        WgGatewayClient { keypair }
    }

    pub fn keypair(&self) -> &encryption::KeyPair {
        &self.keypair
    }

    pub async fn register_wireguard(
        &self,
        // gateway_identity: &str,
        gateway_host: IpAddr,
    ) -> Result<GatewayData> {
        // info!("Lookup ip for {}", gateway_identity);
        // let gateway_host = self.lookup_gateway_ip(gateway_identity).await?;
        // info!("Received wg gateway ip: {}", gateway_host);

        let gateway_api_client = nym_node_requests::api::Client::new_url(
            format!("{}:{}", gateway_host, DEFAULT_NYM_NODE_HTTP_PORT),
            None,
        )?;

        debug!("Registering with the wg gateway...");
        let init_message = ClientMessage::Initial(InitMessage {
            pub_key: PeerPublicKey::new(self.keypair.public_key().to_bytes().into()),
        });
        let ClientRegistrationResponse::PendingRegistration(RegistrationData {
            nonce,
            gateway_data,
            wg_port,
        }) = gateway_api_client
            .post_gateway_register_client(&init_message)
            .await?
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
        let ClientRegistrationResponse::Registered = gateway_api_client
            .post_gateway_register_client(&finalized_message)
            .await?
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
