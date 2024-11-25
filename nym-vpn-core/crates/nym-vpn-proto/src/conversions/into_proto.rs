// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpnd_types::gateway;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

impl From<gateway::Location> for crate::Location {
    fn from(location: gateway::Location) -> Self {
        crate::Location {
            two_letter_iso_country_code: location.two_letter_iso_country_code,
            latitude: location.latitude,
            longitude: location.longitude,
        }
    }
}

impl From<gateway::Entry> for crate::AsEntry {
    fn from(entry: gateway::Entry) -> Self {
        crate::AsEntry {
            can_connect: entry.can_connect,
            can_route: entry.can_route,
        }
    }
}

impl From<gateway::Exit> for crate::AsExit {
    fn from(exit: gateway::Exit) -> Self {
        crate::AsExit {
            can_connect: exit.can_connect,
            can_route_ip_v4: exit.can_route_ip_v4,
            can_route_ip_v6: exit.can_route_ip_v6,
            can_route_ip_external_v4: exit.can_route_ip_external_v4,
            can_route_ip_external_v6: exit.can_route_ip_external_v6,
        }
    }
}

impl From<gateway::ProbeOutcome> for crate::ProbeOutcome {
    fn from(outcome: gateway::ProbeOutcome) -> Self {
        let as_entry = Some(crate::AsEntry::from(outcome.as_entry));
        let as_exit = outcome.as_exit.map(crate::AsExit::from);
        let wg = None;
        crate::ProbeOutcome {
            as_entry,
            as_exit,
            wg,
        }
    }
}

impl From<gateway::Probe> for crate::Probe {
    fn from(probe: gateway::Probe) -> Self {
        let last_updated = OffsetDateTime::parse(&probe.last_updated_utc, &Rfc3339).ok();
        let last_updated_utc = last_updated.map(|timestamp| prost_types::Timestamp {
            seconds: timestamp.unix_timestamp(),
            nanos: timestamp.nanosecond() as i32,
        });
        let outcome = Some(crate::ProbeOutcome::from(probe.outcome));
        crate::Probe {
            last_updated_utc,
            outcome,
        }
    }
}

impl From<gateway::Gateway> for crate::GatewayResponse {
    fn from(gateway: gateway::Gateway) -> Self {
        let id = Some(crate::Gateway {
            id: gateway.identity_key.to_string(),
        });
        let location = gateway.location.map(crate::Location::from);
        let last_probe = gateway.last_probe.map(crate::Probe::from);
        crate::GatewayResponse {
            id,
            location,
            last_probe,
        }
    }
}

impl From<gateway::Country> for crate::Location {
    fn from(country: gateway::Country) -> Self {
        crate::Location {
            two_letter_iso_country_code: country.iso_code().to_string(),
            latitude: None,
            longitude: None,
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

impl From<nym_vpn_network_config::NymVpnNetwork> for crate::NymVpnNetworkDetails {
    fn from(nym_vpn_network: nym_vpn_network_config::NymVpnNetwork) -> Self {
        crate::NymVpnNetworkDetails {
            nym_vpn_api_url: Some(crate::Url::from(nym_vpn_network.nym_vpn_api_url)),
        }
    }
}

impl From<String> for crate::Url {
    fn from(url: String) -> Self {
        crate::Url { url }
    }
}

impl From<url::Url> for crate::Url {
    fn from(url: url::Url) -> Self {
        crate::Url {
            url: url.to_string(),
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

impl From<&nym_vpn_lib::NodeIdentity> for crate::EntryNode {
    fn from(identity: &nym_vpn_lib::NodeIdentity) -> Self {
        Self {
            entry_node_enum: Some(crate::entry_node::EntryNodeEnum::Gateway(crate::Gateway {
                id: identity.to_base58_string(),
            })),
        }
    }
}

impl From<&nym_vpn_lib::NodeIdentity> for crate::ExitNode {
    fn from(identity: &nym_vpn_lib::NodeIdentity) -> Self {
        Self {
            exit_node_enum: Some(crate::exit_node::ExitNodeEnum::Gateway(crate::Gateway {
                id: identity.to_base58_string(),
            })),
        }
    }
}

impl From<&nym_vpn_lib::Recipient> for crate::ExitNode {
    fn from(address: &nym_vpn_lib::Recipient) -> Self {
        Self {
            exit_node_enum: Some(crate::exit_node::ExitNodeEnum::Address(crate::Address {
                nym_address: address.to_string(),
            })),
        }
    }
}

impl From<std::net::IpAddr> for crate::Dns {
    fn from(ip: std::net::IpAddr) -> Self {
        Self { ip: ip.to_string() }
    }
}

impl From<u8> for crate::Threshold {
    fn from(performance: u8) -> Self {
        Self {
            min_performance: performance.into(),
        }
    }
}
