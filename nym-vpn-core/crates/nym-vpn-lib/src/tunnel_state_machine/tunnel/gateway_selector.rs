// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_gateway_directory::{EntryPoint, ExitPoint, Gateway, GatewayCacheHandle, GatewayType};
use nym_vpn_api_client::types::ScoreThresholds;

use crate::{GatewayDirectoryError, tunnel_state_machine::TunnelType};

// First gateways with performance >= 75% are selected
// Performance threshold expressed as percentage from 0 to 100
const HIGH_PERFORMANCE_THRESHOLD: u8 = 75;

// Second, fallback to gateways with performance >= 50%
// Performance threshold expressed as percentage from 0 to 100
const MEDIUM_PERFORMANCE_THRESHOLD: u8 = 50;

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
    wg_score_thresholds: Option<ScoreThresholds>,
    mix_score_thresholds: Option<ScoreThresholds>,
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

    let (mut entry_gateways, exit_gateways) = match tunnel_type {
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

    tracing::info!("Found {} entry gateways", entry_gateways.len());
    tracing::info!("Found {} exit gateways", exit_gateways.len());

    let (min_wg_performance, min_mixnet_performance) =
        high_performance_tier(tunnel_type, wg_score_thresholds, mix_score_thresholds);
    let exit_gateway = exit_point
        .lookup_gateway(&exit_gateways, min_wg_performance, min_mixnet_performance)
        .or_else(|err| {
            // When no gateways could be found, lower performance tier and try again
            if err.is_no_matching_gateway_for_non_specific_gateway() {
                let (min_wg_performance, min_mixnet_performance) = medium_performance_tier(tunnel_type, wg_score_thresholds, mix_score_thresholds);
                tracing::debug!("Could not locate high quality exit gateway. Lowering performance filter to medium and trying again");

                exit_point.lookup_gateway(
                    &exit_gateways,
                    min_wg_performance,
                    min_mixnet_performance,
                ).map_err(GatewayDirectoryError::PerformantExitGatewayUnavailable)
            } else {
                Err(GatewayDirectoryError::SelectExitGateway(err))
            }
        })?;

    // Exclude the exit gateway from the list of entry gateways for privacy reasons
    entry_gateways.remove_gateway(&exit_gateway);

    // If there are no more entry gateways left, it means that entry and exit match.
    if entry_gateways.is_empty() {
        return Err(GatewayDirectoryError::SameEntryAndExitGateway {
            identity: exit_gateway.identity.to_string(),
        });
    }

    let entry_gateway = entry_point
        .lookup_gateway(&entry_gateways, min_wg_performance, min_mixnet_performance)
        .or_else(|err| {
            // When no gateways could be found, lower performance tier and try again
            if err.is_no_matching_gateway_for_non_specific_gateway() {
                let (min_wg_performance, min_mixnet_performance) = medium_performance_tier(tunnel_type, wg_score_thresholds, mix_score_thresholds);
                tracing::debug!("Could not locate high quality entry gateway. Lowering performance filter to medium and trying again");

                entry_point.lookup_gateway(
                    &entry_gateways,
                    min_wg_performance,
                    min_mixnet_performance,
                ).map_err(GatewayDirectoryError::PerformantEntryGatewayUnavailable)
            } else {
                 Err(GatewayDirectoryError::SelectEntryGateway(err))
            }
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

/// Returns minimum wireguard and mixnet performance thresholds for gateways with high performance.
fn high_performance_tier(
    tunnel_type: TunnelType,
    wg_score_thresholds: Option<ScoreThresholds>,
    mix_score_thresholds: Option<ScoreThresholds>,
) -> (Option<u8>, Option<u8>) {
    match tunnel_type {
        TunnelType::Wireguard => (
            Some(
                wg_score_thresholds
                    .map(|v| v.high)
                    .unwrap_or(HIGH_PERFORMANCE_THRESHOLD),
            ),
            None,
        ),
        TunnelType::Mixnet => (
            None,
            Some(
                mix_score_thresholds
                    .map(|v| v.high)
                    .unwrap_or(HIGH_PERFORMANCE_THRESHOLD),
            ),
        ),
    }
}

/// Returns minimum wireguard and mixnet performance thresholds for gateways with medium performance.
fn medium_performance_tier(
    tunnel_type: TunnelType,
    wg_score_thresholds: Option<ScoreThresholds>,
    mix_score_thresholds: Option<ScoreThresholds>,
) -> (Option<u8>, Option<u8>) {
    match tunnel_type {
        TunnelType::Wireguard => (
            Some(
                wg_score_thresholds
                    .map(|v| v.medium)
                    .unwrap_or(MEDIUM_PERFORMANCE_THRESHOLD),
            ),
            None,
        ),
        TunnelType::Mixnet => (
            None,
            Some(
                mix_score_thresholds
                    .map(|v| v.medium)
                    .unwrap_or(MEDIUM_PERFORMANCE_THRESHOLD),
            ),
        ),
    }
}
