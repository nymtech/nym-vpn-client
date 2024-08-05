// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[derive(Debug, Clone, PartialEq)]
pub struct Country {
    iso_code: String,
}

impl Country {
    pub fn iso_code(&self) -> &str {
        &self.iso_code
    }
}

impl From<nym_vpn_api_client::Country> for Country {
    fn from(country: nym_vpn_api_client::Country) -> Self {
        Self {
            iso_code: country.iso_code().to_string(),
        }
    }
}
