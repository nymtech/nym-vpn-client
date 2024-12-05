// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::conversions::ConversionError;

impl TryFrom<crate::ChainDetails> for nym_config::defaults::ChainDetails {
    type Error = ConversionError;

    fn try_from(details: crate::ChainDetails) -> Result<Self, Self::Error> {
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

impl From<crate::DenomDetails> for nym_config::defaults::DenomDetailsOwned {
    fn from(details: crate::DenomDetails) -> Self {
        nym_config::defaults::DenomDetailsOwned {
            base: details.base,
            display: details.display,
            display_exponent: details.display_exponent,
        }
    }
}

impl TryFrom<crate::ValidatorDetails> for nym_config::defaults::ValidatorDetails {
    type Error = ConversionError;

    fn try_from(details: crate::ValidatorDetails) -> Result<Self, Self::Error> {
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

impl From<crate::NymContracts> for nym_config::defaults::NymContracts {
    fn from(contracts: crate::NymContracts) -> Self {
        Self {
            mixnet_contract_address: contracts.mixnet_contract_address,
            vesting_contract_address: contracts.vesting_contract_address,
            ecash_contract_address: contracts.ecash_contract_address,
            group_contract_address: contracts.group_contract_address,
            multisig_contract_address: contracts.multisig_contract_address,
            coconut_dkg_contract_address: contracts.coconut_dkg_contract_address,
        }
    }
}

impl TryFrom<crate::NymNetworkDetails> for nym_config::defaults::NymNetworkDetails {
    type Error = ConversionError;

    fn try_from(details: crate::NymNetworkDetails) -> Result<Self, Self::Error> {
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

        Ok(Self {
            network_name: details.network_name,
            chain_details,
            endpoints,
            contracts,
            explorer_api: None,
            nym_vpn_api_url: None,
        })
    }
}

impl TryFrom<crate::AccountManagement> for nym_vpn_network_config::ParsedAccountLinks {
    type Error = ConversionError;

    fn try_from(account_management: crate::AccountManagement) -> Result<Self, Self::Error> {
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
