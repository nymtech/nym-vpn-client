// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use itertools::Itertools;
use nym_sdk::mixnet::NodeIdentity;
use rand::seq::IteratorRandom;

use crate::{AuthAddress, IpPacketRouterAddress};

#[derive(Clone, Debug)]
pub struct Gateway {
    pub identity: NodeIdentity,
    pub location: Option<Location>,
    pub ipr_address: Option<IpPacketRouterAddress>,
    pub authenticator_address: Option<AuthAddress>,
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
}

#[derive(Debug, Clone, PartialEq)]
pub struct Location {
    pub two_letter_iso_country_code: String,
    pub latitude: f64,
    pub longitude: f64,
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

impl From<nym_vpn_api_client::Gateway> for Gateway {
    fn from(gateway: nym_vpn_api_client::Gateway) -> Self {
        Gateway {
            identity: NodeIdentity::from_base58_string(&gateway.identity_key).unwrap(),
            location: Some(gateway.location.into()),
            ipr_address: None,
            authenticator_address: None,
        }
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
}
