// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::collections::HashMap;

use crate::{conversions::ConversionError, proto};

impl TryFrom<proto::ChainDetails> for nym_vpn_lib_types::ChainDetails {
    type Error = ConversionError;

    fn try_from(details: proto::ChainDetails) -> Result<Self, Self::Error> {
        let mix_denom = details
            .mix_denom
            .clone()
            .map(nym_vpn_lib_types::DenomDetailsOwned::from)
            .ok_or_else(|| ConversionError::Generic("missing mix denom".to_string()))?;
        let stake_denom = details
            .stake_denom
            .clone()
            .map(nym_vpn_lib_types::DenomDetailsOwned::from)
            .ok_or_else(|| ConversionError::Generic("missing stake denom".to_string()))?;
        Ok(Self {
            bech32_account_prefix: details.bech32_account_prefix,
            mix_denom,
            stake_denom,
        })
    }
}

impl From<proto::DenomDetails> for nym_vpn_lib_types::DenomDetailsOwned {
    fn from(details: proto::DenomDetails) -> Self {
        Self {
            base: details.base,
            display: details.display,
            display_exponent: details.display_exponent,
        }
    }
}

impl From<proto::ValidatorDetails> for nym_vpn_lib_types::ValidatorDetails {
    fn from(details: proto::ValidatorDetails) -> Self {
        let nyxd_url = details.nyxd_url;
        let api_url = Some(details.api_url);
        let websocket_url = Some(details.websocket_url);
        Self {
            nyxd_url,
            api_url,
            websocket_url,
        }
    }
}

impl From<proto::NymContracts> for nym_vpn_lib_types::NymContracts {
    fn from(contracts: proto::NymContracts) -> Self {
        Self {
            mixnet_contract_address: contracts.mixnet_contract_address,
            vesting_contract_address: contracts.vesting_contract_address,
            performance_contract_address: contracts.performance_contract_address,
            ecash_contract_address: contracts.ecash_contract_address,
            group_contract_address: contracts.group_contract_address,
            multisig_contract_address: contracts.multisig_contract_address,
            coconut_dkg_contract_address: contracts.coconut_dkg_contract_address,
        }
    }
}

impl TryFrom<proto::NymNetworkDetails> for nym_vpn_lib_types::NymNetworkDetails {
    type Error = ConversionError;

    fn try_from(details: proto::NymNetworkDetails) -> Result<Self, Self::Error> {
        let chain_details = details
            .chain_details
            .map(nym_vpn_lib_types::ChainDetails::try_from)
            .ok_or_else(|| ConversionError::Generic("missing chain details".to_string()))??;
        let endpoints = details
            .endpoints
            .clone()
            .into_iter()
            .map(nym_vpn_lib_types::ValidatorDetails::from)
            .collect::<Vec<_>>();
        let contracts = details
            .contracts
            .clone()
            .map(nym_vpn_lib_types::NymContracts::from)
            .ok_or_else(|| ConversionError::Generic("missing contracts".to_string()))?;
        let nym_api_urls = details
            .nym_api_urls
            .into_iter()
            .map(nym_vpn_lib_types::ApiUrl::from)
            .collect();
        let nym_vpn_api_urls = details
            .nym_vpn_api_urls
            .into_iter()
            .map(nym_vpn_lib_types::ApiUrl::from)
            .collect();

        Ok(Self {
            network_name: details.network_name,
            chain_details,
            endpoints,
            contracts,
            nym_api_urls: Some(nym_api_urls),
            nym_vpn_api_urls: Some(nym_vpn_api_urls),
        })
    }
}

impl From<proto::AccountManagement> for nym_vpn_lib_types::ParsedAccountLinks {
    fn from(account_management: proto::AccountManagement) -> Self {
        let sign_up = account_management.sign_up;
        let sign_in = account_management.sign_in;
        let account = account_management.account;

        Self {
            sign_up,
            sign_in,
            account,
        }
    }
}

impl From<proto::NetworkCompatibility> for nym_vpn_lib_types::NetworkCompatibility {
    fn from(value: proto::NetworkCompatibility) -> Self {
        Self {
            core: value.core,
            ios: value.ios,
            macos: value.macos,
            tauri: value.tauri,
            android: value.android,
        }
    }
}

impl From<proto::GetFeatureFlagsResponse> for nym_vpn_lib_types::FeatureFlags {
    fn from(feature_flags: proto::GetFeatureFlagsResponse) -> Self {
        let mut data = HashMap::default();

        for (k, v) in feature_flags.flags {
            data.insert(k, nym_vpn_lib_types::FlagValue::Value(v));
        }

        for (k, v) in feature_flags.groups {
            data.insert(k, nym_vpn_lib_types::FlagValue::Group(v.map));
        }

        Self { flags: data }
    }
}

impl From<nym_vpn_lib_types::ChainDetails> for proto::ChainDetails {
    fn from(chain_details: nym_vpn_lib_types::ChainDetails) -> Self {
        proto::ChainDetails {
            bech32_account_prefix: chain_details.bech32_account_prefix,
            mix_denom: Some(chain_details.mix_denom.into()),
            stake_denom: Some(chain_details.stake_denom.into()),
        }
    }
}

impl From<nym_vpn_lib_types::DenomDetailsOwned> for proto::DenomDetails {
    fn from(denom_details: nym_vpn_lib_types::DenomDetailsOwned) -> Self {
        proto::DenomDetails {
            base: denom_details.base,
            display: denom_details.display,
            display_exponent: denom_details.display_exponent,
        }
    }
}

impl From<nym_vpn_lib_types::NymContracts> for proto::NymContracts {
    fn from(contracts: nym_vpn_lib_types::NymContracts) -> Self {
        proto::NymContracts {
            mixnet_contract_address: contracts.mixnet_contract_address,
            vesting_contract_address: contracts.vesting_contract_address,
            performance_contract_address: contracts.performance_contract_address,
            ecash_contract_address: contracts.ecash_contract_address,
            group_contract_address: contracts.group_contract_address,
            multisig_contract_address: contracts.multisig_contract_address,
            coconut_dkg_contract_address: contracts.coconut_dkg_contract_address,
        }
    }
}

impl From<nym_vpn_lib_types::ValidatorDetails> for proto::ValidatorDetails {
    fn from(validator_details: nym_vpn_lib_types::ValidatorDetails) -> Self {
        let nyxd_url = validator_details.nyxd_url;
        // todo: rework to return None
        let api_url = validator_details.api_url.unwrap_or_default();
        // todo: rework to return None
        let websocket_url = validator_details.websocket_url.unwrap_or_default();
        proto::ValidatorDetails {
            nyxd_url,
            api_url,
            websocket_url,
        }
    }
}

impl From<nym_vpn_lib_types::NymNetworkDetails> for proto::NymNetworkDetails {
    fn from(nym_network: nym_vpn_lib_types::NymNetworkDetails) -> Self {
        let endpoints = nym_network
            .endpoints
            .into_iter()
            .map(proto::ValidatorDetails::from)
            .collect();
        let nym_api_urls = nym_network
            .nym_api_urls
            .unwrap_or_default()
            .into_iter()
            .map(proto::ApiUrl::from)
            .collect();
        let nym_vpn_api_urls = nym_network
            .nym_vpn_api_urls
            .unwrap_or_default()
            .into_iter()
            .map(proto::ApiUrl::from)
            .collect();

        proto::NymNetworkDetails {
            network_name: nym_network.network_name,
            chain_details: Some(nym_network.chain_details.into()),
            endpoints,
            contracts: Some(nym_network.contracts.into()),
            nym_api_urls,
            nym_vpn_api_urls,
        }
    }
}

impl From<nym_vpn_lib_types::ApiUrl> for proto::ApiUrl {
    fn from(value: nym_vpn_lib_types::ApiUrl) -> Self {
        Self {
            url: value.url,
            front_hosts: value.front_hosts.unwrap_or_default(),
        }
    }
}

impl From<proto::ApiUrl> for nym_vpn_lib_types::ApiUrl {
    fn from(value: proto::ApiUrl) -> Self {
        Self {
            url: value.url,
            front_hosts: Some(value.front_hosts),
        }
    }
}

impl From<nym_vpn_lib_types::NymVpnNetwork> for proto::NymVpnNetworkDetails {
    fn from(nym_vpn_network: nym_vpn_lib_types::NymVpnNetwork) -> Self {
        proto::NymVpnNetworkDetails {
            nym_vpn_api_urls: nym_vpn_network
                .nym_vpn_api_urls
                .into_iter()
                .map(proto::ApiUrl::from)
                .collect(),
        }
    }
}

impl From<nym_vpn_lib_types::SystemMessage> for proto::SystemMessage {
    fn from(system_message: nym_vpn_lib_types::SystemMessage) -> Self {
        Self {
            name: system_message.name,
            message: system_message.message,
            properties: system_message.properties.unwrap_or_default(),
        }
    }
}

impl From<nym_vpn_lib_types::NetworkCompatibility> for proto::NetworkCompatibility {
    fn from(value: nym_vpn_lib_types::NetworkCompatibility) -> Self {
        Self {
            core: value.core,
            ios: value.ios,
            macos: value.macos,
            tauri: value.tauri,
            android: value.android,
        }
    }
}

impl From<nym_vpn_lib_types::ParsedAccountLinks> for proto::AccountManagement {
    fn from(account_links: nym_vpn_lib_types::ParsedAccountLinks) -> Self {
        proto::AccountManagement {
            sign_up: account_links.sign_up,
            sign_in: account_links.sign_in,
            account: account_links.account,
        }
    }
}

impl From<nym_vpn_lib_types::FeatureFlags> for proto::GetFeatureFlagsResponse {
    fn from(feature_flags: nym_vpn_lib_types::FeatureFlags) -> Self {
        let mut response = proto::GetFeatureFlagsResponse {
            flags: Default::default(),
            groups: Default::default(),
        };

        for (k, v) in feature_flags.flags {
            match v {
                nym_vpn_lib_types::FlagValue::Value(value) => {
                    response.flags.insert(k, value);
                }
                nym_vpn_lib_types::FlagValue::Group(group) => {
                    let group = group.into_iter().collect();
                    response
                        .groups
                        .insert(k, proto::FeatureFlagGroup { map: group });
                }
            }
        }

        response
    }
}
