use std::{net::IpAddr, sync::Arc};

use anyhow::bail;
use dns_lookup::lookup_host;
use nym_authenticator_client_v2::{
    AuthClient, ClientMessage, SharedMixnetClient as AuthSharedMixnetClient,
};
use nym_authenticator_requests_v2::v2::{
    registration::{FinalMessage, GatewayClient, InitMessage, RegistrationData},
    response::{AuthenticatorResponseData, PendingRegistrationResponse, RegisteredResponse},
};
use nym_crypto_v2::asymmetric::encryption::PrivateKey;
use nym_gateway_directory_v2::AuthAddress;
use nym_sdk_v2::mixnet::MixnetClient;
use nym_topology_v2::NetworkAddress;
use nym_wireguard_types_v2::PeerPublicKey;
use tokio::sync::Mutex;
use tracing::{debug, info};

use crate::netstack::{self, NetstackCall as _, NetstackCallImpl};

use super::types::WgProbeResults;

pub async fn probe(
    authenticator: AuthAddress,
    shared_mixnet_client: Arc<Mutex<Option<MixnetClient>>>,
    gateway_host: NetworkAddress,
) -> anyhow::Result<WgProbeResults> {
    let auth_shared_client = AuthSharedMixnetClient::from_shared(&shared_mixnet_client);
    let mut auth_client = AuthClient::new(auth_shared_client).await;

    let mut rng = rand::thread_rng();
    let private_key = PrivateKey::new(&mut rng);
    let public_key = private_key.public_key();

    let init_message = ClientMessage::Initial(InitMessage {
        pub_key: PeerPublicKey::new(public_key.to_bytes().into()),
    });

    let mut wg_outcome = WgProbeResults::default();

    if let Some(authenticator_address) = authenticator.0 {
        let response = auth_client
            .send(init_message, authenticator_address)
            .await?;

        let registered_data = match response.data {
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
                gateway_data.verify(&private_key, nonce)?;

                let finalized_message = ClientMessage::Final(Box::new(FinalMessage {
                    gateway_client: GatewayClient::new(
                        &private_key,
                        gateway_data.pub_key().inner(),
                        gateway_data.private_ip,
                        nonce,
                    ),
                    credential: None,
                }));
                // let finalized_message = ClientMessage::Final(Box::new(FinalMessage {
                //     gateway_client: GatewayClient::new(
                //         &private_key,
                //         gateway_data.pub_key().inner(),
                //         gateway_data.private_ip,
                //         nonce,
                //     ),
                //     credential: None,
                // }));
                let response = auth_client
                    .send(finalized_message, authenticator_address)
                    .await?;
                let AuthenticatorResponseData::Registered(RegisteredResponse { reply, .. }) =
                    response.data
                else {
                    bail!("Unexpected response: {response:?}");
                };
                reply
            }
            AuthenticatorResponseData::Registered(RegisteredResponse { reply, .. }) => reply,
            _ => bail!("Unexpected response: {response:?}"),
        };

        let peer_public = registered_data.pub_key.inner();
        let static_private = x25519_dalek::StaticSecret::from(private_key.to_bytes());

        // let private_key_bs64 = general_purpose::STANDARD.encode(static_private.as_bytes());
        let public_key_bs64 = base64::encode(peer_public.as_bytes());

        let private_key_hex = hex::encode(static_private.to_bytes());
        let public_key_hex = hex::encode(peer_public.as_bytes());

        info!("WG connection details");
        // info!("Our private key: {}", private_key_bs64);
        info!("Peer public key: {}", public_key_bs64);
        info!(
            "ips {}(v4), port {}",
            registered_data.private_ip, registered_data.wg_port,
        );

        let gateway_ip = match gateway_host {
            NetworkAddress::Hostname(host) => lookup_host(&host)?
                .first()
                .map(|ip| ip.to_string())
                .unwrap_or_default(),
            NetworkAddress::IpAddr(ip) => match ip {
                IpAddr::V4(ip) => ip.to_string(),
                IpAddr::V6(ip) => format!("[{}]", ip),
            },
        };

        let wg_endpoint = format!("{}:{}", gateway_ip, registered_data.wg_port);

        info!("Successfully registered with the gateway");

        wg_outcome.can_register = true;

        if wg_outcome.can_register {
            let netstack_request = netstack::NetstackRequest {
                wg_ip: registered_data.private_ip.to_string(),
                private_key: private_key_hex.clone(),
                public_key: public_key_hex.clone(),
                endpoint: wg_endpoint.clone(),
                ..Default::default()
            };

            let netstack_response = NetstackCallImpl::ping(&netstack_request);

            info!("Wireguard probe response for IPv4: {:?}", netstack_response);
            wg_outcome.can_handshake_v4 = netstack_response.can_handshake;
            wg_outcome.can_resolve_dns_v4 = netstack_response.can_resolve_dns;
            wg_outcome.ping_hosts_performance_v4 =
                netstack_response.received_hosts as f32 / netstack_response.sent_hosts as f32;
            wg_outcome.ping_ips_performance_v4 =
                netstack_response.received_ips as f32 / netstack_response.sent_ips as f32;
        }
    }

    Ok(wg_outcome)
}
