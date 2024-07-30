// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use itertools::Itertools;
use nym_sdk::mixnet::NodeIdentity;
use nym_topology::IntoGatewayNode;
use rand::seq::IteratorRandom;
use tracing::error;

use crate::{error::Result, AuthAddress, Error, IpPacketRouterAddress};

#[derive(Clone, Debug)]
pub struct Gateway {
    pub identity: NodeIdentity,
    pub location: Option<Location>,
    pub ipr_address: Option<IpPacketRouterAddress>,
    pub authenticator_address: Option<AuthAddress>,
    pub last_probe: Option<Probe>,
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
}

#[derive(Debug, Clone, PartialEq)]
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

impl From<nym_vpn_api_client::Location> for Location {
    fn from(location: nym_vpn_api_client::Location) -> Self {
        Location {
            two_letter_iso_country_code: location.two_letter_iso_country_code,
            latitude: location.latitude,
            longitude: location.longitude,
        }
    }
}

impl From<nym_vpn_api_client::responses::Probe> for Probe {
    fn from(probe: nym_vpn_api_client::responses::Probe) -> Self {
        Probe {
            last_updated_utc: probe.last_updated_utc,
            outcome: ProbeOutcome::from(probe.outcome),
        }
    }
}

impl From<nym_vpn_api_client::responses::ProbeOutcome> for ProbeOutcome {
    fn from(outcome: nym_vpn_api_client::responses::ProbeOutcome) -> Self {
        ProbeOutcome {
            as_entry: Entry::from(outcome.as_entry),
            as_exit: outcome.as_exit.map(Exit::from),
        }
    }
}

impl From<nym_vpn_api_client::responses::Entry> for Entry {
    fn from(entry: nym_vpn_api_client::responses::Entry) -> Self {
        Entry {
            can_connect: entry.can_connect,
            can_route: entry.can_route,
        }
    }
}

impl From<nym_vpn_api_client::responses::Exit> for Exit {
    fn from(exit: nym_vpn_api_client::responses::Exit) -> Self {
        Exit {
            can_connect: exit.can_connect,
            can_route_ip_v4: exit.can_route_ip_v4,
            can_route_ip_external_v4: exit.can_route_ip_external_v4,
            can_route_ip_v6: exit.can_route_ip_v6,
            can_route_ip_external_v6: exit.can_route_ip_external_v6,
        }
    }
}

impl TryFrom<nym_vpn_api_client::Gateway> for Gateway {
    type Error = Error;

    fn try_from(gateway: nym_vpn_api_client::Gateway) -> Result<Self> {
        let identity =
            NodeIdentity::from_base58_string(&gateway.identity_key).map_err(|source| {
                Error::NodeIdentityFormattingError {
                    identity: gateway.identity_key,
                    source,
                }
            })?;
        Ok(Gateway {
            identity,
            location: Some(gateway.location.into()),
            ipr_address: None,
            authenticator_address: None,
            last_probe: gateway.last_probe.map(Probe::from),
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
        Ok(Gateway {
            identity,
            location: None,
            ipr_address,
            authenticator_address,
            last_probe: None,
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

    pub fn all_locations(&self) -> impl Iterator<Item = &Location> {
        self.gateways
            .iter()
            .filter_map(|gateway| gateway.location.as_ref())
    }

    pub fn all_iso_codes(&self) -> Vec<String> {
        self.all_locations()
            .map(|code| code.two_letter_iso_country_code.clone())
            .unique()
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

    pub fn into_inner(self) -> Vec<Gateway> {
        self.gateways
    }
}

impl IntoIterator for GatewayList {
    type Item = Gateway;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.gateways.into_iter()
    }
}
