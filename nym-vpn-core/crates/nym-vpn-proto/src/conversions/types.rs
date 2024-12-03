// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Strongly typed structures representing various types of responses from the nym-vpnd
//! API.

use std::fmt;

use time::OffsetDateTime;

use super::{util::or_not_set, ConversionError};

pub struct InfoResponse {
    pub version: String,
    pub build_timestamp: OffsetDateTime,
    pub triple: String,
    pub platform: String,
    pub git_commit: String,
    pub nym_network: nym_config::defaults::NymNetworkDetails,
    pub nym_vpn_network: NymVpnNetworkDetails,
}

pub struct NymVpnNetworkDetails {
    nym_vpn_api_url: String,
}

impl TryFrom<crate::NymVpnNetworkDetails> for NymVpnNetworkDetails {
    type Error = ConversionError;

    fn try_from(network: crate::NymVpnNetworkDetails) -> Result<Self, Self::Error> {
        let nym_vpn_api_url = network
            .nym_vpn_api_url
            .ok_or(ConversionError::generic("missing nym vpn api url"))
            .map(|u| u.url)?;
        Ok(NymVpnNetworkDetails { nym_vpn_api_url })
    }
}

impl TryFrom<crate::InfoResponse> for InfoResponse {
    type Error = ConversionError;

    fn try_from(response: crate::InfoResponse) -> Result<Self, Self::Error> {
        let build_timestamp = response
            .build_timestamp
            .ok_or(ConversionError::generic("missing build timestamp"))
            .and_then(|timestamp| {
                super::prost::prost_timestamp_into_offset_datetime(timestamp)
                    .map_err(ConversionError::generic)
            })?;
        let nym_network = response
            .nym_network
            .ok_or(ConversionError::generic("missing nym network"))
            .and_then(nym_config::defaults::NymNetworkDetails::try_from)?;
        let nym_vpn_network = response
            .nym_vpn_network
            .ok_or(ConversionError::generic("missing nym vpn network"))
            .and_then(NymVpnNetworkDetails::try_from)?;

        Ok(InfoResponse {
            version: response.version,
            build_timestamp,
            triple: response.triple,
            platform: response.platform,
            git_commit: response.git_commit,
            nym_network,
            nym_vpn_network,
        })
    }
}

impl fmt::Display for InfoResponse {
    #[rustfmt::skip]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "nym-vpnd:")?;
        writeln!(f, "  version:               {}", self.version)?;
        writeln!(f, "  build_timestamp (utc): {}", self.build_timestamp)?;
        writeln!(f, "  triple:                {}", self.triple)?;
        writeln!(f, "  platform:              {}", self.platform)?;
        writeln!(f, "  git_commit:            {}", self.git_commit)?;

        writeln!(f, "\nnym_network:")?;
        writeln!(f, "  network_name:          {}", self.nym_network.network_name)?;
        writeln!(f, "  chain_details:")?;
        writeln!(f, "    bech32_account_prefix: {}", self.nym_network.chain_details.bech32_account_prefix)?;
        writeln!(f, "    mix_denom:")?;
        writeln!(f, "      base:               {}", self.nym_network.chain_details.mix_denom.base)?;
        writeln!(f, "      display:            {}", self.nym_network.chain_details.mix_denom.display)?;
        writeln!(f, "      display_exponent:   {}", self.nym_network.chain_details.mix_denom.display_exponent)?;
        writeln!(f, "    stake_denom:")?;
        writeln!(f, "      base:               {}", self.nym_network.chain_details.stake_denom.base)?;
        writeln!(f, "      display:            {}", self.nym_network.chain_details.stake_denom.display)?;
        writeln!(f, "      display_exponent:   {}", self.nym_network.chain_details.stake_denom.display_exponent)?;

        writeln!(f, "  validators:")?;
        for validator in &self.nym_network.endpoints {
            writeln!(f, "    nyxd_url:              {}", validator.nyxd_url)?;
            writeln!(f, "    api_url:               {}", or_not_set(&validator.api_url))?;
            writeln!(f, "    websocket_url:         {}", or_not_set(&validator.websocket_url))?;
        }

        writeln!(f, "  nym_contracts:")?;
        writeln!(f, "    mixnet_contract_address:       {}", or_not_set(&self.nym_network.contracts.mixnet_contract_address))?;
        writeln!(f, "    vesting_contract_address:      {}", or_not_set(&self.nym_network.contracts.vesting_contract_address))?;
        writeln!(f, "    ecash_contract_address:        {}", or_not_set(&self.nym_network.contracts.ecash_contract_address))?;
        writeln!(f, "    group_contract_address:        {}", or_not_set(&self.nym_network.contracts.group_contract_address))?;
        writeln!(f, "    multisig_contract_address:     {}", or_not_set(&self.nym_network.contracts.multisig_contract_address))?;
        writeln!(f, "    coconut_dkg_contract_address:  {}", or_not_set(&self.nym_network.contracts.coconut_dkg_contract_address))?;

        writeln!(f, "\nnym_vpn_network:")?;
        writeln!(f, "  nym_vpn_api_url: {}", self.nym_vpn_network.nym_vpn_api_url)
    }
}
