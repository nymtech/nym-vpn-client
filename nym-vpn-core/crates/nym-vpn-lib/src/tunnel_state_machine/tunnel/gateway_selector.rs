// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_gateway_directory::{Gateway, GatewayCacheHandle, GatewayList};
use nym_vpn_api_client::types::ScoreThresholds;
use nym_vpn_lib_types::{EntryPoint, ExitPoint};

use crate::{
    GatewayDirectoryError,
    tunnel_state_machine::{TunnelSettings, TunnelType},
};

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
    tunnel_settings: &TunnelSettings,
    wg_score_thresholds: Option<ScoreThresholds>,
    mix_score_thresholds: Option<ScoreThresholds>,
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
                .lookup_gateways(nym_gateway_directory::GatewayType::Wg)
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
                .lookup_gateways(nym_gateway_directory::GatewayType::MixnetExit)
                .await
                .map_err(GatewayDirectoryError::LookupGateways)?;
            // Setup the gateway that we will use as the entry point
            let entry_gateways = gateway_cache_handle
                .lookup_gateways(nym_gateway_directory::GatewayType::MixnetEntry)
                .await
                .map_err(GatewayDirectoryError::LookupGateways)?;
            (entry_gateways, exit_gateways)
        }
    };

    tracing::info!("Found {} entry gateways", entry_gateways.len());
    tracing::info!("Found {} exit gateways", exit_gateways.len());

    let (min_wg_performance, min_mixnet_performance) = high_performance_tier(
        tunnel_settings.tunnel_type,
        wg_score_thresholds,
        mix_score_thresholds,
    );
    let exit_gateway = nym_gateway_directory::ExitPoint::from(exit_point.clone())
        .lookup_gateway(&exit_gateways, min_wg_performance, min_mixnet_performance, tunnel_settings.residential_exit)
        .or_else(|err| {
            // When no gateways could be found, lower performance tier and try again
            if err.is_unmatched_non_specific_gateway() {
                let (min_wg_performance, min_mixnet_performance) = medium_performance_tier(tunnel_settings.tunnel_type, wg_score_thresholds, mix_score_thresholds);
                tracing::debug!("Could not locate high quality exit gateway. Lowering performance filter to medium and trying again");

                nym_gateway_directory::ExitPoint::from(exit_point.clone()).lookup_gateway(
                    &exit_gateways,
                    min_wg_performance,
                    min_mixnet_performance,
                    tunnel_settings.residential_exit
                ).map_err(GatewayDirectoryError::PerformantExitGatewayUnavailable)
            } else {
                Err(GatewayDirectoryError::SelectExitGateway(err))
            }
        })?;

    // Exclude the exit gateway from the list of entry gateways for privacy reasons
    entry_gateways.remove_gateway(&exit_gateway);

    let entry_gateway = nym_gateway_directory::EntryPoint::from(entry_point.clone())
        .lookup_gateway(&entry_gateways, min_wg_performance, min_mixnet_performance)
        .or_else(|err| {
            // When no gateways could be found, lower performance tier and try again
            if err.is_unmatched_non_specific_gateway() {
                let (min_wg_performance, min_mixnet_performance) = medium_performance_tier(tunnel_settings.tunnel_type, wg_score_thresholds, mix_score_thresholds);
                tracing::debug!("Could not locate high quality entry gateway. Lowering performance filter to medium and trying again");

                nym_gateway_directory::EntryPoint::from(entry_point.clone()).lookup_gateway(
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
