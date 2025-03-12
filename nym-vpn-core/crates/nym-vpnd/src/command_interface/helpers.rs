// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_api_client::types::Percent;
use nym_vpn_lib::{
    gateway_directory::{EntryPoint, ExitPoint},
    NodeIdentity, Recipient,
};

// For the future: these functions should be moved to the nym-vpn-proto crate

pub(super) fn parse_entry_point(
    entry: nym_vpn_proto::entry_node::EntryNodeEnum,
) -> Result<EntryPoint, tonic::Status> {
    Ok(match entry {
        nym_vpn_proto::entry_node::EntryNodeEnum::Location(location) => {
            tracing::debug!(
                "Connecting to entry node in country: {:?}",
                location.two_letter_iso_country_code
            );
            EntryPoint::Location {
                location: location.two_letter_iso_country_code.to_string(),
            }
        }
        nym_vpn_proto::entry_node::EntryNodeEnum::Gateway(gateway) => {
            tracing::debug!("Connecting to entry node with gateway id: {:?}", gateway.id);
            let identity = NodeIdentity::from_base58_string(&gateway.id).map_err(|err| {
                tracing::error!("Failed to parse gateway id: {:?}", err);
                tonic::Status::invalid_argument("Invalid gateway id")
            })?;
            EntryPoint::Gateway { identity }
        }
        nym_vpn_proto::entry_node::EntryNodeEnum::RandomLowLatency(_) => {
            tracing::debug!("Connecting to low latency entry node");
            EntryPoint::RandomLowLatency
        }
        nym_vpn_proto::entry_node::EntryNodeEnum::Random(_) => {
            tracing::debug!("Connecting to random entry node");
            EntryPoint::Random
        }
    })
}

pub(super) fn parse_exit_point(
    exit: nym_vpn_proto::exit_node::ExitNodeEnum,
) -> Result<ExitPoint, tonic::Status> {
    Ok(match exit {
        nym_vpn_proto::exit_node::ExitNodeEnum::Address(address) => {
            tracing::debug!(
                "Connecting to exit node at address: {:?}",
                address.nym_address
            );
            let address =
                Recipient::try_from_base58_string(address.nym_address.clone()).map_err(|err| {
                    tracing::error!("Failed to parse exit node address: {:?}", err);
                    tonic::Status::invalid_argument("Invalid exit node address")
                })?;
            ExitPoint::Address {
                address: Box::new(address),
            }
        }
        nym_vpn_proto::exit_node::ExitNodeEnum::Gateway(gateway) => {
            tracing::debug!("Connecting to exit node with gateway id: {:?}", gateway.id);
            let identity = NodeIdentity::from_base58_string(&gateway.id).map_err(|err| {
                tracing::error!("Failed to parse gateway id: {:?}", err);
                tonic::Status::invalid_argument("Invalid gateway id")
            })?;
            ExitPoint::Gateway { identity }
        }
        nym_vpn_proto::exit_node::ExitNodeEnum::Location(location) => {
            tracing::debug!(
                "Connecting to exit node in country: {:?}",
                location.two_letter_iso_country_code
            );
            ExitPoint::Location {
                location: location.two_letter_iso_country_code.to_string(),
            }
        }
        nym_vpn_proto::exit_node::ExitNodeEnum::Random(_) => {
            tracing::debug!("Connecting to random exit node");
            ExitPoint::Random
        }
    })
}

pub(super) fn threshold_into_percent(threshold: nym_vpn_proto::Threshold) -> Percent {
    Percent::from_percentage_value(threshold.min_performance.clamp(0, 100) as u64).unwrap()
}
