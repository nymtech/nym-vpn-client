// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_explorer_client::Location;
use nym_validator_client::nyxd::bip32::secp256k1::pkcs8::der::DateTime;
use nym_validator_client::{client::IdentityKey, models::DescribedGateway};

const BUILD_VERSION: &str = "1.1.34";
const BUILD_TIME: &str = "2024-03-25T10:47:53.981548588Z";

#[derive(Clone, Debug)]
pub struct DescribedGatewayWithLocation {
    pub gateway: DescribedGateway,
    pub location: Option<Location>,
}

impl DescribedGatewayWithLocation {
    pub fn identity_key(&self) -> &IdentityKey {
        &self.gateway.bond.gateway.identity_key
    }

    pub fn has_ip_packet_router(&self) -> bool {
        self.gateway
            .self_described
            .as_ref()
            .and_then(|d| d.ip_packet_router.as_ref())
            .is_some()
    }

    pub fn has_current_api_version(&self) -> bool {
        self.has_v6_exit_build_timestamp() && self.has_current_build_version()
    }

    fn has_v6_exit_build_timestamp(&self) -> bool {
        self.gateway
            .self_described
            .as_ref()
            .map(|d| {
                d.build_information
                    .build_timestamp
                    .parse()
                    .map_or(false, |build_time| {
                        let expected_build_time: DateTime =
                            BUILD_TIME.parse().expect("Invalid timestamp");
                        build_time >= expected_build_time
                    })
            })
            .is_some()
    }

    //can make this more flexible with backwards compatibility
    fn has_current_build_version(&self) -> bool {
        self.gateway
            .self_described
            .as_ref()
            .map(|d| d.build_information.build_version == BUILD_VERSION)
            .is_some()
    }

    pub fn has_location(&self) -> bool {
        self.location.is_some()
    }

    pub fn location(&self) -> Option<Location> {
        self.location.clone()
    }

    pub fn two_letter_iso_country_code(&self) -> Option<String> {
        self.location
            .as_ref()
            .map(|l| l.two_letter_iso_country_code.clone())
    }

    pub fn is_two_letter_iso_country_code(&self, code: &str) -> bool {
        self.two_letter_iso_country_code()
            .map_or(false, |gateway_iso_code| gateway_iso_code == code)
    }

    pub fn country_name(&self) -> Option<String> {
        self.location.as_ref().map(|l| l.country_name.clone())
    }
}

impl From<DescribedGateway> for DescribedGatewayWithLocation {
    fn from(gateway: DescribedGateway) -> Self {
        DescribedGatewayWithLocation {
            gateway,
            location: None,
        }
    }
}
