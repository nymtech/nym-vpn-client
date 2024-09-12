// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{net::IpAddr, str::FromStr};

use itertools::Itertools;
use nym_sdk::mixnet::NodeIdentity;
use nym_topology::IntoGatewayNode;
use rand::seq::IteratorRandom;
use tracing::error;

use crate::{error::Result, AuthAddress, Country, Error, IpPacketRouterAddress};

// Decimal between 0 and 1 representing the performance of a gateway, measured over 24h.
type Performance = u8;

#[derive(Clone)]
pub struct Gateway {
    pub identity: NodeIdentity,
    pub location: Option<Location>,
    pub ipr_address: Option<IpPacketRouterAddress>,
    pub authenticator_address: Option<AuthAddress>,
    pub last_probe: Option<Probe>,
    pub host: Option<nym_topology::NetworkAddress>,
    pub clients_ws_port: Option<u16>,
    pub clients_wss_port: Option<u16>,
    pub performance: Option<Performance>,
}

impl std::fmt::Debug for Gateway {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Gateway")
            .field("identity", &self.identity.to_base58_string())
            .field("location", &self.location)
            .field("ipr_address", &self.ipr_address)
            .field("authenticator_address", &self.authenticator_address)
            .field("last_probe", &self.last_probe)
            .field("host", &self.host)
            .field("clients_ws_port", &self.clients_ws_port)
            .field("clients_wss_port", &self.clients_wss_port)
            .field("performance", &self.performance)
            .finish()
    }
}

impl Gateway {
    pub fn identity(&self) -> &NodeIdentity {
        &self.identity
    }

    pub fn two_letter_iso_country_code(&self) -> Option<&str> {
        self.location
            .as_ref()
            .map(|l| l.two_letter_iso_country_code.as_str())
    }

    pub fn is_two_letter_iso_country_code(&self, code: &str) -> bool {
        self.two_letter_iso_country_code()
            .map_or(false, |gw_code| gw_code == code)
    }

    pub fn has_ipr_address(&self) -> bool {
        self.ipr_address.is_some()
    }

    pub fn clients_address_no_tls(&self) -> Option<String> {
        match (&self.host, &self.clients_ws_port) {
            (Some(host), Some(port)) => Some(format!("ws://{}:{}", host, port)),
            _ => None,
        }
    }

    pub fn clients_address_tls(&self) -> Option<String> {
        match (&self.host, &self.clients_wss_port) {
            (Some(host), Some(port)) => Some(format!("wss://{}:{}", host, port)),
            _ => None,
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Location {
    pub two_letter_iso_country_code: String,
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Probe {
    pub last_updated_utc: String,
    pub outcome: ProbeOutcome,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProbeOutcome {
    pub as_entry: Entry,
    pub as_exit: Option<Exit>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    pub can_connect: bool,
    pub can_route: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Exit {
    pub can_connect: bool,
    pub can_route_ip_v4: bool,
    pub can_route_ip_external_v4: bool,
    pub can_route_ip_v6: bool,
    pub can_route_ip_external_v6: bool,
}

impl From<nym_vpn_api_client::response::Location> for Location {
    fn from(location: nym_vpn_api_client::response::Location) -> Self {
        Location {
            two_letter_iso_country_code: location.two_letter_iso_country_code,
            latitude: location.latitude,
            longitude: location.longitude,
        }
    }
}

impl From<nym_vpn_api_client::response::Probe> for Probe {
    fn from(probe: nym_vpn_api_client::response::Probe) -> Self {
        Probe {
            last_updated_utc: probe.last_updated_utc,
            outcome: ProbeOutcome::from(probe.outcome),
        }
    }
}

impl From<nym_vpn_api_client::response::ProbeOutcome> for ProbeOutcome {
    fn from(outcome: nym_vpn_api_client::response::ProbeOutcome) -> Self {
        ProbeOutcome {
            as_entry: Entry::from(outcome.as_entry),
            as_exit: outcome.as_exit.map(Exit::from),
        }
    }
}

impl From<nym_vpn_api_client::response::Entry> for Entry {
    fn from(entry: nym_vpn_api_client::response::Entry) -> Self {
        Entry {
            can_connect: entry.can_connect,
            can_route: entry.can_route,
        }
    }
}

impl From<nym_vpn_api_client::response::Exit> for Exit {
    fn from(exit: nym_vpn_api_client::response::Exit) -> Self {
        Exit {
            can_connect: exit.can_connect,
            can_route_ip_v4: exit.can_route_ip_v4,
            can_route_ip_external_v4: exit.can_route_ip_external_v4,
            can_route_ip_v6: exit.can_route_ip_v6,
            can_route_ip_external_v6: exit.can_route_ip_external_v6,
        }
    }
}

impl TryFrom<nym_vpn_api_client::response::NymDirectoryGateway> for Gateway {
    type Error = Error;

    fn try_from(gateway: nym_vpn_api_client::response::NymDirectoryGateway) -> Result<Self> {
        let identity =
            NodeIdentity::from_base58_string(&gateway.identity_key).map_err(|source| {
                Error::NodeIdentityFormattingError {
                    identity: gateway.identity_key,
                    source,
                }
            })?;

        let host = gateway
            .entry
            .hostname
            .map(nym_topology::NetworkAddress::Hostname)
            .or(gateway
                .ip_addresses
                .first()
                .cloned()
                .and_then(|ip| IpAddr::from_str(&ip).ok())
                .map(nym_topology::NetworkAddress::IpAddr));

        let performance = gateway
            .performance
            .parse::<f64>()
            .map(|p| p * 100.0)
            .map(|p| p.round() as u8)
            .map(|p| p.clamp(0, 100))
            .ok();

        Ok(Gateway {
            identity,
            location: Some(gateway.location.into()),
            ipr_address: gateway
                .ipr_address
                .and_then(|ipr| IpPacketRouterAddress::try_from_base58_string(&ipr).ok()),
            authenticator_address: gateway
                .authenticator_address
                .and_then(|auth| AuthAddress::try_from_base58_string(&auth).ok()),
            last_probe: gateway.last_probe.map(Probe::from),
            host,
            clients_ws_port: Some(gateway.entry.ws_port),
            clients_wss_port: gateway.entry.wss_port,
            performance,
        })
    }
}

impl TryFrom<nym_validator_client::models::DescribedGateway> for Gateway {
    type Error = Error;

    fn try_from(gateway: nym_validator_client::models::DescribedGateway) -> Result<Self> {
        let identity = NodeIdentity::from_base58_string(gateway.identity()).map_err(|source| {
            Error::NodeIdentityFormattingError {
                identity: gateway.identity().to_string(),
                source,
            }
        })?;
        let location = gateway
            .self_described
            .as_ref()
            .and_then(|d| d.auxiliary_details.location)
            .map(|l| Location {
                two_letter_iso_country_code: l.alpha2.to_string(),
                ..Default::default()
            });
        let ipr_address = gateway
            .self_described
            .as_ref()
            .and_then(|d| d.ip_packet_router.clone())
            .and_then(|ipr| {
                IpPacketRouterAddress::try_from_base58_string(&ipr.address)
                    .inspect_err(|err| error!("Failed to parse IPR address: {err}"))
                    .ok()
            });
        let authenticator_address = gateway
            .self_described
            .as_ref()
            .and_then(|d| d.authenticator.clone())
            .and_then(|a| {
                AuthAddress::try_from_base58_string(&a.address)
                    .inspect_err(|err| error!("Failed to parse authenticator address: {err}"))
                    .ok()
            });
        let gateway = nym_topology::gateway::Node::try_from(gateway).ok();
        let host = gateway.clone().map(|g| g.host);
        let clients_ws_port = gateway.as_ref().map(|g| g.clients_ws_port);
        let clients_wss_port = gateway.and_then(|g| g.clients_wss_port);
        Ok(Gateway {
            identity,
            location,
            ipr_address,
            authenticator_address,
            last_probe: None,
            host,
            clients_ws_port,
            clients_wss_port,
            performance: None,
        })
    }
}

#[derive(Debug, Clone)]
pub struct GatewayList {
    gateways: Vec<Gateway>,
}

impl GatewayList {
    pub fn new(gateways: Vec<Gateway>) -> Self {
        GatewayList { gateways }
    }

    // Returns a list of all locations of the gateways, including duplicates
    fn all_locations(&self) -> impl Iterator<Item = &Location> {
        self.gateways
            .iter()
            .filter_map(|gateway| gateway.location.as_ref())
    }

    pub fn all_countries(&self) -> Vec<Country> {
        self.all_locations()
            .cloned()
            .map(Country::from)
            .unique()
            .collect()
    }

    pub fn all_iso_codes(&self) -> Vec<String> {
        self.all_countries()
            .into_iter()
            .map(|country| country.iso_code().to_string())
            .collect()
    }

    pub fn gateway_with_identity(&self, identity: &NodeIdentity) -> Option<&Gateway> {
        self.gateways
            .iter()
            .find(|gateway| gateway.identity() == identity)
    }

    pub fn gateways_located_at(&self, code: String) -> impl Iterator<Item = &Gateway> {
        self.gateways.iter().filter(move |gateway| {
            gateway
                .two_letter_iso_country_code()
                .map_or(false, |gw_code| gw_code == code)
        })
    }

    pub fn random_gateway(&self) -> Option<Gateway> {
        self.gateways
            .iter()
            .choose(&mut rand::thread_rng())
            .cloned()
    }

    pub fn random_gateway_located_at(&self, code: String) -> Option<Gateway> {
        self.gateways_located_at(code)
            .choose(&mut rand::thread_rng())
            .cloned()
    }

    pub fn remove_gateway(&mut self, entry_gateway: &Gateway) {
        self.gateways
            .retain(|gateway| gateway.identity() != entry_gateway.identity());
    }

    pub fn len(&self) -> usize {
        self.gateways.len()
    }

    pub fn is_empty(&self) -> bool {
        self.gateways.is_empty()
    }

    pub fn into_exit_gateways(self) -> GatewayList {
        let gw = self
            .gateways
            .into_iter()
            .filter(Gateway::has_ipr_address)
            .collect();
        Self::new(gw)
    }

    pub fn into_countries(self) -> Vec<Country> {
        self.all_countries()
    }

    pub fn into_inner(self) -> Vec<Gateway> {
        self.gateways
    }

    pub(crate) async fn random_low_latency_gateway(&self) -> Result<Gateway> {
        let mut rng = rand::rngs::OsRng;
        nym_client_core::init::helpers::choose_gateway_by_latency(&mut rng, &self.gateways, false)
            .await
            .map_err(|err| Error::FailedToSelectGatewayBasedOnLowLatency { source: err })
    }
}

impl IntoIterator for GatewayList {
    type Item = Gateway;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.gateways.into_iter()
    }
}

impl nym_client_core::init::helpers::ConnectableGateway for Gateway {
    fn identity(&self) -> &nym_sdk::mixnet::NodeIdentity {
        self.identity()
    }

    fn clients_address(&self) -> String {
        // This is a bit of a sharp edge, but temporary until we can remove Option from host
        // and tls port when we add these to the vpn API endpoints.
        self.clients_address_tls()
            .or(self.clients_address_no_tls())
            .unwrap_or("ws://".to_string())
    }

    fn is_wss(&self) -> bool {
        self.clients_address_tls().is_some()
    }
}
