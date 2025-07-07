// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

mod cli;

use anyhow::{Context, Result, bail};
use clap::Parser;
use cli::Internal;
use itertools::Itertools;
use nym_gateway_directory::GatewayType;
use nym_http_api_client::UserAgent;
use nym_vpn_lib_types::TunnelState;
use nym_vpn_network_config::ParsedAccountLinks;
use nym_vpn_proto::rpc_client::RpcClient;
use nym_vpnd_types::{
    ConnectArgs, ConnectOptions, ListCountriesOptions, ListGatewaysOptions, StoreAccountRequest,
    service::VpnServiceInfo,
};
use sysinfo::System;
use tokio_stream::StreamExt;

use crate::cli::Command;

#[derive(Clone, Debug)]
struct CliOptions {
    verbose: bool,
    user_agent: Option<UserAgent>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = cli::CliArgs::parse();
    let opts = CliOptions {
        verbose: args.verbose,
        user_agent: args.user_agent,
    };

    let rpc_client = RpcClient::new()
        .await
        .context("Failed to create RPC client")?;

    match args.command {
        Command::Connect(connect_args) => connect(rpc_client, connect_args).await?,
        Command::Disconnect { wait } => disconnect(rpc_client, wait).await?,
        Command::Status { listen } => status(rpc_client, listen).await?,
        Command::Info => info(rpc_client).await?,
        Command::SetNetwork(args) => set_network(rpc_client, args).await?,
        Command::StoreAccount(store_args) => store_account(rpc_client, store_args).await?,
        Command::IsAccountStored => is_account_stored(rpc_client).await?,
        Command::ForgetAccount => forget_account(rpc_client).await?,
        Command::GetAccountId => get_account_id(rpc_client).await?,
        Command::GetAccountLinks(args) => get_account_links(rpc_client, args).await?,
        Command::GetAccountState => get_account_state(rpc_client).await?,
        Command::ListEntryGateways => list_gateways(rpc_client, GatewayType::MixnetEntry).await?,
        Command::ListExitGateways => list_gateways(rpc_client, GatewayType::MixnetExit).await?,
        Command::ListVpnGateways => list_gateways(rpc_client, GatewayType::Wg).await?,
        Command::ListEntryCountries => list_countries(rpc_client, GatewayType::MixnetEntry).await?,
        Command::ListExitCountries => list_countries(rpc_client, GatewayType::MixnetExit).await?,
        Command::ListVpnCountries => list_countries(rpc_client, GatewayType::Wg).await?,
        Command::GetDeviceId => get_device_id(rpc_client).await?,
        Command::Internal(internal) => match internal {
            Internal::GetSystemMessages => get_system_messages(rpc_client).await?,
            Internal::GetFeatureFlags => get_feature_flags(rpc_client).await?,
            Internal::SyncAccountState => refresh_account_state(rpc_client).await?,
            Internal::GetAccountUsage => get_account_usage(rpc_client).await?,
            Internal::ResetDeviceIdentity(args) => reset_device_identity(rpc_client, args).await?,
            Internal::RegisterDevice => register_device(rpc_client).await?,
            Internal::GetDevices => get_devices(rpc_client).await?,
            Internal::GetActiveDevices => get_active_devices(rpc_client).await?,
            Internal::RequestZkNym => request_zk_nym(rpc_client).await?,
            Internal::GetDeviceZkNym => get_device_zk_nym(rpc_client).await?,
            Internal::GetZkNymsAvailableForDownload => {
                get_zk_nyms_available_for_download(rpc_client).await?
            }
            Internal::GetZkNymById(args) => get_zk_nym_by_id(rpc_client, args).await?,
            Internal::ConfirmZkNymDownloaded(args) => {
                confirm_zk_nym_downloaded(rpc_client, args).await?
            }
            Internal::GetAvailableTickets => get_available_tickets(rpc_client).await?,
        },
    }
    Ok(())
}

fn setup_user_agent(opts: &CliOptions, daemon_info: VpnServiceInfo) -> UserAgent {
    opts.user_agent
        .clone()
        .map(UserAgent::from)
        .unwrap_or_else(|| construct_user_agent(daemon_info))
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

async fn connect(mut rpc_client: RpcClient, connect_args: cli::ConnectArgs) -> Result<()> {
    let entry = cli::parse_entry_point(&connect_args)?;
    let exit = cli::parse_exit_point(&connect_args)?;

    let user_agent = None; // todo!

    let options = ConnectArgs {
        entry,
        exit,
        options: ConnectOptions {
            dns: connect_args.dns,
            enable_two_hop: connect_args.enable_two_hop,
            netstack: connect_args.netstack,
            disable_poisson_rate: connect_args.disable_poisson_rate,
            disable_background_cover_traffic: connect_args.disable_background_cover_traffic,
            enable_credentials_mode: connect_args.enable_credentials_mode,
            min_gateway_mixnet_performance: None,
            min_mixnode_performance: None,
            min_gateway_vpn_performance: None,
            user_agent,
        },
    };

    let accepted = rpc_client.connect_tunnel(options).await?;

    if accepted && connect_args.wait {
        println!("Waiting until connected or failed");
        wait_until_connected(rpc_client).await
    } else {
        Ok(())
    }
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
    let accepted = rpc_client.disconnect_tunnel().await?;

    if accepted && wait {
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

        while let Some(new_state) = stream.next().await {
            let new_state = new_state?;
            println!("{new_state}");
        }
    } else {
        let new_state = TunnelState::try_from(rpc_client.get_tunnel_state().await?)?;
        println!("{new_state}");
    }

    Ok(())
}

async fn info(mut rpc_client: RpcClient) -> Result<()> {
    let service_info = rpc_client.get_info().await?;
    println!("{service_info:?}");
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
    let response = rpc_client.refresh_account_state().await?;
    println!("{response:#?}");
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
    let response = rpc_client.get_account_links(args.locale).await?;
    let links = ParsedAccountLinks::try_from(response)?;
    println!("{links}");

    Ok(())
}

async fn get_account_state(mut rpc_client: RpcClient) -> Result<()> {
    let response = rpc_client.get_account_state().await?;
    println!("{response:#?}");
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

async fn register_device(mut rpc_client: RpcClient) -> Result<()> {
    let response = rpc_client.register_device().await?;
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

async fn request_zk_nym(mut rpc_client: RpcClient) -> Result<()> {
    let response = rpc_client.request_zk_nym().await?;
    println!("{response:#?}");
    Ok(())
}

async fn get_device_zk_nym(mut rpc_client: RpcClient) -> Result<()> {
    let response = rpc_client.get_device_zk_nyms().await?;
    println!("{response:#?}");
    Ok(())
}

async fn get_zk_nyms_available_for_download(mut rpc_client: RpcClient) -> Result<()> {
    let response = rpc_client.get_zk_nyms_available_for_download().await?;
    println!("{response:#?}");
    Ok(())
}

async fn get_zk_nym_by_id(mut rpc_client: RpcClient, args: cli::GetZkNymByIdArgs) -> Result<()> {
    let response = rpc_client.get_zk_nym_by_id(args.id).await?;
    println!("{response:#?}");
    Ok(())
}

async fn confirm_zk_nym_downloaded(
    mut rpc_client: RpcClient,
    args: cli::ConfirmZkNymDownloadedArgs,
) -> Result<()> {
    let response = rpc_client.confirm_zk_nym_downloaded(args.id).await?;
    println!("{response:#?}");
    Ok(())
}

async fn get_available_tickets(mut rpc_client: RpcClient) -> Result<()> {
    let response = rpc_client.get_available_tickets().await?.into_inner();
    println!("{response:#?}");
    Ok(())
}

async fn list_gateways(mut rpc_client: RpcClient, gw_type: GatewayType) -> Result<()> {
    let user_agent = None; // todo!

    let gateways = rpc_client
        .list_gateways(ListGatewaysOptions {
            gw_type,
            user_agent,
        })
        .await?;

    println!("Gateways available for: {gw_type}");
    println!("Total gateways: {}", gateways.len());
    for gateway in gateways {
        println!("  {gateway}");
    }
    Ok(())
}

async fn list_countries(mut rpc_client: RpcClient, gw_type: GatewayType) -> Result<()> {
    let user_agent = None; // todo!

    let countries = rpc_client
        .list_countries(ListCountriesOptions {
            gw_type,
            user_agent,
        })
        .await?;

    println!(
        "Countries for {} ({}): {}",
        gw_type,
        countries.len(),
        countries.iter().join(", ")
    );

    Ok(())
}
