// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

impl From<nym_config::defaults::ChainDetails> for crate::ChainDetails {
    fn from(chain_details: nym_config::defaults::ChainDetails) -> Self {
        crate::ChainDetails {
            bech32_account_prefix: chain_details.bech32_account_prefix,
            mix_denom: Some(chain_details.mix_denom.into()),
            stake_denom: Some(chain_details.stake_denom.into()),
        }
    }
}

impl From<nym_config::defaults::DenomDetailsOwned> for crate::DenomDetails {
    fn from(denom_details: nym_config::defaults::DenomDetailsOwned) -> Self {
        crate::DenomDetails {
            base: denom_details.base,
            display: denom_details.display,
            display_exponent: denom_details.display_exponent,
        }
    }
}

impl From<nym_config::defaults::NymContracts> for crate::NymContracts {
    fn from(contracts: nym_config::defaults::NymContracts) -> Self {
        crate::NymContracts {
            mixnet_contract_address: contracts.mixnet_contract_address,
            vesting_contract_address: contracts.vesting_contract_address,
            ecash_contract_address: contracts.ecash_contract_address,
            group_contract_address: contracts.group_contract_address,
            multisig_contract_address: contracts.multisig_contract_address,
            coconut_dkg_contract_address: contracts.coconut_dkg_contract_address,
        }
    }
}

impl From<nym_config::defaults::ValidatorDetails> for crate::ValidatorDetails {
    fn from(validator_details: nym_config::defaults::ValidatorDetails) -> Self {
        let nyxd_url = Some(crate::Url::from(validator_details.nyxd_url));
        let api_url = validator_details.api_url.map(crate::Url::from);
        let websocket_url = validator_details.websocket_url.map(crate::Url::from);
        crate::ValidatorDetails {
            nyxd_url,
            api_url,
            websocket_url,
        }
    }
}

impl From<nym_vpn_network_config::NymNetwork> for crate::NymNetworkDetails {
    fn from(nym_network: nym_vpn_network_config::NymNetwork) -> Self {
        crate::NymNetworkDetails {
            network_name: nym_network.network.network_name,
            chain_details: Some(nym_network.network.chain_details.into()),
            endpoints: nym_network
                .network
                .endpoints
                .into_iter()
                .map(crate::ValidatorDetails::from)
                .collect(),
            contracts: Some(nym_network.network.contracts.into()),
        }
    }
}

impl From<nym_vpn_network_config::NymVpnNetwork> for crate::NymVpnNetworkDetails {
    fn from(nym_vpn_network: nym_vpn_network_config::NymVpnNetwork) -> Self {
        crate::NymVpnNetworkDetails {
            nym_vpn_api_url: Some(crate::Url::from(nym_vpn_network.nym_vpn_api_url)),
        }
    }
}

impl From<nym_vpn_network_config::SystemMessage> for crate::SystemMessage {
    fn from(system_message: nym_vpn_network_config::SystemMessage) -> Self {
        Self {
            name: system_message.name,
            message: system_message.message,
            properties: system_message.properties.into_inner(),
        }
    }
}

impl From<nym_vpn_network_config::NetworkCompatibility> for crate::NetworkCompatibility {
    fn from(value: nym_vpn_network_config::NetworkCompatibility) -> Self {
        Self {
            core: value.core,
            ios: value.ios,
            macos: value.macos,
            tauri: value.tauri,
            android: value.android,
        }
    }
}

impl From<nym_vpn_network_config::ParsedAccountLinks> for crate::AccountManagement {
    fn from(account_links: nym_vpn_network_config::ParsedAccountLinks) -> Self {
        crate::AccountManagement {
            sign_up: Some(crate::Url::from(account_links.sign_up)),
            sign_in: Some(crate::Url::from(account_links.sign_in)),
            account: account_links.account.map(crate::Url::from),
        }
    }
}

impl From<nym_vpn_network_config::FeatureFlags> for crate::GetFeatureFlagsResponse {
    fn from(feature_flags: nym_vpn_network_config::FeatureFlags) -> Self {
        let mut response = crate::GetFeatureFlagsResponse {
            flags: Default::default(),
            groups: Default::default(),
        };

        for (k, v) in feature_flags.flags {
            match v {
                nym_vpn_network_config::feature_flags::FlagValue::Value(value) => {
                    response.flags.insert(k, value);
                }
                nym_vpn_network_config::feature_flags::FlagValue::Group(group) => {
                    let group = group.into_iter().collect();
                    response
                        .groups
                        .insert(k, crate::FeatureFlagGroup { map: group });
                }
            }
        }

        response
    }
}
