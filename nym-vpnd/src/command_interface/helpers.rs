// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_lib::{
    gateway_directory::{EntryPoint, ExitPoint},
    NodeIdentity, Recipient,
};
use tracing::{error, info};

pub(super) fn parse_entry_point(
    entry: nym_vpn_proto::entry_node::EntryNodeEnum,
) -> Result<EntryPoint, tonic::Status> {
    Ok(match entry {
        nym_vpn_proto::entry_node::EntryNodeEnum::Location(location) => {
            info!(
                "Connecting to entry node in country: {:?}",
                location.two_letter_iso_country_code
            );
            EntryPoint::Location {
                location: location.two_letter_iso_country_code.to_string(),
            }
        }
        nym_vpn_proto::entry_node::EntryNodeEnum::Gateway(gateway) => {
            info!("Connecting to entry node with gateway id: {:?}", gateway.id);
            let identity = NodeIdentity::from_base58_string(&gateway.id).map_err(|err| {
                error!("Failed to parse gateway id: {:?}", err);
                tonic::Status::invalid_argument("Invalid gateway id")
            })?;
            EntryPoint::Gateway { identity }
        }
        nym_vpn_proto::entry_node::EntryNodeEnum::RandomLowLatency(_) => {
            info!("Connecting to low latency entry node");
            EntryPoint::RandomLowLatency
        }
        nym_vpn_proto::entry_node::EntryNodeEnum::Random(_) => {
            info!("Connecting to random entry node");
            EntryPoint::Random
        }
    })
}

pub(super) fn parse_exit_point(
    exit: nym_vpn_proto::exit_node::ExitNodeEnum,
) -> Result<ExitPoint, tonic::Status> {
    Ok(match exit {
        nym_vpn_proto::exit_node::ExitNodeEnum::Address(address) => {
            info!(
                "Connecting to exit node at address: {:?}",
                address.nym_address
            );
            let address =
                Recipient::try_from_base58_string(address.nym_address.clone()).map_err(|err| {
                    error!("Failed to parse exit node address: {:?}", err);
                    tonic::Status::invalid_argument("Invalid exit node address")
                })?;
            ExitPoint::Address { address }
        }
        nym_vpn_proto::exit_node::ExitNodeEnum::Gateway(gateway) => {
            info!("Connecting to exit node with gateway id: {:?}", gateway.id);
            let identity = NodeIdentity::from_base58_string(&gateway.id).map_err(|err| {
                error!("Failed to parse gateway id: {:?}", err);
                tonic::Status::invalid_argument("Invalid gateway id")
            })?;
            ExitPoint::Gateway { identity }
        }
        nym_vpn_proto::exit_node::ExitNodeEnum::Location(location) => {
            info!(
                "Connecting to exit node in country: {:?}",
                location.two_letter_iso_country_code
            );
            ExitPoint::Location {
                location: location.two_letter_iso_country_code.to_string(),
            }
        }
        nym_vpn_proto::exit_node::ExitNodeEnum::Random(_) => {
            info!("Connecting to low latency exit node");
            ExitPoint::Random
        }
    })
}
