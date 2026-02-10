// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{net::SocketAddr, sync::Arc};

use nym_crypto::asymmetric::x25519::KeyPair;
use nym_gateway_directory::{
    BlacklistedGateways, EntryPoint, ExitPoint, Gateway, GatewayCacheHandle, GatewayFilter,
    GatewayFilters, GatewayList, GatewayType,
};
use nym_registration_client::RegistrationNymNode;
use nym_registration_common::{NymNodeInformation, NymNodeLPInformation};
use nym_vpn_store::keys::wireguard::{WireguardKeyStore, WireguardKeysDb};

use crate::{
    GatewayDirectoryError,
    tunnel_state_machine::{TunnelSettings, TunnelType, tunnel},
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

        let lp_data = value.gateway.lp_information.and_then(|data| {
            let kem_keys = data.kem_keys().ok()?;
            let signing_keys = data.signing_keys().ok()?;

            Some(NymNodeLPInformation {
                address: SocketAddr::new(ip_address, data.control_port),
                expected_kem_key_hashes: kem_keys,
                expected_signing_key_hashes: signing_keys,
                x25519: data.x25519,
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

pub async fn select_gateways(
    gateway_cache_handle: GatewayCacheHandle,
    blacklisted_entry_gateways: &BlacklistedGateways,
    tunnel_settings: &TunnelSettings,
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

    let (mut entry_gateways, exit_gateways) = match tunnel_settings.tunnel_type {
        TunnelType::Wireguard => {
            let all_gateways = gateway_cache_handle
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

            // Temporary filtering of LP gateways, while `enable_lewes_protocol` exists
            if tunnel_settings.enable_lewes_protocol {
                let entry_lp_gateways = GatewayList::new(
                    entry_gateways.gw_type(),
                    entry_gateways
                        .clone()
                        .into_iter()
                        .filter(|gw| gw.lp_information.is_some())
                        .collect(),
                );
                let all_lp_gateways = GatewayList::new(
                    all_gateways.gw_type(),
                    all_gateways
                        .clone()
                        .into_iter()
                        .filter(|gw| gw.lp_information.is_some())
                        .collect(),
                );
                (entry_lp_gateways, all_lp_gateways)
            } else {
                (entry_gateways, all_gateways)
            }
        }
        TunnelType::Mixnet => {
            // Setup the gateway that we will use as the exit point
            let exit_gateways = gateway_cache_handle
                .lookup_gateways(GatewayType::MixnetExit)
                .await
                .map_err(GatewayDirectoryError::LookupGateways)?;
            // Setup the gateway that we will use as the entry point
            let entry_gateways = gateway_cache_handle
                .lookup_gateways(GatewayType::MixnetEntry)
                .await
                .map_err(GatewayDirectoryError::LookupGateways)?;
            (entry_gateways, exit_gateways)
        }
    };

    tracing::info!("Found {} entry gateways", entry_gateways.len());
    tracing::info!("Found {} exit gateways", exit_gateways.len());

    let exit_filters = if tunnel_settings.residential_exit {
        GatewayFilters::from(&[GatewayFilter::Residential, GatewayFilter::Exit])
    } else {
        GatewayFilters::default()
    };

    let exit_gateway = exit_gateways
        .find_best_exit_gateway(&exit_point, &exit_filters)
        .map_err(GatewayDirectoryError::ExitGatewayUnavailable)?;

    // Exclude the exit gateway from the list of entry gateways for privacy reasons
    entry_gateways.remove_gateway(&exit_gateway);

    let entry_filters = if blacklisted_entry_gateways.is_empty().unwrap_or(true) {
        GatewayFilters::default()
    } else {
        GatewayFilters::from(&[GatewayFilter::NotBlacklisted(
            blacklisted_entry_gateways.clone(),
        )])
    };

    let entry_gateway = entry_gateways
        .find_best_entry_gateway(&entry_point, &entry_filters)
        .map_err(GatewayDirectoryError::EntryGatewayUnavailable)?;

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

    tracing::debug!("Using entry public key: {}", entry_keys.public_key());
    tracing::debug!("Using exit public key: {}", exit_keys.public_key());

    tracing::info!(
        "Using entry gateway: {}, location: {}, performance: {}",
        entry_gateway.identity(),
        entry_gateway
            .two_letter_iso_country_code()
            .map_or_else(|| "unknown".to_string(), |code| code.to_string()),
        entry_gateway
            .mixnet_performance
            .map_or_else(|| "unknown".to_string(), |perf| perf.to_string()),
    );
    tracing::info!(
        "Using exit gateway: {}, location: {}, performance: {}",
        exit_gateway.identity(),
        exit_gateway
            .two_letter_iso_country_code()
            .map_or_else(|| "unknown".to_string(), |code| code.to_string()),
        exit_gateway
            .mixnet_performance
            .map_or_else(|| "unknown".to_string(), |perf| perf.to_string()),
    );
    tracing::info!(
        "Using exit router address {}",
        exit_gateway
            .ipr_address
            .map_or_else(|| "none".to_string(), |ipr| ipr.to_string())
    );

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
