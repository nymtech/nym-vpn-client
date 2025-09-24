// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_gateway_directory::{EntryPoint, ExitPoint, Gateway, GatewayCacheHandle, GatewayType};

use crate::{GatewayDirectoryError, tunnel_state_machine::TunnelType};

// Performance threshold expressed as percentage from 0 to 100
// All gateways below this threshold will be removed
const MIN_PERFORMANCE_THRESHOLD: u8 = 75;

#[derive(Debug, Clone)]
pub struct SelectedGateways {
    pub entry: Box<Gateway>,
    pub exit: Box<Gateway>,
}

pub async fn select_gateways(
    gateway_cache_handle: GatewayCacheHandle,
    tunnel_type: TunnelType,
    entry_point: Box<EntryPoint>,
    exit_point: Box<ExitPoint>,
) -> Result<SelectedGateways, GatewayDirectoryError> {
    // The set of exit gateways is smaller than the set of entry gateways, so we start by selecting
    // the exit gateway and then filter out the exit gateway from the set of entry gateways.

    if let (
        EntryPoint::Gateway {
            identity: entry_identity,
        },
        ExitPoint::Gateway {
            identity: exit_identity,
        },
    ) = (entry_point.as_ref(), &exit_point.as_ref())
        && entry_identity == exit_identity
    {
        return Err(GatewayDirectoryError::SameEntryAndExitGateway {
            identity: entry_identity.to_string(),
        });
    };

    let (mut entry_gateways, mut exit_gateways) = match tunnel_type {
        TunnelType::Wireguard => {
            let all_gateways = gateway_cache_handle
                .lookup_gateways(GatewayType::Wg)
                .await
                .map_err(GatewayDirectoryError::LookupGateways)?;
            (all_gateways.clone(), all_gateways)
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

    let total_entry_gateways = entry_gateways.len();
    let total_exit_gateways = exit_gateways.len();

    // Gateways to exclude from low performance filtering
    let mut exclude_gateway_idents = vec![];

    // Exclude explicitly selected exit gateway from performance filtering
    if let ExitPoint::Gateway { identity } = exit_point.as_ref() {
        exclude_gateway_idents.push(identity.clone());
    } else if let ExitPoint::Address { address } = exit_point.as_ref() {
        exclude_gateway_idents.push(address.identity().clone());
    }

    // Exclude explicitly selected entry gateway from performance filtering
    if let EntryPoint::Gateway { identity } = entry_point.as_ref() {
        exclude_gateway_idents.push(identity.clone());
    }

    // Remove entry and exit gateways with performance below the min threshold
    entry_gateways.remove_gateways_with_performance_less_than(
        tunnel_type == TunnelType::Wireguard,
        MIN_PERFORMANCE_THRESHOLD,
        &exclude_gateway_idents,
    );
    exit_gateways.remove_gateways_with_performance_less_than(
        tunnel_type == TunnelType::Wireguard,
        MIN_PERFORMANCE_THRESHOLD,
        &exclude_gateway_idents,
    );

    tracing::info!(
        "Found {} entry gateways ({} with >={}% performance)",
        total_entry_gateways,
        entry_gateways.len(),
        MIN_PERFORMANCE_THRESHOLD
    );
    tracing::info!(
        "Found {} exit gateways ({} with >={}% performance)",
        total_exit_gateways,
        exit_gateways.len(),
        MIN_PERFORMANCE_THRESHOLD
    );

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
