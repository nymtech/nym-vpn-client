// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::collections::HashMap;

use crate::{conversions::ConversionError, proto};

impl TryFrom<proto::ChainDetails> for nym_config::defaults::ChainDetails {
    type Error = ConversionError;

    fn try_from(details: proto::ChainDetails) -> Result<Self, Self::Error> {
        let mix_denom = details
            .mix_denom
            .clone()
            .map(nym_config::defaults::DenomDetailsOwned::from)
            .ok_or_else(|| ConversionError::Generic("missing mix denom".to_string()))?;
        let stake_denom = details
            .stake_denom
            .clone()
            .map(nym_config::defaults::DenomDetailsOwned::from)
            .ok_or_else(|| ConversionError::Generic("missing stake denom".to_string()))?;
        Ok(Self {
            bech32_account_prefix: details.bech32_account_prefix,
            mix_denom,
            stake_denom,
        })
    }
}

impl From<proto::DenomDetails> for nym_config::defaults::DenomDetailsOwned {
    fn from(details: proto::DenomDetails) -> Self {
        nym_config::defaults::DenomDetailsOwned {
            base: details.base,
            display: details.display,
            display_exponent: details.display_exponent,
        }
    }
}

impl TryFrom<proto::ValidatorDetails> for nym_config::defaults::ValidatorDetails {
    type Error = ConversionError;

    fn try_from(details: proto::ValidatorDetails) -> Result<Self, Self::Error> {
        let nyxd_url = details
            .nyxd_url
            .clone()
            .map(|url| url.url)
            .ok_or_else(|| ConversionError::Generic("missing nyxd url".to_string()))?;
        let api_url = details.api_url.clone().map(|url| url.url);
        let websocket_url = details.websocket_url.clone().map(|url| url.url);
        Ok(Self {
            nyxd_url,
            api_url,
            websocket_url,
        })
    }
}

impl From<proto::NymContracts> for nym_config::defaults::NymContracts {
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

impl From<proto::ApiUrl> for nym_config::defaults::ApiUrl {
    fn from(value: proto::ApiUrl) -> Self {
        Self {
            url: value.url,
            front_hosts: Some(value.front_hosts),
        }
    }
}

impl From<nym_config::defaults::ApiUrl> for proto::ApiUrl {
    fn from(value: nym_config::defaults::ApiUrl) -> Self {
        Self {
            url: value.url,
            front_hosts: value.front_hosts.unwrap_or_default(),
        }
    }
}

impl TryFrom<proto::NymNetworkDetails> for nym_config::defaults::NymNetworkDetails {
    type Error = ConversionError;

    fn try_from(details: proto::NymNetworkDetails) -> Result<Self, Self::Error> {
        let chain_details = details
            .chain_details
            .clone()
            .map(nym_config::defaults::ChainDetails::try_from)
            .ok_or_else(|| ConversionError::Generic("missing chain details".to_string()))??;
        let endpoints = details
            .endpoints
            .clone()
            .into_iter()
            .map(nym_config::defaults::ValidatorDetails::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        let contracts = details
            .contracts
            .clone()
            .map(nym_config::defaults::NymContracts::from)
            .ok_or_else(|| ConversionError::Generic("missing contracts".to_string()))?;
        let nym_api_urls = details
            .nym_api_urls
            .into_iter()
            .map(nym_config::defaults::ApiUrl::from)
            .collect();
        let nym_vpn_api_urls = details
            .nym_vpn_api_urls
            .into_iter()
            .map(nym_config::defaults::ApiUrl::from)
            .collect();

        Ok(Self {
            network_name: details.network_name,
            chain_details,
            endpoints,
            contracts,
            nym_vpn_api_url: details.nym_vpn_api_url,
            nym_api_urls: Some(nym_api_urls),
            nym_vpn_api_urls: Some(nym_vpn_api_urls),
        })
    }
}

impl TryFrom<proto::AccountManagement> for nym_vpn_network_config::ParsedAccountLinks {
    type Error = ConversionError;

    fn try_from(account_management: proto::AccountManagement) -> Result<Self, Self::Error> {
        let sign_up = account_management
            .sign_up
            .map(|url| url.url)
            .ok_or(ConversionError::Generic("missing sign up URL".to_string()))?
            .parse::<url::Url>()
            .map_err(|e| ConversionError::Generic(e.to_string()))?;
        let sign_in = account_management
            .sign_in
            .map(|url| url.url)
            .ok_or(ConversionError::Generic("missing sign in URL".to_string()))?
            .parse::<url::Url>()
            .map_err(|e| ConversionError::Generic(e.to_string()))?;
        let account = account_management
            .account
            .and_then(|url| url.url.parse().ok());

        Ok(Self {
            sign_up,
            sign_in,
            account,
        })
    }
}

impl From<proto::NetworkCompatibility> for nym_vpn_api_client::NetworkCompatibility {
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

impl From<proto::GetFeatureFlagsResponse> for nym_vpn_network_config::FeatureFlags {
    fn from(feature_flags: proto::GetFeatureFlagsResponse) -> Self {
        let mut data = HashMap::default();

        for (k, v) in feature_flags.flags {
            data.insert(k, nym_vpn_network_config::FlagValue::Value(v));
        }

        for (k, v) in feature_flags.groups {
            data.insert(k, nym_vpn_network_config::FlagValue::Group(v.map));
        }

        Self::from(data)
    }
}

impl From<nym_config::defaults::ChainDetails> for proto::ChainDetails {
    fn from(chain_details: nym_config::defaults::ChainDetails) -> Self {
        proto::ChainDetails {
            bech32_account_prefix: chain_details.bech32_account_prefix,
            mix_denom: Some(chain_details.mix_denom.into()),
            stake_denom: Some(chain_details.stake_denom.into()),
        }
    }
}

impl From<nym_config::defaults::DenomDetailsOwned> for proto::DenomDetails {
    fn from(denom_details: nym_config::defaults::DenomDetailsOwned) -> Self {
        proto::DenomDetails {
            base: denom_details.base,
            display: denom_details.display,
            display_exponent: denom_details.display_exponent,
        }
    }
}

impl From<nym_config::defaults::NymContracts> for proto::NymContracts {
    fn from(contracts: nym_config::defaults::NymContracts) -> Self {
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

impl From<nym_config::defaults::ValidatorDetails> for proto::ValidatorDetails {
    fn from(validator_details: nym_config::defaults::ValidatorDetails) -> Self {
        let nyxd_url = Some(proto::Url::from(validator_details.nyxd_url));
        let api_url = validator_details.api_url.map(proto::Url::from);
        let websocket_url = validator_details.websocket_url.map(proto::Url::from);
        proto::ValidatorDetails {
            nyxd_url,
            api_url,
            websocket_url,
        }
    }
}

impl From<nym_vpn_network_config::NymNetwork> for proto::NymNetworkDetails {
    fn from(nym_network: nym_vpn_network_config::NymNetwork) -> Self {
        let endpoints = nym_network
            .network
            .endpoints
            .into_iter()
            .map(proto::ValidatorDetails::from)
            .collect();
        let nym_api_urls = nym_network
            .network
            .nym_api_urls
            .unwrap_or_default()
            .into_iter()
            .map(proto::ApiUrl::from)
            .collect();
        let nym_vpn_api_urls = nym_network
            .network
            .nym_vpn_api_urls
            .unwrap_or_default()
            .into_iter()
            .map(proto::ApiUrl::from)
            .collect();

        proto::NymNetworkDetails {
            network_name: nym_network.network.network_name,
            chain_details: Some(nym_network.network.chain_details.into()),
            endpoints,
            contracts: Some(nym_network.network.contracts.into()),
            nym_vpn_api_url: nym_network.network.nym_vpn_api_url,
            nym_api_urls,
            nym_vpn_api_urls,
        }
    }
}

impl From<nym_vpn_network_config::ApiUrl> for proto::ApiUrl {
    fn from(value: nym_vpn_network_config::ApiUrl) -> Self {
        Self {
            url: value.url.to_string(),
            front_hosts: value.fronts.unwrap_or_default(),
        }
    }
}

impl From<proto::ApiUrl> for nym_vpn_network_config::ApiUrl {
    fn from(value: proto::ApiUrl) -> Self {
        Self {
            url: value.url,
            fronts: Some(value.front_hosts),
        }
    }
}

impl From<nym_vpn_network_config::NymVpnNetwork> for proto::NymVpnNetworkDetails {
    fn from(nym_vpn_network: nym_vpn_network_config::NymVpnNetwork) -> Self {
        proto::NymVpnNetworkDetails {
            nym_vpn_api_url: Some(proto::Url::from(nym_vpn_network.nym_vpn_api_url)),
            nym_vpn_api_urls: nym_vpn_network
                .nym_vpn_api_urls
                .into_iter()
                .map(proto::ApiUrl::from)
                .collect(),
        }
    }
}

impl From<nym_vpn_network_config::SystemMessage> for proto::SystemMessage {
    fn from(system_message: nym_vpn_network_config::SystemMessage) -> Self {
        Self {
            name: system_message.name,
            message: system_message.message,
            properties: system_message.properties.into_inner(),
        }
    }
}

impl From<nym_vpn_api_client::NetworkCompatibility> for proto::NetworkCompatibility {
    fn from(value: nym_vpn_api_client::NetworkCompatibility) -> Self {
        Self {
            core: value.core,
            ios: value.ios,
            macos: value.macos,
            tauri: value.tauri,
            android: value.android,
        }
    }
}

impl From<nym_vpn_network_config::ParsedAccountLinks> for proto::AccountManagement {
    fn from(account_links: nym_vpn_network_config::ParsedAccountLinks) -> Self {
        proto::AccountManagement {
            sign_up: Some(proto::Url::from(account_links.sign_up)),
            sign_in: Some(proto::Url::from(account_links.sign_in)),
            account: account_links.account.map(proto::Url::from),
        }
    }
}

impl From<nym_vpn_network_config::FeatureFlags> for proto::GetFeatureFlagsResponse {
    fn from(feature_flags: nym_vpn_network_config::FeatureFlags) -> Self {
        let mut response = proto::GetFeatureFlagsResponse {
            flags: Default::default(),
            groups: Default::default(),
        };

        for (k, v) in feature_flags.into_hash_map() {
            match v {
                nym_vpn_network_config::feature_flags::FlagValue::Value(value) => {
                    response.flags.insert(k, value);
                }
                nym_vpn_network_config::feature_flags::FlagValue::Group(group) => {
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
