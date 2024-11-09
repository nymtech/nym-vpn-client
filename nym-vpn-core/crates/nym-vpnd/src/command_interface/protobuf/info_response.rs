// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use crate::service::VpnServiceInfo;

impl From<VpnServiceInfo> for nym_vpn_proto::InfoResponse {
    fn from(info: VpnServiceInfo) -> Self {
        let build_timestamp = info
            .build_timestamp
            .map(offset_datetime_into_proto_timestamp);

        let nym_network = Some(into_proto_nym_network_details(info.nym_network.clone()));
        let nym_vpn_network = Some(into_proto_nym_vpn_network_details(
            info.nym_vpn_network.clone(),
        ));

        Self {
            version: info.version,
            build_timestamp,
            triple: info.triple,
            platform: info.platform,
            git_commit: info.git_commit,
            nym_network,
            nym_vpn_network,
        }
    }
}

fn into_proto_nym_network_details(
    nym_network: nym_vpn_network_config::NymNetwork,
) -> nym_vpn_proto::NymNetworkDetails {
    nym_vpn_proto::NymNetworkDetails {
        network_name: nym_network.network.network_name,
        chain_details: Some(into_proto_chain_details(nym_network.network.chain_details)),
        endpoints: nym_network
            .network
            .endpoints
            .into_iter()
            .map(validator_details_into_proto_endpoints)
            .collect(),
        contracts: Some(into_proto_nym_contracts(nym_network.network.contracts)),
    }
}

fn into_proto_chain_details(
    chain_details: nym_vpn_lib::nym_config::defaults::ChainDetails,
) -> nym_vpn_proto::ChainDetails {
    nym_vpn_proto::ChainDetails {
        bech32_account_prefix: chain_details.bech32_account_prefix,
        mix_denom: Some(into_proto_denom_details(chain_details.mix_denom)),
        stake_denom: Some(into_proto_denom_details(chain_details.stake_denom)),
    }
}

fn into_proto_denom_details(
    denom_details: nym_vpn_lib::nym_config::defaults::DenomDetailsOwned,
) -> nym_vpn_proto::DenomDetails {
    nym_vpn_proto::DenomDetails {
        base: denom_details.base,
        display: denom_details.display,
        display_exponent: denom_details.display_exponent,
    }
}

fn into_proto_nym_contracts(
    contracts: nym_vpn_lib::nym_config::defaults::NymContracts,
) -> nym_vpn_proto::NymContracts {
    nym_vpn_proto::NymContracts {
        mixnet_contract_address: contracts.mixnet_contract_address,
        vesting_contract_address: contracts.vesting_contract_address,
        ecash_contract_address: contracts.ecash_contract_address,
        group_contract_address: contracts.group_contract_address,
        multisig_contract_address: contracts.multisig_contract_address,
        coconut_dkg_contract_address: contracts.coconut_dkg_contract_address,
    }
}

fn into_proto_nym_vpn_network_details(
    nym_vpn_network: nym_vpn_network_config::NymVpnNetwork,
) -> nym_vpn_proto::NymVpnNetworkDetails {
    nym_vpn_proto::NymVpnNetworkDetails {
        nym_vpn_api_url: Some(into_proto_url(nym_vpn_network.nym_vpn_api_url)),
    }
}

fn offset_datetime_into_proto_timestamp(datetime: time::OffsetDateTime) -> prost_types::Timestamp {
    prost_types::Timestamp {
        seconds: datetime.unix_timestamp(),
        nanos: datetime.nanosecond() as i32,
    }
}

fn validator_details_into_proto_endpoints(
    validator_details: nym_vpn_lib::nym_config::defaults::ValidatorDetails,
) -> nym_vpn_proto::ValidatorDetails {
    nym_vpn_proto::ValidatorDetails {
        nyxd_url: Some(string_into_proto_url(validator_details.nyxd_url)),
        websocket_url: validator_details.websocket_url.map(string_into_proto_url),
        api_url: validator_details.api_url.map(string_into_proto_url),
    }
}

fn string_into_proto_url(url: String) -> nym_vpn_proto::Url {
    nym_vpn_proto::Url { url }
}

fn into_proto_url(url: url::Url) -> nym_vpn_proto::Url {
    nym_vpn_proto::Url {
        url: url.to_string(),
    }
}

pub(crate) fn into_proto_system_message(
    system_message: nym_vpn_network_config::SystemMessage,
) -> nym_vpn_proto::SystemMessage {
    nym_vpn_proto::SystemMessage {
        name: system_message.name,
        message: system_message.message,
        properties: system_message.properties.into_inner(),
    }
}

pub(crate) fn into_account_management_links(
    account_links: nym_vpn_network_config::ParsedAccountLinks,
) -> nym_vpn_proto::AccountManagement {
    nym_vpn_proto::AccountManagement {
        sign_up: Some(into_proto_url(account_links.sign_up)),
        sign_in: Some(into_proto_url(account_links.sign_in)),
        account: Some(into_proto_url(account_links.account)),
    }
}
