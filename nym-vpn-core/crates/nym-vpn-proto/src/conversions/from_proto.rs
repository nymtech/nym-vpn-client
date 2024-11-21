// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use super::error::ConversionError;

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

impl From<crate::Location> for nym_vpnd_types::gateway::Location {
    fn from(location: crate::Location) -> Self {
        Self {
            two_letter_iso_country_code: location.two_letter_iso_country_code,
            latitude: location.latitude,
            longitude: location.longitude,
        }
    }
}

impl From<crate::AsEntry> for nym_vpnd_types::gateway::Entry {
    fn from(entry: crate::AsEntry) -> Self {
        Self {
            can_connect: entry.can_connect,
            can_route: entry.can_route,
        }
    }
}

impl From<crate::AsExit> for nym_vpnd_types::gateway::Exit {
    fn from(exit: crate::AsExit) -> Self {
        Self {
            can_connect: exit.can_connect,
            can_route_ip_v4: exit.can_route_ip_v4,
            can_route_ip_external_v4: exit.can_route_ip_external_v4,
            can_route_ip_v6: exit.can_route_ip_v6,
            can_route_ip_external_v6: exit.can_route_ip_external_v6,
        }
    }
}

impl TryFrom<crate::ProbeOutcome> for nym_vpnd_types::gateway::ProbeOutcome {
    type Error = ConversionError;

    fn try_from(outcome: crate::ProbeOutcome) -> Result<Self, Self::Error> {
        let as_entry = outcome
            .as_entry
            .map(nym_vpnd_types::gateway::Entry::from)
            .ok_or(ConversionError::generic("missing as entry"))?;
        let as_exit = outcome.as_exit.map(nym_vpnd_types::gateway::Exit::from);
        Ok(Self { as_entry, as_exit })
    }
}

impl TryFrom<crate::Probe> for nym_vpnd_types::gateway::Probe {
    type Error = ConversionError;

    fn try_from(probe: crate::Probe) -> Result<Self, Self::Error> {
        let last_updated_utc = probe
            .last_updated_utc
            .ok_or(ConversionError::generic("missing last updated timestamp"))
            .map(|timestamp| timestamp.to_string())?;
        let outcome = probe
            .outcome
            .ok_or(ConversionError::generic("missing probe outcome"))
            .and_then(nym_vpnd_types::gateway::ProbeOutcome::try_from)?;
        Ok(Self {
            last_updated_utc,
            outcome,
        })
    }
}

impl TryFrom<crate::GatewayResponse> for nym_vpnd_types::gateway::Gateway {
    type Error = ConversionError;
    fn try_from(gateway: crate::GatewayResponse) -> Result<Self, Self::Error> {
        let identity_key = gateway
            .id
            .map(|id| id.id)
            .ok_or_else(|| ConversionError::generic("missing gateway id"))?;
        let location = gateway
            .location
            .map(nym_vpnd_types::gateway::Location::from);
        let last_probe = gateway
            .last_probe
            .map(nym_vpnd_types::gateway::Probe::try_from)
            .transpose()?;
        Ok(Self {
            identity_key,
            location,
            last_probe,
        })
    }
}

impl From<crate::Location> for nym_vpnd_types::gateway::Country {
    fn from(location: crate::Location) -> Self {
        Self {
            iso_code: location.two_letter_iso_country_code,
        }
    }
}

impl TryFrom<crate::GatewayType> for nym_vpn_lib::gateway_directory::GatewayType {
    type Error = ConversionError;

    fn try_from(gateway_type: crate::GatewayType) -> Result<Self, Self::Error> {
        use nym_vpn_lib::gateway_directory::GatewayType;
        let gw_type = match gateway_type {
            crate::GatewayType::Unspecified => {
                return Err(ConversionError::Generic(
                    "gateway type unspecified".to_string(),
                ))
            }
            crate::GatewayType::MixnetEntry => GatewayType::MixnetEntry,
            crate::GatewayType::MixnetExit => GatewayType::MixnetExit,
            crate::GatewayType::Wg => GatewayType::Wg,
        };
        Ok(gw_type)
    }
}

impl From<crate::UserAgent> for nym_vpn_lib::UserAgent {
    fn from(user_agent: crate::UserAgent) -> Self {
        Self {
            application: user_agent.application,
            version: user_agent.version,
            platform: user_agent.platform,
            git_commit: user_agent.git_commit,
        }
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
