// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_gateway_directory::GatewayClient;
use tracing::info;

use crate::SpecificVpn;

#[derive(thiserror::Error, Debug)]
pub enum SelectGatewaysError {
    #[error("failed to setup gateway directory client: {source}")]
    FailedtoSetupGatewayDirectoryClient {
        config: Box<nym_gateway_directory::Config>,
        source: nym_gateway_directory::Error,
    },

    #[error("failed to lookup gateways: {source}")]
    FailedToLookupGateways {
        source: nym_gateway_directory::Error,
    },

    #[error("failed to lookup gateway identity: {source}")]
    FailedToLookupGatewayIdentity {
        source: nym_gateway_directory::Error,
    },

    #[error("failed to select entry gateway: {source}")]
    FailedToSelectEntryGateway {
        source: nym_gateway_directory::Error,
    },

    #[error("failed to select exit gateway: {source}")]
    FailedToSelectExitGateway {
        source: nym_gateway_directory::Error,
    },

    #[error("failed to lookup router address: {source}")]
    FailedToLookupRouterAddress {
        source: nym_gateway_directory::Error,
    },

    #[error("unable to use same entry and exit gateway for location: {requested_location}")]
    SameEntryAndExitGatewayFromCountry { requested_location: String },
}

pub(super) struct SelectedGateways {
    pub(super) entry: nym_gateway_directory::Gateway,
    pub(super) exit: nym_gateway_directory::Gateway,
}

pub(super) async fn select_gateways(
    gateway_directory_client: &GatewayClient,
    nym_vpn: &SpecificVpn,
) -> std::result::Result<SelectedGateways, SelectGatewaysError> {
    // The set of exit gateways is smaller than the set of entry gateways, so we start by selecting
    // the exit gateway and then filter out the exit gateway from the set of entry gateways.

    let (mut entry_gateways, exit_gateways) = if let SpecificVpn::Mix(_) = nym_vpn {
        // Setup the gateway that we will use as the exit point
        let exit_gateways = gateway_directory_client
            .lookup_exit_gateways()
            .await
            .map_err(|source| SelectGatewaysError::FailedToLookupGateways { source })?;
        // Setup the gateway that we will use as the entry point
        let entry_gateways = gateway_directory_client
            .lookup_entry_gateways()
            .await
            .map_err(|source| SelectGatewaysError::FailedToLookupGateways { source })?;
        (entry_gateways, exit_gateways)
    } else {
        let all_gateways = gateway_directory_client
            .lookup_all_gateways()
            .await
            .map_err(|source| SelectGatewaysError::FailedToLookupGateways { source })?;
        (all_gateways.clone(), all_gateways)
    };

    let exit_gateway = nym_vpn
        .exit_point()
        .lookup_gateway(&exit_gateways)
        .map_err(|source| SelectGatewaysError::FailedToSelectExitGateway { source })?;

    // Exclude the exit gateway from the list of entry gateways for privacy reasons
    entry_gateways.remove_gateway(&exit_gateway);

    let entry_gateway = nym_vpn
        .entry_point()
        .lookup_gateway(&entry_gateways)
        .await
        .map_err(|source| match source {
            nym_gateway_directory::Error::NoMatchingEntryGatewayForLocation {
                requested_location,
                available_countries: _,
            } if Some(requested_location.as_str())
                == exit_gateway.two_letter_iso_country_code() =>
            {
                SelectGatewaysError::SameEntryAndExitGatewayFromCountry {
                    requested_location: requested_location.to_string(),
                }
            }
            _ => SelectGatewaysError::FailedToSelectEntryGateway { source },
        })?;

    info!("Found {} entry gateways", entry_gateways.len());
    info!("Found {} exit gateways", exit_gateways.len());
    info!(
        "Using entry gateway: {}, location: {}, performance: {}",
        *entry_gateway.identity(),
        entry_gateway
            .two_letter_iso_country_code()
            .map_or_else(|| "unknown".to_string(), |code| code.to_string()),
        entry_gateway
            .performance
            .map_or_else(|| "unknown".to_string(), |perf| perf.to_string()),
    );
    info!(
        "Using exit gateway: {}, location: {}, performance: {}",
        *exit_gateway.identity(),
        exit_gateway
            .two_letter_iso_country_code()
            .map_or_else(|| "unknown".to_string(), |code| code.to_string()),
        entry_gateway
            .performance
            .map_or_else(|| "unknown".to_string(), |perf| perf.to_string()),
    );
    info!(
        "Using exit router address {}",
        exit_gateway
            .ipr_address
            .map_or_else(|| "none".to_string(), |ipr| ipr.to_string())
    );

    Ok(SelectedGateways {
        entry: entry_gateway,
        exit: exit_gateway,
    })
}
