// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::fmt;

struct NymVpnNetworkDetails {
    nym_vpn_api_url: String,
}

impl From<nym_vpn_proto::NymVpnNetworkDetails> for NymVpnNetworkDetails {
    fn from(network: nym_vpn_proto::NymVpnNetworkDetails) -> Self {
        NymVpnNetworkDetails {
            nym_vpn_api_url: network
                .nym_vpn_api_url
                .map(|u| u.url)
                .unwrap_or("not set".to_string()),
        }
    }
}

struct NymContracts {
    mixnet_contract_address: String,
    vesting_contract_address: String,
    ecash_contract_address: String,
    group_contract_address: String,
    multisig_contract_address: String,
    coconut_dkg_contract_address: String,
}

impl From<nym_vpn_proto::NymContracts> for NymContracts {
    fn from(contracts: nym_vpn_proto::NymContracts) -> Self {
        NymContracts {
            mixnet_contract_address: contracts
                .mixnet_contract_address
                .unwrap_or("not set".to_string()),
            vesting_contract_address: contracts
                .vesting_contract_address
                .unwrap_or("not set".to_string()),
            ecash_contract_address: contracts
                .ecash_contract_address
                .unwrap_or("not set".to_string()),
            group_contract_address: contracts
                .group_contract_address
                .unwrap_or("not set".to_string()),
            multisig_contract_address: contracts
                .multisig_contract_address
                .unwrap_or("not set".to_string()),
            coconut_dkg_contract_address: contracts
                .coconut_dkg_contract_address
                .unwrap_or("not set".to_string()),
        }
    }
}

struct DenomDetails {
    base: String,
    display: String,
    display_exponent: u32,
}

impl From<nym_vpn_proto::DenomDetails> for DenomDetails {
    fn from(details: nym_vpn_proto::DenomDetails) -> Self {
        DenomDetails {
            base: details.base,
            display: details.display,
            display_exponent: details.display_exponent,
        }
    }
}

struct ChainDetails {
    bech32_account_prefix: String,
    mix_denom: DenomDetails,
    stake_denom: DenomDetails,
}

impl From<nym_vpn_proto::ChainDetails> for ChainDetails {
    fn from(details: nym_vpn_proto::ChainDetails) -> Self {
        let mix_denom = details.mix_denom.clone().map(DenomDetails::from).unwrap();
        let stake_denom = details.stake_denom.clone().map(DenomDetails::from).unwrap();
        ChainDetails {
            bech32_account_prefix: details.bech32_account_prefix,
            mix_denom,
            stake_denom,
        }
    }
}

struct ValidatorDetails {
    nyxd_url: String,
    api_url: String,
    websocket_url: String,
}

impl From<nym_vpn_proto::ValidatorDetails> for ValidatorDetails {
    fn from(details: nym_vpn_proto::ValidatorDetails) -> Self {
        let nyxd_url = details
            .nyxd_url
            .clone()
            .map(|p| p.url)
            .unwrap_or("not set".to_string());
        let api_url = details
            .api_url
            .clone()
            .map(|p| p.url)
            .unwrap_or("not set".to_string());
        let websocket_url = details
            .websocket_url
            .clone()
            .map(|p| p.url)
            .unwrap_or("not set".to_string());
        ValidatorDetails {
            nyxd_url,
            api_url,
            websocket_url,
        }
    }
}

struct NymNetworkDetails {
    network_name: String,
    chain_details: ChainDetails,
    endpoints: Vec<ValidatorDetails>,
    contracts: NymContracts,
}

pub struct Info {
    version: String,
    build_timestamp: String,
    triple: String,
    platform: String,
    git_commit: String,
    nym_network: NymNetworkDetails,
    nym_vpn_network: NymVpnNetworkDetails,
}

impl From<nym_vpn_proto::InfoResponse> for Info {
    fn from(response: nym_vpn_proto::InfoResponse) -> Self {
        let utc_build_timestamp = response
            .build_timestamp
            .and_then(|ts| crate::protobuf_conversion::parse_offset_datetime(ts).ok())
            .map(|ts| ts.to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let nym_network = response.nym_network.unwrap();
        let chain_details = nym_network
            .chain_details
            .clone()
            .map(ChainDetails::from)
            .unwrap();
        let endpoints = nym_network
            .endpoints
            .into_iter()
            .map(ValidatorDetails::from)
            .collect();
        let contracts = nym_network
            .contracts
            .clone()
            .map(NymContracts::from)
            .unwrap();
        let nym_network_details = NymNetworkDetails {
            network_name: nym_network.network_name,
            chain_details,
            endpoints,
            contracts,
        };

        let nym_vpn_network_details = response
            .nym_vpn_network
            .map(NymVpnNetworkDetails::from)
            .unwrap();

        Info {
            version: response.version,
            build_timestamp: utc_build_timestamp,
            triple: response.triple,
            platform: response.platform,
            git_commit: response.git_commit,
            nym_network: nym_network_details,
            nym_vpn_network: nym_vpn_network_details,
        }
    }
}

impl From<nym_vpn_proto::NymNetworkDetails> for NymNetworkDetails {
    fn from(details: nym_vpn_proto::NymNetworkDetails) -> Self {
        let chain_details = details
            .chain_details
            .clone()
            .map(ChainDetails::from)
            .unwrap();
        let endpoints = details
            .endpoints
            .into_iter()
            .map(ValidatorDetails::from)
            .collect();
        let contracts = details.contracts.clone().map(NymContracts::from).unwrap();
        NymNetworkDetails {
            network_name: details.network_name,
            chain_details,
            endpoints,
            contracts,
        }
    }
}

impl fmt::Display for Info {
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
            writeln!(f, "    api_url:               {}", validator.api_url)?;
            writeln!(f, "    websocket_url:         {}", validator.websocket_url)?;
        }

        writeln!(f, "  nym_contracts:")?;
        writeln!(f, "    mixnet_contract_address:       {}", self.nym_network.contracts.mixnet_contract_address)?;
        writeln!(f, "    vesting_contract_address:      {}", self.nym_network.contracts.vesting_contract_address)?;
        writeln!(f, "    ecash_contract_address:        {}", self.nym_network.contracts.ecash_contract_address)?;
        writeln!(f, "    group_contract_address:        {}", self.nym_network.contracts.group_contract_address)?;
        writeln!(f, "    multisig_contract_address:     {}", self.nym_network.contracts.multisig_contract_address)?;
        writeln!(f, "    coconut_dkg_contract_address:  {}", self.nym_network.contracts.coconut_dkg_contract_address)?;

        writeln!(f, "\nnym_vpn_network:")?;
        writeln!(f, "  nym_vpn_api_url: {}", self.nym_vpn_network.nym_vpn_api_url)
    }
}
