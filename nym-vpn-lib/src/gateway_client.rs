// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::error::{Error, Result};
use crate::mixnet_processor::IpPacketRouterAddress;
#[cfg(target_os = "macos")]
use crate::UniffiCustomTypeConverter;
use nym_client_core::init::helpers::choose_gateway_by_latency;
use nym_config::defaults::DEFAULT_NYM_NODE_HTTP_PORT;
use nym_crypto::asymmetric::encryption;
use nym_explorer_client::{ExplorerClient, Location, PrettyDetailedGatewayBond};
use nym_node_requests::api::client::NymNodeApiClientExt;
use nym_node_requests::api::v1::gateway::client_interfaces::wireguard::models::{
    ClientMessage, ClientRegistrationResponse, InitMessage, PeerPublicKey,
};
use nym_sdk::mixnet::{NodeIdentity, Recipient};
use nym_validator_client::client::IdentityKey;
use nym_validator_client::models::DescribedGateway;
use nym_validator_client::NymApiClient;
use rand::seq::IteratorRandom;
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;
use talpid_types::net::wireguard::PublicKey;
use tracing::{debug, info};
use url::Url;

const FORCE_TLS_FOR_GATEWAY_SELECTION: bool = false;

#[derive(Clone, Debug)]
pub struct Config {
    pub(crate) api_url: Url,
    pub(crate) explorer_url: Option<Url>,
    pub(crate) local_private_key: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        let network_defaults = nym_sdk::NymNetworkDetails::default();
        let default_api_url = network_defaults
            .endpoints
            .first()
            .expect("rust sdk mainnet default incorrectly configured")
            .api_url
            .clone()
            .expect("rust sdk mainnet default missing api_url")
            .parse()
            .expect("rust sdk mainnet default api_url not parseable");
        let default_explorer_url = network_defaults.explorer_api.clone().map(|url| {
            url.parse()
                .expect("rust sdk mainnet default explorer url not parseable")
        });

        Config {
            api_url: default_api_url,
            explorer_url: default_explorer_url,
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

    pub fn explorer_url(&self) -> Option<&Url> {
        self.explorer_url.as_ref()
    }

    pub fn with_custom_explorer_url(mut self, explorer_url: Url) -> Self {
        self.explorer_url = Some(explorer_url);
        self
    }

    pub fn with_local_private_key(mut self, local_private_key: String) -> Self {
        self.local_private_key = Some(local_private_key);
        self
    }
}

// The entry point is always a gateway identity, or some other entry that can be resolved to a
// gateway identity.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum EntryPoint {
    Gateway { identity: NodeIdentity },
    // NOTE: Consider using a crate with strongly typed country codes instead of strings
    Location { location: String },
    RandomLowLatency,
    Random,
}

impl EntryPoint {
    pub fn is_location(&self) -> bool {
        matches!(self, EntryPoint::Location { .. })
    }
}

// The exit point is a nym-address, but if the exit ip-packet-router is running embedded on a
// gateway, we can refer to it by the gateway identity.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[allow(clippy::large_enum_variant)]
pub enum ExitPoint {
    // An explicit exit address. This is useful when the exit ip-packet-router is running as a
    // standalone entity (private).
    Address { address: Recipient },
    // An explicit exit gateway identity. This is useful when the exit ip-packet-router is running
    // embedded on a gateway.
    Gateway { identity: NodeIdentity },
    // NOTE: Consider using a crate with strongly typed country codes instead of strings
    Location { location: String },
}

impl ExitPoint {
    pub fn is_location(&self) -> bool {
        matches!(self, ExitPoint::Location { .. })
    }
}

#[cfg(target_os = "macos")]
impl UniffiCustomTypeConverter for Recipient {
    type Builtin = String;

    fn into_custom(val: Self::Builtin) -> uniffi::Result<Self> {
        Ok(Recipient::try_from_base58_string(val)?)
    }

    fn from_custom(obj: Self) -> Self::Builtin {
        obj.to_string()
    }
}

#[cfg(target_os = "macos")]
impl UniffiCustomTypeConverter for NodeIdentity {
    type Builtin = String;

    fn into_custom(val: Self::Builtin) -> uniffi::Result<Self> {
        Ok(NodeIdentity::from_base58_string(val)?)
    }

    fn from_custom(obj: Self) -> Self::Builtin {
        obj.to_base58_string()
    }
}

impl EntryPoint {
    pub async fn lookup_gateway_identity(
        &self,
        gateways: &[DescribedGatewayWithLocation],
    ) -> Result<NodeIdentity> {
        match &self {
            EntryPoint::Gateway { identity } => {
                // Confirm up front that the gateway identity is in the list of gateways from the
                // directory.
                gateways
                    .iter()
                    .find(|gateway| gateway.identity_key() == &identity.to_string())
                    .ok_or(Error::NoMatchingGateway)?;
                Ok(*identity)
            }
            EntryPoint::Location { location } => {
                // If an explorer-api for a different network was specified, then none of the
                // gateways will have an associated location.
                let gateways_with_specified_location = gateways
                    .iter()
                    .filter(|g| g.is_two_letter_iso_country_code(location));
                let random_gateway = gateways_with_specified_location
                    .choose(&mut rand::thread_rng())
                    .ok_or(Error::NoMatchingGatewayForLocation(location.to_string()))?;
                NodeIdentity::from_base58_string(random_gateway.identity_key())
                    .map_err(|_| Error::NodeIdentityFormattingError)
            }
            EntryPoint::RandomLowLatency => {
                // Recall, even though the mixnet client is able to randomly select a gateway, we
                // have to do it up front since it affects how we setup the routing table when
                // wireguard is enabled for the first hop
                log::info!("Selecting a random low latency entry gateway");
                let mut rng = rand::rngs::OsRng;
                let must_use_tls = FORCE_TLS_FOR_GATEWAY_SELECTION;
                let gateway_nodes: Vec<nym_topology::gateway::Node> = gateways
                    .iter()
                    .filter_map(|gateway| {
                        nym_topology::gateway::Node::try_from(&gateway.gateway).ok()
                    })
                    .collect();
                choose_gateway_by_latency(&mut rng, &gateway_nodes, must_use_tls)
                    .await
                    .map(|gateway| *gateway.identity())
                    .map_err(|err| Error::FailedToSelectGatewayBasedOnLowLatency { source: err })
            }
            EntryPoint::Random => {
                // Recall, even though the mixnet client is able to randomly select a gateway, we
                // have to do it up front since it affects how we setup the routing table when
                // wireguard is enabled for the first hop
                log::info!("Selecting a random entry gateway");
                let random_gateway = gateways
                    .iter()
                    .choose(&mut rand::thread_rng())
                    .ok_or(Error::FailedToSelectEntryGatewayRandomly)?;
                NodeIdentity::from_base58_string(random_gateway.identity_key())
                    .map_err(|_| Error::NodeIdentityFormattingError)
            }
        }
    }
}

impl ExitPoint {
    pub fn lookup_router_address(
        &self,
        gateways: &[DescribedGatewayWithLocation],
    ) -> Result<IpPacketRouterAddress> {
        match &self {
            ExitPoint::Address { address } => {
                // There is no validation done when a ip packet router is specified by address
                // since it might be private and not available in any directory.
                Ok(IpPacketRouterAddress(*address))
            }
            ExitPoint::Gateway { identity } => {
                let gateway = gateways
                    .iter()
                    .find(|gateway| gateway.identity_key() == &identity.to_string())
                    .ok_or(Error::NoMatchingGateway)?;
                IpPacketRouterAddress::try_from_described_gateway(&gateway.gateway)
            }
            ExitPoint::Location { location } => {
                let exit_gateways = gateways.iter().filter(|g| g.has_ip_packet_router());
                let gateways_with_specified_location = exit_gateways
                    .filter(|gateway| gateway.is_two_letter_iso_country_code(location));
                let random_gateway = gateways_with_specified_location
                    .choose(&mut rand::thread_rng())
                    .ok_or(Error::NoMatchingGatewayForLocation(location.to_string()))?;
                IpPacketRouterAddress::try_from_described_gateway(&random_gateway.gateway)
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct DescribedGatewayWithLocation {
    pub gateway: DescribedGateway,
    pub location: Option<Location>,
}

impl DescribedGatewayWithLocation {
    pub fn identity_key(&self) -> &IdentityKey {
        &self.gateway.bond.gateway.identity_key
    }

    pub fn has_ip_packet_router(&self) -> bool {
        self.gateway
            .self_described
            .as_ref()
            .and_then(|d| d.ip_packet_router.as_ref())
            .is_some()
    }

    pub fn has_location(&self) -> bool {
        self.location.is_some()
    }

    pub fn two_letter_iso_country_code(&self) -> Option<String> {
        self.location
            .as_ref()
            .map(|l| l.two_letter_iso_country_code.clone())
    }

    pub fn is_two_letter_iso_country_code(&self, code: &str) -> bool {
        self.two_letter_iso_country_code()
            .map_or(false, |gateway_iso_code| gateway_iso_code == code)
    }
}

impl From<DescribedGateway> for DescribedGatewayWithLocation {
    fn from(gateway: DescribedGateway) -> Self {
        DescribedGatewayWithLocation {
            gateway,
            location: None,
        }
    }
}

pub struct GatewayClient {
    api_client: NymApiClient,
    explorer_client: Option<ExplorerClient>,
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
        let explorer_client = if let Some(url) = config.explorer_url {
            Some(ExplorerClient::new(url)?)
        } else {
            None
        };

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
            explorer_client,
            keypair,
        })
    }

    pub async fn lookup_described_gateways(&self) -> Result<Vec<DescribedGateway>> {
        log::info!("Fetching gateways from nym-api...");
        self.api_client
            .get_cached_described_gateways()
            .await
            .map_err(|source| Error::FailedToLookupDescribedGateways { source })
    }

    pub async fn lookup_gateways_in_explorer(
        &self,
    ) -> Option<Result<Vec<PrettyDetailedGatewayBond>>> {
        log::info!("Fetching gateway geo-locations from nym-explorer...");
        if let Some(explorer_client) = &self.explorer_client {
            Some(
                explorer_client
                    .get_gateways()
                    .await
                    .map_err(|error| Error::FailedFetchLocationData { error }),
            )
        } else {
            None
        }
    }

    pub async fn lookup_described_gateways_with_location(
        &self,
    ) -> Result<Vec<DescribedGatewayWithLocation>> {
        let described_gateways = self.lookup_described_gateways().await?;
        match self.lookup_gateways_in_explorer().await {
            Some(Ok(gateway_locations)) => described_gateways
                .into_iter()
                .map(|gateway| {
                    let location = gateway_locations
                        .iter()
                        .find(|gateway_location| {
                            gateway_location.gateway.identity_key
                                == gateway.bond.gateway.identity_key
                        })
                        .and_then(|gateway_location| gateway_location.location.clone());
                    Ok(DescribedGatewayWithLocation { gateway, location })
                })
                .collect(),
            Some(Err(error)) => {
                // If there was an error fetching the location data, log it and keep on going
                // without location data. This is not a fatal error since we can still refer to the
                // gateways by identity.
                log::warn!("{error}");
                Ok(described_gateways
                    .into_iter()
                    .map(DescribedGatewayWithLocation::from)
                    .collect())
            }
            None => Ok(described_gateways
                .into_iter()
                .map(DescribedGatewayWithLocation::from)
                .collect()),
        }
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
            .ok_or(Error::RequestedGatewayIdNotFound(
                gateway_identity.to_string(),
            ))
            .and_then(|ip| ip.parse().map_err(|_| Error::InvalidGatewayIp(ip)))
    }

    pub async fn register_wireguard(
        &self,
        gateway_identity: &str,
        wg_ip: IpAddr,
    ) -> Result<GatewayData> {
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
            pub_key: PeerPublicKey::new(keypair.public_key().to_bytes().into()),
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
            private_ip: wg_ip,
            // private_ip: "10.1.0.2".parse().unwrap(), // placeholder value for now
        };

        Ok(gateway_data)
    }
}
