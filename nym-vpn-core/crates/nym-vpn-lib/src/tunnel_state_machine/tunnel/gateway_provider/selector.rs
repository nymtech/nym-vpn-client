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
use nym_vpn_store::keys::wireguard::{WireguardKeyStore, WireguardKeysDb};

use crate::tunnel_state_machine::{
    TunnelSettings, TunnelType,
    tunnel::{
        self,
        gateway_provider::{
            error::GatewayProviderError,
            gateway_cache::GatewayCache,
            geo_ip::{closest_gateway, same_jurisdiction},
            independence::gateways_are_independent,
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
                GatewayProviderError::MalformedGateway(
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

#[derive(Debug)]
pub enum OrderingCriteria<T> {
    Random(T),
    ClosestTo(Location),
}

impl OrderingCriteria<EntryPoint> {
    pub fn new_entry(
        entry_gateways: &mut GatewayList,
        entry_point: nym_vpn_lib_types::EntryPoint,
        device_location: Option<&Location>,
    ) -> Result<Self, GatewayProviderError> {
        match entry_point {
            nym_vpn_lib_types::EntryPoint::Gateway { identity } => {
                Ok(OrderingCriteria::Random(EntryPoint::Gateway {
                    identity: *identity.inner(),
                }))
            }
            nym_vpn_lib_types::EntryPoint::Country {
                two_letter_iso_country_code,
            } => Ok(OrderingCriteria::Random(EntryPoint::Country {
                two_letter_iso_country_code,
            })),
            nym_vpn_lib_types::EntryPoint::Region { region } => {
                Ok(OrderingCriteria::Random(EntryPoint::Region { region }))
            }
            nym_vpn_lib_types::EntryPoint::Random => {
                Ok(OrderingCriteria::Random(EntryPoint::Random))
            }
            nym_vpn_lib_types::EntryPoint::Auto {
                exclude_user_country,
            } => {
                if let Some(device_location) = device_location {
                    if exclude_user_country {
                        // Remove same jurisdiction as device from entry gateways
                        entry_gateways.retain_gateways_by(|gateway| {
                            gateway
                                .location
                                .as_ref()
                                .is_some_and(|entry_gateway_location| {
                                    !same_jurisdiction(entry_gateway_location, device_location)
                                })
                        });
                    }
                    Ok(OrderingCriteria::ClosestTo(device_location.clone()))
                } else {
                    Err(GatewayProviderError::NeedsDeviceLocation)
                }
            }
        }
    }
}

impl OrderingCriteria<ExitPoint> {
    pub fn new_exit(
        entry_gateway_location: Option<Location>,
        exit_gateways: &mut GatewayList,
        exit_point: nym_vpn_lib_types::ExitPoint,
        device_location: Option<&Location>,
    ) -> Result<Self, GatewayProviderError> {
        match exit_point {
            nym_vpn_lib_types::ExitPoint::Gateway { identity } => {
                Ok(OrderingCriteria::Random(ExitPoint::Gateway {
                    identity: *identity.inner(),
                }))
            }
            nym_vpn_lib_types::ExitPoint::Address { address } => {
                Ok(OrderingCriteria::Random(ExitPoint::Address {
                    address: Box::new(nym_gateway_directory::Recipient::from(*address)),
                }))
            }
            nym_vpn_lib_types::ExitPoint::Country {
                two_letter_iso_country_code,
            } => Ok(OrderingCriteria::Random(ExitPoint::Country {
                two_letter_iso_country_code,
            })),
            nym_vpn_lib_types::ExitPoint::Region { region } => {
                Ok(OrderingCriteria::Random(ExitPoint::Region { region }))
            }
            nym_vpn_lib_types::ExitPoint::Random => Ok(OrderingCriteria::Random(ExitPoint::Random)),
            nym_vpn_lib_types::ExitPoint::Auto {
                exclude_entry_point_country,
                exclude_user_country,
            } => {
                let fallback_criteria = if let Some(device_location) = device_location
                    && exclude_user_country
                {
                    // Remove same jurisdiction as device from exit gateways
                    exit_gateways.retain_gateways_by(|gateway| {
                        gateway
                            .location
                            .as_ref()
                            .is_some_and(|exit_gateway_location| {
                                !same_jurisdiction(exit_gateway_location, device_location)
                            })
                    });
                    // save it, just in case entry gateway doesn't have a location, we can use the device location instead
                    Some(OrderingCriteria::ClosestTo(device_location.clone()))
                } else {
                    None
                };
                if let Some(entry_gateway_location) = entry_gateway_location {
                    if exclude_entry_point_country {
                        // Remove same jurisdiction as entry gateway from exit gateways
                        exit_gateways.retain_gateways_by(|gateway| {
                            gateway
                                .location
                                .as_ref()
                                .is_some_and(|exit_gateway_location| {
                                    !same_jurisdiction(
                                        exit_gateway_location,
                                        &entry_gateway_location,
                                    )
                                })
                        });
                    }
                    Ok(OrderingCriteria::ClosestTo(entry_gateway_location))
                } else if let Some(criteria) = fallback_criteria {
                    Ok(criteria)
                } else {
                    Err(GatewayProviderError::NeedsDeviceLocation)
                }
            }
        }
    }
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

fn select_entry(
    mut entry_gateways: GatewayList,
    blacklisted_gateways: &BlacklistedGateways,
    tunnel_settings: &TunnelSettings,
    device_location: Option<&Location>,
) -> Result<Gateway, GatewayProviderError> {
    let entry_filters = if blacklisted_gateways.is_empty().unwrap_or(true) {
        GatewayFilters::default()
    } else {
        GatewayFilters::from(&[GatewayFilter::NotBlacklisted(blacklisted_gateways.clone())])
    };

    let entry_ordering_criteria = OrderingCriteria::new_entry(
        &mut entry_gateways,
        tunnel_settings.entry_point.as_ref().clone(),
        device_location,
    )?;

    find_best_entry_gateway(&entry_gateways, entry_ordering_criteria, &entry_filters)
        .map_err(GatewayProviderError::EntryGatewayUnavailable)
}

fn select_exit(
    entry_gateway: &Gateway,
    mut exit_gateways: GatewayList,
    blacklisted_gateways: &BlacklistedGateways,
    tunnel_settings: &TunnelSettings,
    device_location: Option<&Location>,
) -> Result<Gateway, GatewayProviderError> {
    // Exclude the entry gateway from the list of exit gateways for privacy reasons
    exit_gateways.retain_gateways_by(|exit_gateway| {
        gateways_are_independent(
            entry_gateway,
            exit_gateway,
            tunnel_settings.gateway_independence,
        )
    });

    let exit_ordering_criteria = OrderingCriteria::new_exit(
        entry_gateway.location.clone(),
        &mut exit_gateways,
        tunnel_settings.exit_point.as_ref().clone(),
        device_location,
    )?;

    let mut exit_filter_items: Vec<GatewayFilter> = Vec::new();
    if tunnel_settings.residential_exit {
        exit_filter_items.push(GatewayFilter::Exit);
        exit_filter_items.push(GatewayFilter::Residential);
    }
    if !blacklisted_gateways.is_empty().unwrap_or(true) {
        exit_filter_items.push(GatewayFilter::NotBlacklisted(blacklisted_gateways.clone()));
    }
    let exit_filters = GatewayFilters::from(&exit_filter_items);

    find_best_exit_gateway(&exit_gateways, exit_ordering_criteria, &exit_filters)
        .map_err(GatewayProviderError::ExitGatewayUnavailable)
}

fn loop_select(
    mut entry_gateways: GatewayList,
    exit_gateways: GatewayList,
    blacklisted_gateways: &BlacklistedGateways,
    tunnel_settings: &TunnelSettings,
    device_location: Option<&Location>,
) -> Result<(Gateway, Gateway), GatewayProviderError> {
    let mut exit_error = None;
    loop {
        let entry_gateway = select_entry(
            entry_gateways.clone(),
            blacklisted_gateways,
            tunnel_settings,
            device_location,
        )
        // if we failed previously on exit selection, we return that error
        // entry error is returned if there was no previous exit selection error
        .map_err(|entry_error| exit_error.unwrap_or(entry_error))?;
        match select_exit(
            &entry_gateway,
            exit_gateways.clone(),
            blacklisted_gateways,
            tunnel_settings,
            device_location,
        ) {
            Ok(exit_gateway) => return Ok((entry_gateway, exit_gateway)),
            Err(err) => {
                exit_error = Some(err);
                entry_gateways
                    .retain_gateways_by(|gateway| gateway.identity() != entry_gateway.identity());
            }
        }
    }
}

pub async fn select_gateways(
    gateway_cache: impl GatewayCache,
    blacklisted_gateways: &BlacklistedGateways,
    tunnel_settings: &TunnelSettings,
    device_location: Option<Location>,
    wg_keys_db: &WireguardKeysDb,
) -> Result<SelectedGateways, GatewayProviderError> {
    // The set of exit gateways is smaller than the set of entry gateways, so we start by selecting
    // the exit gateway and then filter out the exit gateway from the set of entry gateways.

    if let (
        nym_vpn_lib_types::EntryPoint::Gateway {
            identity: entry_identity,
        },
        nym_vpn_lib_types::ExitPoint::Gateway {
            identity: exit_identity,
        },
    ) = (
        tunnel_settings.entry_point.as_ref(),
        tunnel_settings.exit_point.as_ref(),
    ) && *entry_identity == *exit_identity
    {
        return Err(GatewayProviderError::SameEntryAndExitGateway {
            identity: entry_identity.to_string(),
        });
    };

    let (entry_gateways, exit_gateways) = match tunnel_settings.tunnel_type_used() {
        TunnelType::Wireguard => {
            let all_gateways = gateway_cache
                .lookup_gateways(GatewayType::Wg)
                .await
                .map_err(GatewayProviderError::LookupGateways)?;

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
                .map_err(GatewayProviderError::LookupGateways)?;
            // Setup the gateway that we will use as the entry point
            let entry_gateways = gateway_cache
                .lookup_gateways(GatewayType::MixnetEntry)
                .await
                .map_err(GatewayProviderError::LookupGateways)?;
            (entry_gateways, exit_gateways)
        }
    };

    let (entry_gateway, exit_gateway) = if tunnel_settings.gateway_independence.active() {
        // Try with gateway independent gateways first
        if let Ok(pair) = loop_select(
            entry_gateways.clone(),
            exit_gateways.clone(),
            blacklisted_gateways,
            tunnel_settings,
            device_location.as_ref(),
        ) {
            pair
        } else {
            // Check if removing the independence criteria allows us to select a gateway pair
            // so we know what to tell the user.
            let mut no_gateway_independence_settings = tunnel_settings.clone();
            no_gateway_independence_settings
                .gateway_independence
                .set_enabled(false);
            // if we still can't select, we just return the error
            loop_select(
                entry_gateways,
                exit_gateways,
                blacklisted_gateways,
                &no_gateway_independence_settings,
                device_location.as_ref(),
            )?;
            // otherwise we return an error that prompts the user to explicitly agree to possible non-independent gateways
            return Err(GatewayProviderError::NeedsRelaxedIndependenceCriteria);
        }
    } else {
        let entry_gateway = select_entry(
            entry_gateways,
            blacklisted_gateways,
            tunnel_settings,
            device_location.as_ref(),
        )?;
        let exit_gateway = select_exit(
            &entry_gateway,
            exit_gateways,
            blacklisted_gateways,
            tunnel_settings,
            device_location.as_ref(),
        )?;
        (entry_gateway, exit_gateway)
    };

    let entry_keys = wg_keys_db
        .load_or_create_keys(&entry_gateway.identity().to_string())
        .await
        .map_err(|source| GatewayProviderError::LoadKeypair {
            identity: entry_gateway.identity().to_string(),
            source,
        })?
        .entry_keypair()
        .clone();
    let exit_keys = wg_keys_db
        .load_or_create_keys(&exit_gateway.identity().to_string())
        .await
        .map_err(|source| GatewayProviderError::LoadKeypair {
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

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use nym_gateway_directory::{
        Asn, AsnKind, BlacklistedGateways, Location, Performance, ScoreValue,
    };
    use nym_vpn_lib_types::{EntryPoint, ExitPoint, GatewayIndependence};
    use nym_vpn_store::keys::wireguard::WireguardKeysDb;
    use tokio::sync::RwLock;

    use crate::tunnel_state_machine::tunnel::gateway_provider::{
        error::GatewayProviderError,
        gateway_cache::tests::MockGatewayCache,
        tests::{default_tunnel_settings, gateway_id_to_gateway},
    };

    use super::*;

    const GW_ID_1: &str = "2zHiExNRKiCXVKS35SNKtK4apGfZELMpA1jJ2gVevJoz";
    const GW_ID_2: &str = "38zcSsvjXsAX7C28ko2H3Lt55X4TYxfZYkPADxKXZHUj";

    fn make_gw_with_asn(id: &str, asn: Asn) -> Gateway {
        Gateway::builder()
            .identity(id.parse().unwrap())
            .location(Location {
                asn: Some(asn),
                ..Default::default()
            })
            .performance(Performance {
                last_updated_utc: Default::default(),
                score: ScoreValue::High,
                mixnet_score: ScoreValue::High,
                load: ScoreValue::Low,
                uptime_percentage_last_24_hours: Default::default(),
            })
            .build()
    }

    #[tokio::test]
    async fn same_entry_and_exit_gateway_identity_returns_error() {
        let gateways = Arc::new(RwLock::new(Some(vec![
            gateway_id_to_gateway(GW_ID_1),
            gateway_id_to_gateway(GW_ID_2),
        ])));
        let gateway_cache = MockGatewayCache::new(gateways);

        let identity: nym_vpn_lib_types::NodeIdentity = GW_ID_1.parse().unwrap();
        let mut settings = default_tunnel_settings();
        *settings.entry_point = EntryPoint::Gateway {
            identity: identity.clone(),
        };
        *settings.exit_point = ExitPoint::Gateway { identity };

        let result = select_gateways(
            gateway_cache,
            &BlacklistedGateways::new(),
            &settings,
            None,
            &WireguardKeysDb::Ephemeral(Default::default()),
        )
        .await;

        assert!(matches!(
            result,
            Err(GatewayProviderError::SameEntryAndExitGateway { .. })
        ));
    }

    #[tokio::test]
    async fn active_independence_criteria_triggers_needs_relaxed_error() {
        // Both gateways share the same ASN so no independent pair can be found with the default
        // (fully active) independence criteria. Without any criteria they CAN be paired, so the
        // selector should suggest relaxing the criteria.
        let gateways = Arc::new(RwLock::new(Some(vec![
            make_gw_with_asn(
                GW_ID_1,
                Asn {
                    asn: "AS100".to_string(),
                    name: "ISP".to_string(),
                    route: "10.10.10.10/16".parse().unwrap(),
                    kind: AsnKind::Other,
                },
            ),
            make_gw_with_asn(
                GW_ID_2,
                Asn {
                    asn: "AS100".to_string(),
                    name: "ISP".to_string(),
                    route: "10.11.10.10/16".parse().unwrap(),
                    kind: AsnKind::Other,
                },
            ),
        ])));
        let gateway_cache = MockGatewayCache::new(gateways);

        let mut settings = default_tunnel_settings();
        settings.gateway_independence = GatewayIndependence::default();

        let result = select_gateways(
            gateway_cache,
            &BlacklistedGateways::new(),
            &settings,
            None,
            &WireguardKeysDb::Ephemeral(Default::default()),
        )
        .await;

        assert!(matches!(
            result,
            Err(GatewayProviderError::NeedsRelaxedIndependenceCriteria)
        ));
    }

    #[tokio::test]
    async fn gateways_with_different_asns_succeed_with_full_independence_criteria() {
        // Gateways have different ASNs and no node family, so the full independence criteria
        // is satisfied (missing family is treated as independent).
        let gateways = Arc::new(RwLock::new(Some(vec![
            make_gw_with_asn(
                GW_ID_1,
                Asn {
                    asn: "AS100".to_string(),
                    name: "ISP".to_string(),
                    route: "10.10.10.10/16".parse().unwrap(),
                    kind: AsnKind::Other,
                },
            ),
            make_gw_with_asn(
                GW_ID_2,
                Asn {
                    asn: "AS200".to_string(),
                    name: "ISP".to_string(),
                    route: "10.11.10.10/16".parse().unwrap(),
                    kind: AsnKind::Other,
                },
            ),
        ])));
        let gateway_cache = MockGatewayCache::new(gateways);

        let mut settings = default_tunnel_settings();
        settings.gateway_independence = GatewayIndependence::default();

        let result = select_gateways(
            gateway_cache,
            &BlacklistedGateways::new(),
            &settings,
            None,
            &WireguardKeysDb::Ephemeral(Default::default()),
        )
        .await;

        assert!(result.is_ok());
    }
}
