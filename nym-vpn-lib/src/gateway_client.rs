use crate::error::Result;
use nym_config::defaults::DEFAULT_NYM_NODE_HTTP_PORT;
use nym_crypto::asymmetric::encryption;
use nym_node_requests::api::client::NymNodeApiClientExt;
use nym_node_requests::api::v1::gateway::client_interfaces::wireguard::models::{
    ClientMessage, ClientRegistrationResponse, InitMessage, PeerPublicKey,
};
use nym_validator_client::NymApiClient;
use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;
use talpid_types::net::wireguard::PublicKey;
use tracing::{debug, info};
use url::Url;

const DEFAULT_API_URL: &str = "http://127.0.0.1:8000";

#[derive(Clone)]
pub struct Config {
    pub(crate) api_url: Url,
    pub(crate) local_private_key: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            api_url: DEFAULT_API_URL.parse().unwrap(),
            local_private_key: Default::default(),
        }
    }
}

impl Config {
    pub fn api_url(&self) -> &Url {
        &self.api_url
    }
    pub fn with_custom_api_url(mut self, api_url: Url) -> Self {
        self.api_url = api_url;
        self
    }
    pub fn with_local_private_key(mut self, local_private_key: String) -> Self {
        self.local_private_key = Some(local_private_key);
        self
    }
}

pub struct GatewayClient {
    api_client: NymApiClient,
    keypair: Option<encryption::KeyPair>,
}
#[derive(Clone, Debug)]
pub struct GatewayData {
    pub(crate) public_key: PublicKey,
    pub(crate) endpoint: SocketAddr,
    pub(crate) private_ip: IpAddr,
}

impl GatewayClient {
    pub fn new(config: Config) -> Result<Self> {
        let api_client = NymApiClient::new(config.api_url);

        let keypair = if let Some(local_private_key) = config.local_private_key {
            let private_key_intermediate = PublicKey::from_base64(&local_private_key)
                .map_err(|_| crate::error::Error::InvalidWireGuardKey)?;
            let private_key =
                encryption::PrivateKey::from_bytes(private_key_intermediate.as_bytes())?;
            let public_key = encryption::PublicKey::from(&private_key);
            let keypair =
                encryption::KeyPair::from_bytes(&private_key.to_bytes(), &public_key.to_bytes())
                    .expect("The keys should be valid from the previous decoding");
            Some(keypair)
        } else {
            None
        };

        Ok(GatewayClient {
            api_client,
            keypair,
        })
    }

    pub async fn lookup_gateway_ip(&self, gateway_identity: &str) -> Result<IpAddr> {
        self.api_client
            .get_cached_gateways()
            .await?
            .iter()
            .find_map(|gateway_bond| {
                if gateway_bond.identity() == gateway_identity {
                    Some(gateway_bond.gateway().host.clone())
                } else {
                    None
                }
            })
            .ok_or(crate::error::Error::InvalidGatewayID)
            .and_then(|ip| {
                ip.parse()
                    .map_err(|_| crate::error::Error::InvalidGatewayID)
            })
    }

    pub async fn register_wireguard(&self, gateway_identity: &str) -> Result<GatewayData> {
        info!("Lookup ip for {}", gateway_identity);
        let gateway_host = self.lookup_gateway_ip(gateway_identity).await?;
        info!("Received wg gateway ip: {}", gateway_host);

        let gateway_api_client = nym_node_requests::api::Client::new_url(
            format!("{}:{}", gateway_host, DEFAULT_NYM_NODE_HTTP_PORT),
            None,
        )?;

        // In the CLI it's ensured that the keypair is always present when wireguard is enabled.
        let keypair = self.keypair.as_ref().unwrap();

        debug!("Registering with the wg gateway...");
        let init_message = ClientMessage::Initial(InitMessage {
            pub_key: PeerPublicKey::new(keypair.public_key().to_bytes().try_into().unwrap()),
        });
        let ClientRegistrationResponse::PendingRegistration {
            nonce,
            gateway_data,
            wg_port,
        } = gateway_api_client
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
        gateway_data.verify(keypair.private_key(), nonce)?;

        // let mut mac = HmacSha256::new_from_slice(client_dh.as_bytes()).unwrap();
        // mac.update(client_static_public.as_bytes());
        // mac.update(&nonce.to_le_bytes());
        // let mac = mac.finalize().into_bytes();
        //
        // let finalized_message = ClientMessage::Final(GatewayClient {
        //     pub_key: PeerPublicKey::new(client_static_public),
        //     mac: ClientMac::new(mac.as_slice().to_vec()),
        // });
        let gateway_data = GatewayData {
            public_key: PublicKey::from(gateway_data.pub_key().to_bytes()),
            endpoint: SocketAddr::from_str(&format!("{}:{}", gateway_host, wg_port))?,
            private_ip: "10.1.0.2".parse().unwrap(), // placeholder value for now
        };

        Ok(gateway_data)
    }
}
