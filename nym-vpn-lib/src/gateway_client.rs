// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::error::{Error, Result};
use crate::mixnet_processor::IpPacketRouterAddress;
use hickory_resolver::config::{ResolverConfig, ResolverOpts};
use hickory_resolver::TokioAsyncResolver;
use itertools::Itertools;
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
}

#[derive(Clone, Debug, Default)]
pub struct WgConfig {
    pub(crate) local_private_key: Option<String>,
}

impl WgConfig {
    pub fn new() -> Self {
        WgConfig {
            local_private_key: None,
        }
    }

    pub fn with_local_private_key(mut self, local_private_key: String) -> Self {
        self.local_private_key = Some(local_private_key);
        self
    }
}

// The entry point is always a gateway identity, or some other entry that can be resolved to a
// gateway identity.
#[derive(Clone, Debug, Deserialize, Serialize, uniffi::Enum)]
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
#[derive(Clone, Debug, Deserialize, Serialize, uniffi::Enum)]
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

fn select_random_described_gateway<'a, I>(gateways: I) -> Result<&'a DescribedGatewayWithLocation>
where
    I: IntoIterator<Item = &'a DescribedGatewayWithLocation>,
{
    gateways
        .into_iter()
        .choose(&mut rand::thread_rng())
        .ok_or(Error::FailedToSelectGatewayRandomly)
}

fn select_random_gateway_node<'a, I>(gateways: I) -> Result<NodeIdentity>
where
    I: IntoIterator<Item = &'a DescribedGatewayWithLocation>,
{
    let random_gateway = select_random_described_gateway(gateways)?;
    NodeIdentity::from_base58_string(random_gateway.identity_key())
        .map_err(|_| Error::NodeIdentityFormattingError)
}

async fn select_random_low_latency_gateway_node(
    gateways: &[DescribedGatewayWithLocation],
) -> Result<NodeIdentity> {
    let mut rng = rand::rngs::OsRng;
    let must_use_tls = FORCE_TLS_FOR_GATEWAY_SELECTION;
    let gateway_nodes: Vec<nym_topology::gateway::Node> = gateways
        .iter()
        .filter_map(|gateway| nym_topology::gateway::Node::try_from(&gateway.gateway).ok())
        .collect();
    choose_gateway_by_latency(&mut rng, &gateway_nodes, must_use_tls)
        .await
        .map(|gateway| *gateway.identity())
        .map_err(|err| Error::FailedToSelectGatewayBasedOnLowLatency { source: err })
}

fn list_all_country_iso_codes<'a, I>(gateways: I) -> Vec<String>
where
    I: IntoIterator<Item = &'a DescribedGatewayWithLocation>,
{
    gateways
        .into_iter()
        .filter_map(|gateway| gateway.two_letter_iso_country_code())
        .unique()
        .collect()
}

async fn select_random_low_latency_described_gateway(
    gateways: &[DescribedGatewayWithLocation],
) -> Result<&DescribedGatewayWithLocation> {
    let low_latency_gateway = select_random_low_latency_gateway_node(gateways).await?;
    gateways
        .iter()
        .find(|gateway| gateway.identity_key() == &low_latency_gateway.to_string())
        .ok_or(Error::NoMatchingGateway)
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
                // Caution: if an explorer-api for a different network was specified, then
                // none of the gateways will have an associated location. There is a check
                // against this earlier in the call stack to guard against this scenario.
                let gateways_with_specified_location = gateways
                    .iter()
                    .filter(|g| g.is_two_letter_iso_country_code(location));
                if gateways_with_specified_location.clone().count() == 0 {
                    return Err(Error::NoMatchingEntryGatewayForLocation {
                        requested_location: location.to_string(),
                        available_countries: list_all_country_iso_codes(gateways),
                    });
                }
                select_random_gateway_node(gateways_with_specified_location)
            }
            EntryPoint::RandomLowLatency => {
                log::info!("Selecting a random low latency entry gateway");
                select_random_low_latency_gateway_node(gateways).await
            }
            EntryPoint::Random => {
                log::info!("Selecting a random entry gateway");
                select_random_gateway_node(gateways)
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
                    .clone()
                    .filter(|gateway| gateway.is_two_letter_iso_country_code(location));
                let random_gateway =
                    gateways_with_specified_location.choose(&mut rand::thread_rng());

                match random_gateway {
                    Some(random_gateway) => {
                        IpPacketRouterAddress::try_from_described_gateway(&random_gateway.gateway)
                    }
                    None => Err(Error::NoMatchingExitGatewayForLocation {
                        requested_location: location.to_string(),
                        available_countries: list_all_country_iso_codes(exit_gateways),
                    }),
                }
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

    pub fn location(&self) -> Option<Location> {
        self.location.clone()
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

    pub fn country_name(&self) -> Option<String> {
        self.location.as_ref().map(|l| l.country_name.clone())
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
}

pub struct WgGatewayClient {
    keypair: Option<encryption::KeyPair>,
}

#[derive(Clone, Debug)]
pub struct GatewayData {
    pub(crate) public_key: PublicKey,
    pub(crate) endpoint: SocketAddr,
    pub(crate) private_ip: IpAddr,
}

impl WgGatewayClient {
    pub fn new(config: WgConfig) -> Result<Self> {
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

        Ok(WgGatewayClient { keypair })
    }

    pub async fn register_wireguard(
        &self,
        // gateway_identity: &str,
        gateway_host: IpAddr,
        wg_ip: IpAddr,
    ) -> Result<GatewayData> {
        // info!("Lookup ip for {}", gateway_identity);
        // let gateway_host = self.lookup_gateway_ip(gateway_identity).await?;
        // info!("Received wg gateway ip: {}", gateway_host);

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

impl GatewayClient {
    pub fn new(config: Config) -> Result<Self> {
        let api_client = NymApiClient::new(config.api_url);
        let explorer_client = if let Some(url) = config.explorer_url {
            Some(ExplorerClient::new(url)?)
        } else {
            None
        };

        Ok(GatewayClient {
            api_client,
            explorer_client,
        })
    }

    async fn lookup_described_gateways(&self) -> Result<Vec<DescribedGateway>> {
        log::info!("Fetching gateways from nym-api...");
        self.api_client
            .get_cached_described_gateways()
            .await
            .map_err(|source| Error::FailedToLookupDescribedGateways { source })
    }

    async fn lookup_gateways_in_explorer(&self) -> Option<Result<Vec<PrettyDetailedGatewayBond>>> {
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

    pub async fn lookup_described_exit_gateways_with_location(
        &self,
    ) -> Result<Vec<DescribedGatewayWithLocation>> {
        let described_gateways = self.lookup_described_gateways_with_location().await?;
        Ok(described_gateways
            .into_iter()
            .filter(|gateway| gateway.has_ip_packet_router())
            .collect())
    }

    pub async fn lookup_low_latency_entry_gateway(&self) -> Result<DescribedGatewayWithLocation> {
        let described_gateways = self.lookup_described_gateways_with_location().await?;
        select_random_low_latency_described_gateway(&described_gateways)
            .await
            .cloned()
    }

    pub async fn lookup_all_countries(&self) -> Result<Vec<Location>> {
        let described_gateways = self.lookup_described_gateways_with_location().await?;
        Ok(described_gateways
            .into_iter()
            .filter_map(|gateway| gateway.location)
            .unique_by(|location| location.country_name.clone())
            .collect())
    }

    pub async fn lookup_all_countries_iso(&self) -> Result<Vec<Location>> {
        let described_gateways = self.lookup_described_gateways_with_location().await?;
        Ok(described_gateways
            .into_iter()
            .filter_map(|gateway| gateway.location)
            .unique_by(|location| location.two_letter_iso_country_code.clone())
            .collect())
    }

    pub async fn lookup_all_exit_countries_iso(&self) -> Result<Vec<Location>> {
        let described_gateways = self.lookup_described_exit_gateways_with_location().await?;
        Ok(described_gateways
            .into_iter()
            .filter_map(|gateway| gateway.location)
            .unique_by(|location| location.two_letter_iso_country_code.clone())
            .collect())
    }

    pub async fn lookup_all_exit_countries(&self) -> Result<Vec<Location>> {
        let described_gateways = self.lookup_described_exit_gateways_with_location().await?;
        Ok(described_gateways
            .into_iter()
            .filter_map(|gateway| gateway.location)
            .unique_by(|location| location.country_name.clone())
            .collect())
    }

    pub async fn lookup_gateway_ip(&self, gateway_identity: &str) -> Result<IpAddr> {
        let ip_or_hostname = self
            .api_client
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
            ))?;

        // If it's a plain IP
        if let Ok(ip) = ip_or_hostname.parse::<IpAddr>() {
            return Ok(ip);
        }

        // If it's not an IP, try to resolve it as a hostname
        let ip = try_resolve_hostname(&ip_or_hostname).await?;
        info!("Resolved {ip_or_hostname} to {ip}");
        Ok(ip)
    }
}

async fn try_resolve_hostname(hostname: &str) -> Result<IpAddr> {
    debug!("Trying to resolve hostname: {hostname}");
    let resolver = TokioAsyncResolver::tokio(ResolverConfig::default(), ResolverOpts::default());
    let addrs = resolver.lookup_ip(hostname).await.map_err(|err| {
        tracing::error!("Failed to resolve gateway hostname: {}", err);
        Error::FailedToDnsResolveGateway {
            hostname: hostname.to_string(),
            source: err,
        }
    })?;
    debug!("Resolved to: {addrs:?}");

    // Just pick the first one
    addrs
        .iter()
        .next()
        .ok_or(Error::ResolvedHostnameButNoIp(hostname.to_string()))
}
