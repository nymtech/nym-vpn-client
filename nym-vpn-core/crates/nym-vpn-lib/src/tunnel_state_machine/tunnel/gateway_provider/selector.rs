// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{net::SocketAddr, sync::Arc};

use nym_crypto::asymmetric::x25519::KeyPair;
use nym_gateway_directory::{
    BlacklistedGateways, EntryPoint, ExitPoint, Gateway, GatewayFilter, GatewayFilters,
    GatewayList, GatewayType, Location,
};
use nym_registration_client::RegistrationNymNode;
use nym_registration_common::{NymNodeInformation, NymNodeLPInformation};
use nym_vpn_lib_types::GatewaySelectionAlgorithm;
use nym_vpn_store::keys::wireguard::{WireguardKeyStore, WireguardKeysDb};

use crate::{
    GatewayDirectoryError,
    tunnel_state_machine::{
        TunnelSettings, TunnelType,
        tunnel::{
            self,
            gateway_provider::{
                gateway_cache::GatewayCache,
                geo_ip::{closest_gateway, same_jurisdiction},
            },
        },
    },
};

#[derive(Clone)]
pub struct GatewayWithKeys {
    gateway: Gateway,
    keys: Arc<KeyPair>,
}

impl TryFrom<GatewayWithKeys> for RegistrationNymNode {
    type Error = tunnel::Error;
    fn try_from(value: GatewayWithKeys) -> Result<Self, Self::Error> {
        let ip_address = value
            .gateway
            .lookup_ip()
            .ok_or(tunnel::Error::NoIpAddressAnnounced {
                gateway_id: value.gateway.identity().to_base58_string(),
            })?;

        if let Some(data) = value.gateway.lewes_protocol_details.as_ref()
            && !data.verify(&value.gateway.identity)
        {
            tracing::warn!(
                "Gateway {} has malformed LP information, something fishy is going on",
                value.gateway.identity
            );
            // Signature check doesn't pass, something fishy is going on
            return Err(tunnel::Error::SelectGateways(Box::new(
                GatewayDirectoryError::MalformedGateway(
                    nym_gateway_directory::Error::MalformedGateway,
                ),
            )));
        }

        let lp_data = value.gateway.lewes_protocol_details.and_then(|data| {
            let kem_keys = data.content.kem_keys().ok()?;
            let ciphersuite = nym_lp::Ciphersuite::from_node_version(
                semver::Version::parse(value.gateway.version.as_ref()?).ok()?,
            )?;

            Some(NymNodeLPInformation {
                address: SocketAddr::new(ip_address, data.content.control_port),
                expected_kem_key_hashes: kem_keys,
                x25519: data.content.x25519,
                ciphersuite,
                // \/ TODO: proper derivation from build version
                lp_protocol_version: 1, // From @JS : for now just hardcode it to 1, we'll update it later (famous last words)
            })
        });
        Ok(Self {
            node: NymNodeInformation {
                identity: value.gateway.identity,
                ipr_address: value.gateway.ipr_address.map(Into::into),
                authenticator_address: value.gateway.authenticator_address.map(Into::into),
                ip_address,
                version: value.gateway.version.clone().into(),
                lp_data,
            },
            keys: value.keys.clone(),
        })
    }
}

impl std::fmt::Debug for GatewayWithKeys {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GatewayWithKeys")
            .field("gateway", &self.gateway)
            .field("client_wireguard_public_key", &self.keys.public_key())
            .finish()
    }
}

#[derive(Debug, Clone)]
pub struct SelectedGateways {
    entry: Box<GatewayWithKeys>,
    exit: Box<GatewayWithKeys>,
}

impl SelectedGateways {
    pub fn entry_gateway(&self) -> &Gateway {
        &self.entry.gateway
    }

    pub fn exit_gateway(&self) -> &Gateway {
        &self.exit.gateway
    }

    pub fn entry_keypair(&self) -> &Arc<KeyPair> {
        &self.entry.keys
    }

    pub fn exit_keypair(&self) -> &Arc<KeyPair> {
        &self.exit.keys
    }

    pub fn entry(&self) -> &GatewayWithKeys {
        &self.entry
    }

    pub fn exit(&self) -> &GatewayWithKeys {
        &self.exit
    }
}

pub enum OrderingCriteria<T> {
    Random(T),
    ClosestTo(Location),
}

fn find_best_entry_gateway(
    entry_gateways: &GatewayList,
    selection_criteria: OrderingCriteria<EntryPoint>,
    entry_filters: &GatewayFilters,
) -> Result<Gateway, nym_gateway_directory::Error> {
    match selection_criteria {
        OrderingCriteria::Random(entry_point) => {
            entry_gateways.find_best_entry_point_gateway(&entry_point, entry_filters)
        }
        OrderingCriteria::ClosestTo(geo_location) => entry_gateways
            .find_best_ordering_criteria_gateway(
                |gw1, gw2| closest_gateway(&geo_location, gw1, gw2),
                entry_filters,
            ),
    }
}

fn find_best_exit_gateway(
    exit_gateways: &GatewayList,
    selection_criteria: OrderingCriteria<ExitPoint>,
    exit_filters: &GatewayFilters,
) -> Result<Gateway, nym_gateway_directory::Error> {
    match selection_criteria {
        OrderingCriteria::Random(exit_point) => {
            exit_gateways.find_best_exit_point_gateway(&exit_point, exit_filters)
        }
        OrderingCriteria::ClosestTo(geo_location) => exit_gateways
            .find_best_ordering_criteria_gateway(
                |gw1, gw2| closest_gateway(&geo_location, gw1, gw2),
                exit_filters,
            ),
    }
}

pub async fn select_gateways(
    gateway_cache: impl GatewayCache,
    blacklisted_entry_gateways: &BlacklistedGateways,
    tunnel_settings: &TunnelSettings,
    device_location: Option<Location>,
    wg_keys_db: WireguardKeysDb,
) -> Result<SelectedGateways, GatewayDirectoryError> {
    // The set of exit gateways is smaller than the set of entry gateways, so we start by selecting
    // the exit gateway and then filter out the exit gateway from the set of entry gateways.

    let entry_point = EntryPoint::from(*tunnel_settings.entry_point.clone());
    let exit_point = ExitPoint::from(*tunnel_settings.exit_point.clone());

    if let (
        EntryPoint::Gateway {
            identity: entry_identity,
        },
        ExitPoint::Gateway {
            identity: exit_identity,
        },
    ) = (&entry_point, &exit_point)
        && entry_identity == exit_identity
    {
        return Err(GatewayDirectoryError::SameEntryAndExitGateway {
            identity: entry_identity.to_string(),
        });
    };

    let (mut entry_gateways, mut exit_gateways) = match tunnel_settings.tunnel_type {
        TunnelType::Wireguard => {
            let all_gateways = gateway_cache
                .lookup_gateways(GatewayType::Wg)
                .await
                .map_err(GatewayDirectoryError::LookupGateways)?;

            let entry_gateways = if tunnel_settings.bridges_enabled() {
                GatewayList::new(
                    all_gateways.gw_type(),
                    all_gateways
                        .clone()
                        .into_iter()
                        .filter(|gw| gw.bridge_params.is_some())
                        .collect(),
                )
            } else {
                all_gateways.clone()
            };

            (entry_gateways, all_gateways)
        }
        TunnelType::Mixnet => {
            // Setup the gateway that we will use as the exit point
            let exit_gateways = gateway_cache
                .lookup_gateways(GatewayType::MixnetExit)
                .await
                .map_err(GatewayDirectoryError::LookupGateways)?;
            // Setup the gateway that we will use as the entry point
            let entry_gateways = gateway_cache
                .lookup_gateways(GatewayType::MixnetEntry)
                .await
                .map_err(GatewayDirectoryError::LookupGateways)?;
            (entry_gateways, exit_gateways)
        }
    };

    let entry_filters = if blacklisted_entry_gateways.is_empty().unwrap_or(true) {
        GatewayFilters::default()
    } else {
        GatewayFilters::from(&[GatewayFilter::NotBlacklisted(
            blacklisted_entry_gateways.clone(),
        )])
    };

    let gateway_selection_algorithm = tunnel_settings
        .gateway_selection_algorithm_config
        .gateway_selection_algorithm;

    let entry_ordering_criteria = match (device_location.clone(), gateway_selection_algorithm) {
        (_, GatewaySelectionAlgorithm::Explicit) | (None, _) => {
            OrderingCriteria::Random(entry_point)
        }
        (Some(device_location), GatewaySelectionAlgorithm::AutoEntryExplicitExit)
        | (Some(device_location), GatewaySelectionAlgorithm::Auto) => {
            // Remove same jurisdiction as device from entry gateways
            entry_gateways.retain_gateways_by(|gateway| {
                gateway
                    .location
                    .as_ref()
                    .is_some_and(|entry_gateway_location| {
                        !same_jurisdiction(entry_gateway_location, &device_location)
                    })
            });
            OrderingCriteria::ClosestTo(device_location)
        }
    };

    let entry_gateway =
        find_best_entry_gateway(&entry_gateways, entry_ordering_criteria, &entry_filters)
            .map_err(GatewayDirectoryError::EntryGatewayUnavailable)?;

    // Exclude the entry gateway from the list of exit gateways for privacy reasons
    exit_gateways.retain_gateways_by(|gateway| gateway.identity() != entry_gateway.identity());

    let exit_ordering_criteria = match (device_location, gateway_selection_algorithm) {
        (_, GatewaySelectionAlgorithm::Explicit)
        | (_, GatewaySelectionAlgorithm::AutoEntryExplicitExit)
        | (None, _) => OrderingCriteria::Random(exit_point),
        (Some(device_location), GatewaySelectionAlgorithm::Auto) => {
            if let Some(entry_gateway_location) = entry_gateway.location.clone() {
                // Remove same jurisdiction as device and as entry gateway from exit gateways
                exit_gateways.retain_gateways_by(|gateway| {
                    gateway
                        .location
                        .as_ref()
                        .is_some_and(|exit_gateway_location| {
                            !same_jurisdiction(exit_gateway_location, &device_location)
                                && !same_jurisdiction(
                                    exit_gateway_location,
                                    &entry_gateway_location,
                                )
                        })
                });
                OrderingCriteria::ClosestTo(entry_gateway_location)
            } else {
                tracing::error!(
                    "The selected entry gateway should have a specified location, falling back to the explicit exit point: {exit_point}"
                );
                OrderingCriteria::Random(exit_point)
            }
        }
    };

    let exit_filters = if tunnel_settings.residential_exit {
        GatewayFilters::from(&[GatewayFilter::Residential, GatewayFilter::Exit])
    } else {
        GatewayFilters::default()
    };

    let exit_gateway =
        find_best_exit_gateway(&exit_gateways, exit_ordering_criteria, &exit_filters)
            .map_err(GatewayDirectoryError::ExitGatewayUnavailable)?;

    let entry_keys = wg_keys_db
        .load_or_create_keys(&entry_gateway.identity().to_string())
        .await
        .map_err(|source| GatewayDirectoryError::LoadKeypair {
            identity: entry_gateway.identity().to_string(),
            source,
        })?
        .entry_keypair()
        .clone();
    let exit_keys = wg_keys_db
        .load_or_create_keys(&exit_gateway.identity().to_string())
        .await
        .map_err(|source| GatewayDirectoryError::LoadKeypair {
            identity: exit_gateway.identity().to_string(),
            source,
        })?
        .exit_keypair()
        .clone();

    Ok(SelectedGateways {
        entry: Box::new(GatewayWithKeys {
            gateway: entry_gateway,
            keys: entry_keys,
        }),
        exit: Box::new(GatewayWithKeys {
            gateway: exit_gateway,
            keys: exit_keys,
        }),
    })
}
