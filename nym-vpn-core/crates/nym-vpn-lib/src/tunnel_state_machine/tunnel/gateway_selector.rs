// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_gateway_directory::{
    EntryPoint, ExitPoint, Gateway, GatewayCacheHandle, GatewayList, GatewayType,
};

use crate::{
    GatewayDirectoryError,
    tunnel_state_machine::{TunnelSettings, TunnelType},
};

#[derive(Debug, Clone)]
pub struct SelectedGateways {
    pub entry: Box<Gateway>,
    pub exit: Box<Gateway>,
}

pub async fn select_gateways(
    gateway_cache_handle: GatewayCacheHandle,
    tunnel_settings: &TunnelSettings,
) -> Result<SelectedGateways, GatewayDirectoryError> {
    // The set of exit gateways is smaller than the set of entry gateways, so we start by selecting
    // the exit gateway and then filter out the exit gateway from the set of entry gateways.

    let entry_point = tunnel_settings.entry_point.as_ref();
    let exit_point = tunnel_settings.exit_point.as_ref();
    if let (
        EntryPoint::Gateway {
            identity: entry_identity,
        },
        ExitPoint::Gateway {
            identity: exit_identity,
        },
    ) = (entry_point, &exit_point)
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

    let exit_gateway = exit_point
        .lookup_gateway(&exit_gateways)
        .map_err(GatewayDirectoryError::SelectExitGateway)?;

    // Exclude the exit gateway from the list of entry gateways for privacy reasons
    entry_gateways.remove_gateway(&exit_gateway);

    let entry_gateway = entry_point
        .lookup_gateway(&entry_gateways)
        .await
        .map_err(|source| match source {
            nym_gateway_directory::Error::NoMatchingEntryGatewayForLocation {
                requested_location,
                available_countries: _,
            } if Some(requested_location.as_str())
                == exit_gateway.two_letter_iso_country_code() =>
            {
                GatewayDirectoryError::SameEntryAndExitGateway {
                    identity: exit_gateway.identity.to_string(),
                }
            }
            _ => GatewayDirectoryError::SelectEntryGateway(source),
        })?;

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
        entry: Box::new(entry_gateway),
        exit: Box::new(exit_gateway),
    })
}
