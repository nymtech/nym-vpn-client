// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

mod cli;

use anyhow::{Context, Result, bail};
use clap::Parser;
use cli::Internal;
use nym_gateway_directory::GatewayType;
use nym_http_api_client::UserAgent;
use nym_vpn_lib_types::TunnelState;
use nym_vpn_proto::rpc_client::RpcClient;
use nym_vpnd_types::{
    ListCountriesOptions, ListGatewaysOptions, StoreAccountRequest,
    service::{ConnectArgs, ConnectOptions, VpnServiceInfo},
};
use sysinfo::System;
use tokio_stream::StreamExt;

use crate::cli::{CliEntry, CliExit, Command};

#[tokio::main]
async fn main() -> Result<()> {
    let args = cli::LegacyCliArgs::parse();
    let mut rpc_client = RpcClient::new()
        .await
        .context("Failed to create RPC client")?;

    let user_agent = if let Some(user_agent) = args.user_agent {
        user_agent
    } else {
        let daemon_info = rpc_client.get_info().await?;
        construct_user_agent(daemon_info)
    };

    match args.command {
        Command::Connect(connect_args) => connect(rpc_client, *connect_args, user_agent).await?,
        Command::ConnectV2 { wait } => connect_v2(rpc_client, wait).await?,
        Command::Reconnect => reconnect(rpc_client).await?,
        Command::Disconnect { wait } => disconnect(rpc_client, wait).await?,
        Command::Status { listen } => status(rpc_client, listen).await?,
        Command::Info => info(rpc_client).await?,
        Command::GetConfig => get_config(rpc_client).await?,
        Command::SetEntry { entry } => set_entry_point(rpc_client, entry).await?,
        Command::SetExit { exit } => set_exit_point(rpc_client, exit).await?,
        Command::SetIpv6 { enabled } => set_disable_ipv6(rpc_client, !*enabled).await?,
        Command::SetTwoHop { enabled } => set_enable_two_hop(rpc_client, *enabled).await?,
        Command::SetNetstack { enabled } => set_netstack(rpc_client, *enabled).await?,
        Command::SetAllowLan { allow } => set_allow_lan(rpc_client, *allow).await?,
        Command::SetNetwork(args) => set_network(rpc_client, args).await?,
        Command::StoreAccount(store_args) => store_account(rpc_client, store_args).await?,
        Command::IsAccountStored => is_account_stored(rpc_client).await?,
        Command::ForgetAccount => forget_account(rpc_client).await?,
        Command::GetAccountId => get_account_id(rpc_client).await?,
        Command::GetAccountLinks(args) => get_account_links(rpc_client, args).await?,
        Command::GetAccountState { listen } => get_account_state(rpc_client, listen).await?,
        Command::ListEntryGateways => {
            list_gateways(rpc_client, GatewayType::MixnetEntry, user_agent).await?
        }
        Command::ListExitGateways => {
            list_gateways(rpc_client, GatewayType::MixnetExit, user_agent).await?
        }
        Command::ListVpnGateways => list_gateways(rpc_client, GatewayType::Wg, user_agent).await?,
        Command::ListEntryCountries => {
            list_countries(rpc_client, GatewayType::MixnetEntry, user_agent).await?
        }
        Command::ListExitCountries => {
            list_countries(rpc_client, GatewayType::MixnetExit, user_agent).await?
        }
        Command::ListVpnCountries => {
            list_countries(rpc_client, GatewayType::Wg, user_agent).await?
        }
        Command::GetDeviceId => get_device_id(rpc_client).await?,
        Command::Internal(internal) => match internal {
            Internal::GetSystemMessages => get_system_messages(rpc_client).await?,
            Internal::GetFeatureFlags => get_feature_flags(rpc_client).await?,
            Internal::SyncAccountState => refresh_account_state(rpc_client).await?,
            Internal::GetAccountUsage => get_account_usage(rpc_client).await?,
            Internal::ResetDeviceIdentity(args) => reset_device_identity(rpc_client, args).await?,
            Internal::GetDevices => get_devices(rpc_client).await?,
            Internal::GetActiveDevices => get_active_devices(rpc_client).await?,
            Internal::GetAvailableTickets => get_available_tickets(rpc_client).await?,
        },
    }
    Ok(())
}

fn construct_user_agent(daemon_info: VpnServiceInfo) -> UserAgent {
    let bin_info = nym_bin_common::bin_info_local_vergen!();
    let version = format!("{} ({})", bin_info.build_version, daemon_info.version);

    // Construct the platform string similar to how user agents are constructed in web browsers
    let name = System::name().unwrap_or("unknown".to_string());
    let os_long = System::long_os_version().unwrap_or("unknown".to_string());
    let arch = System::cpu_arch();
    let platform = format!("{name}; {os_long}; {arch}");

    let git_commit = format!("{} ({})", bin_info.commit_sha, daemon_info.git_commit);
    UserAgent {
        application: bin_info.binary_name.to_string(),
        version,
        platform,
        git_commit,
    }
}

async fn connect(
    mut rpc_client: RpcClient,
    connect_args: cli::ConnectArgs,
    user_agent: UserAgent,
) -> Result<()> {
    println!(
        "This call is deprecated and going to be removed soon. Please switch to using: nym-vpnc connect-v2"
    );

    let options = ConnectArgs {
        entry: connect_args.entry_point()?,
        exit: connect_args.exit_point()?,
        options: ConnectOptions {
            dns: connect_args.dns,
            disable_ipv6: connect_args.disable_ipv6,
            enable_two_hop: connect_args.enable_two_hop,
            netstack: connect_args.netstack,
            disable_poisson_rate: connect_args.disable_poisson_rate,
            disable_background_cover_traffic: connect_args.disable_background_cover_traffic,
            enable_credentials_mode: connect_args.enable_credentials_mode,
            user_agent: Some(user_agent),
        },
    };

    rpc_client.connect_tunnel(options).await?;

    if connect_args.wait {
        println!("Waiting until connected or failed");
        wait_until_connected(rpc_client).await
    } else {
        Ok(())
    }
}

async fn connect_v2(mut rpc_client: RpcClient, wait: bool) -> Result<()> {
    rpc_client.connect_tunnel_v2().await?;

    if wait {
        println!("Waiting until connected or failed");
        wait_until_connected(rpc_client).await
    } else {
        Ok(())
    }
}

async fn reconnect(mut rpc_client: RpcClient) -> Result<()> {
    let _accepted = rpc_client.reconnect_tunnel().await?;
    Ok(())
}

async fn wait_until_connected(mut rpc_client: RpcClient) -> Result<()> {
    let mut stream = rpc_client.listen_to_tunnel_state().await?;
    while let Some(new_state) = stream.next().await {
        let new_state = new_state?;
        println!("{new_state}");

        match new_state {
            TunnelState::Connected { .. } => {
                break;
            }
            TunnelState::Offline { reconnect } => {
                if reconnect {
                    println!("Device is offline. Waiting for network connectivity.");
                } else {
                    bail!("Device is offline");
                }
            }
            TunnelState::Error(reason) => {
                bail!("Tunnel entered error state {:?}", reason);
            }
            _ => {}
        }
    }
    Ok(())
}

async fn disconnect(mut rpc_client: RpcClient, wait: bool) -> Result<()> {
    rpc_client.disconnect_tunnel().await?;

    if wait {
        println!("Waiting until disconnected");
        wait_until_disconnected(rpc_client).await
    } else {
        Ok(())
    }
}

async fn wait_until_disconnected(mut rpc_client: RpcClient) -> Result<()> {
    let mut stream = rpc_client.listen_to_tunnel_state().await?;

    while let Some(new_state) = stream.next().await {
        let new_state = new_state?;

        println!("{new_state}");

        match new_state {
            TunnelState::Disconnected | TunnelState::Offline { .. } => {
                break;
            }
            TunnelState::Error(reason) => {
                bail!("Tunnel entered error state: {:?}", reason)
            }
            _ => {}
        }
    }
    Ok(())
}

async fn status(mut rpc_client: RpcClient, listen: bool) -> Result<()> {
    if listen {
        let mut stream = rpc_client.listen_to_tunnel_state().await?;

        let initial_state = tokio::select! {
            initial_state = rpc_client.get_tunnel_state() => initial_state,
            Some(initial_state) = stream.next() => initial_state
        }?;
        println!("{initial_state}");

        while let Some(new_state) = stream.next().await {
            let new_state = new_state?;
            println!("{new_state}");
        }
    } else {
        let new_state = rpc_client.get_tunnel_state().await?;
        println!("{new_state}");
    }

    Ok(())
}

async fn info(mut rpc_client: RpcClient) -> Result<()> {
    let service_info = rpc_client.get_info().await?;
    print_service_info(service_info);
    Ok(())
}

async fn get_config(mut rpc_client: RpcClient) -> Result<()> {
    let config = rpc_client.get_config().await?;
    println!("{config:#?}");
    Ok(())
}

async fn set_entry_point(mut rpc_client: RpcClient, entry: CliEntry) -> Result<()> {
    rpc_client.set_entry_point(entry.entry_point()?).await?;
    Ok(())
}

async fn set_exit_point(mut rpc_client: RpcClient, exit: CliExit) -> Result<()> {
    rpc_client.set_exit_point(exit.exit_point()?).await?;
    Ok(())
}

async fn set_disable_ipv6(mut rpc_client: RpcClient, disable_ipv6: bool) -> Result<()> {
    rpc_client.set_disable_ipv6(disable_ipv6).await?;
    Ok(())
}

async fn set_enable_two_hop(mut rpc_client: RpcClient, enable_two_hop: bool) -> Result<()> {
    rpc_client.set_enable_two_hop(enable_two_hop).await?;
    Ok(())
}

async fn set_netstack(mut rpc_client: RpcClient, netstack: bool) -> Result<()> {
    rpc_client.set_netstack(netstack).await?;
    Ok(())
}

async fn set_allow_lan(mut rpc_client: RpcClient, allow_lan: bool) -> Result<()> {
    rpc_client.set_allow_lan(allow_lan).await?;
    Ok(())
}

async fn set_network(mut rpc_client: RpcClient, args: cli::SetNetworkArgs) -> Result<()> {
    rpc_client.set_network(args.network).await?;
    Ok(())
}

async fn get_system_messages(mut rpc_client: RpcClient) -> Result<()> {
    let response = rpc_client.get_system_messages().await?;
    println!("{response:#?}");
    Ok(())
}

async fn get_feature_flags(mut rpc_client: RpcClient) -> Result<()> {
    let response = rpc_client.get_feature_flags().await?;
    println!("{response:#?}");
    Ok(())
}

async fn store_account(mut rpc_client: RpcClient, store_args: cli::StoreAccountArgs) -> Result<()> {
    let request = StoreAccountRequest {
        mnemonic: store_args.mnemonic.clone(),
    };
    let response = rpc_client.store_account(request).await?;

    if let Some(err) = response.error {
        println!("Error: {err}");
    } else {
        println!("Account recovery phrase stored");
    }

    Ok(())
}

async fn refresh_account_state(mut rpc_client: RpcClient) -> Result<()> {
    rpc_client.refresh_account_state().await?;
    Ok(())
}

async fn is_account_stored(mut rpc_client: RpcClient) -> Result<()> {
    let is_stored = rpc_client.is_account_stored().await?;
    if is_stored {
        println!("Account is stored");
    } else {
        println!("No account is stored");
    }
    Ok(())
}

async fn get_account_usage(mut rpc_client: RpcClient) -> Result<()> {
    let response = rpc_client.get_account_usage().await?;
    println!("{response:#?}");
    Ok(())
}

async fn forget_account(mut rpc_client: RpcClient) -> Result<()> {
    let response = rpc_client.forget_account().await?;
    if let Some(err) = response.error {
        println!("Error: {err}");
    } else {
        println!("Account forgotten successfully");
    }
    Ok(())
}

async fn get_account_id(mut rpc_client: RpcClient) -> Result<()> {
    let response = rpc_client.get_account_identity().await?;
    println!("{response:#?}");
    Ok(())
}

async fn get_account_links(
    mut rpc_client: RpcClient,
    args: cli::GetAccountLinksArgs,
) -> Result<()> {
    let links = rpc_client.get_account_links(args.locale).await?;
    println!("{links}");

    Ok(())
}

async fn get_account_state(mut rpc_client: RpcClient, listen: bool) -> Result<()> {
    if listen {
        let mut stream = rpc_client.listen_to_account_controller_state().await?;

        let initial_state = tokio::select! {
            initial_state = rpc_client.get_account_state() => initial_state,
            Some(initial_state) = stream.next() => initial_state
        }?;
        println!("{initial_state}");

        while let Some(new_state) = stream.next().await {
            let new_state = new_state?;
            println!("{new_state}");
        }
    } else {
        let response = rpc_client.get_account_state().await?;
        println!("{response}");
    }

    Ok(())
}

async fn reset_device_identity(
    mut rpc_client: RpcClient,
    args: cli::ResetDeviceIdentityArgs,
) -> Result<()> {
    let seed = args.seed.map(|seed| seed.into_bytes());
    rpc_client.reset_device_identity(seed).await?;
    Ok(())
}

async fn get_device_id(mut rpc_client: RpcClient) -> Result<()> {
    let response = rpc_client.get_device_identity().await?;
    println!("{response:#?}");
    Ok(())
}

async fn get_devices(mut rpc_client: RpcClient) -> Result<()> {
    let response = rpc_client.get_devices().await?;
    println!("{response:#?}");
    Ok(())
}

async fn get_active_devices(mut rpc_client: RpcClient) -> Result<()> {
    let response = rpc_client.get_active_devices().await?;
    println!("{response:#?}");
    Ok(())
}

async fn get_available_tickets(mut rpc_client: RpcClient) -> Result<()> {
    let response = rpc_client.get_available_tickets().await?;
    println!("{response:#?}");
    Ok(())
}

async fn list_gateways(
    mut rpc_client: RpcClient,
    gw_type: GatewayType,
    user_agent: UserAgent,
) -> Result<()> {
    let gateways = rpc_client
        .list_gateways(ListGatewaysOptions {
            gw_type,
            user_agent: Some(user_agent),
        })
        .await?;

    println!("Gateways available for: {gw_type}");
    println!("Total gateways: {}", gateways.len());
    for gateway in gateways {
        println!("  {gateway:?}");
    }
    Ok(())
}

async fn list_countries(
    mut rpc_client: RpcClient,
    gw_type: GatewayType,
    user_agent: UserAgent,
) -> Result<()> {
    let countries = rpc_client
        .list_countries(ListCountriesOptions {
            gw_type,
            user_agent: Some(user_agent),
        })
        .await?;

    println!(
        "Countries for {} ({}): {}",
        gw_type,
        countries.len(),
        countries
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>()
            .join(", ")
    );

    Ok(())
}

fn print_service_info(service_info: VpnServiceInfo) {
    println!("nym-vpnd:");
    println!("  version: {}", service_info.version);
    println!(
        "  build_timestamp (utc): {}",
        service_info
            .build_timestamp
            .map(|s| s.to_string())
            .unwrap_or_default()
    );
    println!("  triple: {}", service_info.triple);
    println!("  platform: {}", service_info.platform);
    println!("  git_commit: {}", service_info.git_commit);
    println!();
    println!("nym_network:");
    println!(
        "  network_name: {}",
        service_info.nym_network.network.network_name
    );
    println!("  chain_details:");
    println!(
        "    bech32_account_prefix: {}",
        service_info
            .nym_network
            .network
            .chain_details
            .bech32_account_prefix
    );
    println!("    mix_denom:");
    println!(
        "      base: {}",
        service_info
            .nym_network
            .network
            .chain_details
            .mix_denom
            .base
    );
    println!(
        "      display: {}",
        service_info
            .nym_network
            .network
            .chain_details
            .mix_denom
            .display
    );
    println!(
        "      display_exponent: {}",
        service_info
            .nym_network
            .network
            .chain_details
            .mix_denom
            .display_exponent
    );
    println!("    stake_denom:");
    println!(
        "      base: {}",
        service_info
            .nym_network
            .network
            .chain_details
            .stake_denom
            .base
    );
    println!(
        "      display: {}",
        service_info
            .nym_network
            .network
            .chain_details
            .stake_denom
            .display
    );
    println!(
        "      display_exponent: {}",
        service_info
            .nym_network
            .network
            .chain_details
            .stake_denom
            .display_exponent
    );

    println!("  validators:");
    for validator in &service_info.nym_network.network.endpoints {
        println!("    nyxd_url: {}", validator.nyxd_url);
        println!("    api_url: {}", or_not_set(&validator.api_url));
        println!(
            "    websocket_url: {}",
            or_not_set(&validator.websocket_url)
        );
    }

    println!("  nym_contracts:");
    println!(
        "    mixnet_contract_address: {}",
        or_not_set(
            &service_info
                .nym_network
                .network
                .contracts
                .mixnet_contract_address
        )
    );
    println!(
        "    vesting_contract_address: {}",
        or_not_set(
            &service_info
                .nym_network
                .network
                .contracts
                .vesting_contract_address
        )
    );
    println!(
        "    ecash_contract_address: {}",
        or_not_set(
            &service_info
                .nym_network
                .network
                .contracts
                .ecash_contract_address
        )
    );
    println!(
        "    group_contract_address: {}",
        or_not_set(
            &service_info
                .nym_network
                .network
                .contracts
                .group_contract_address
        )
    );
    println!(
        "    multisig_contract_address: {}",
        or_not_set(
            &service_info
                .nym_network
                .network
                .contracts
                .multisig_contract_address
        )
    );
    println!(
        "    coconut_dkg_contract_address: {}",
        or_not_set(
            &service_info
                .nym_network
                .network
                .contracts
                .coconut_dkg_contract_address
        )
    );
    println!();
    println!("nym_vpn_network:");
    println!(
        "  nym_vpn_api_url: {}",
        service_info.nym_vpn_network.nym_vpn_api_url
    )
}

pub fn or_not_set<T: ToString + Clone>(value: &Option<T>) -> String {
    value
        .clone()
        .map(|v| v.to_string())
        .unwrap_or("not set".to_string())
}
