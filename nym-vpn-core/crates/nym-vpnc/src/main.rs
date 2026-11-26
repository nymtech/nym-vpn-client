// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

mod boolean_option;
mod commands;
mod display_helpers;
mod fs;
mod table_style;

use std::io::Write;

use anyhow::{Context, Result, bail};
use clap::{ArgAction, CommandFactory, Parser};
use clap_complete::Shell;
use tokio_stream::{Stream, StreamExt};

use nym_vpn_lib_types::{
    ApiUrl, DenomDetailsOwned, PairingResult, TunnelEvent, TunnelState, VpnServiceInfo,
};
use nym_vpn_proto::rpc_client::RpcClient;

use crate::{
    display_helpers::{Rows, display_date_time},
    table_style::TableStyle,
};

#[tokio::main]
async fn main() -> Result<()> {
    ProgramArgs::parse().command.execute().await
}

#[derive(Parser, Debug)]
#[clap(version, about, after_long_help = TableStyle::env_help())]
pub struct ProgramArgs {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(clap::Subcommand, Debug)]
pub enum Command {
    /// Create an interactive session to issue commands with a one time, initial authentication
    StartSession,

    /// Exit a started interactive session
    #[clap(alias = "exit")]
    ExitSession,

    /// Connect the tunnel
    #[clap(alias = "connect-v2")]
    Connect {
        /// Blocks until the connection is established or failed
        #[arg(short, long)]
        wait: bool,
    },

    /// Reconnect the tunnel to any matching gateway
    Reconnect {
        /// Blocks until the connection is re-established or failed
        #[arg(short, long)]
        wait: bool,
    },

    /// Disconnect the tunnel
    Disconnect {
        /// Blocks until disconnected.
        #[arg(short, long, default_value = "false", action = ArgAction::SetTrue)]
        wait: bool,
    },

    /// Get the current connection status
    Status {
        /// Monitor tunnel state continuously until ctrl+c.
        #[arg(short, long, default_value = "false", action = ArgAction::SetTrue)]
        listen: bool,
    },

    /// Get info about the current client. Things like version and network details.
    Info,

    /// Manage entry and exit gateway nodes, list available gateways
    Gateway(commands::gateway::Args),

    /// Local network policy
    Lan {
        #[command(subcommand)]
        subcommand: commands::lan::Command,
    },

    /// Ad blocking
    #[clap(alias = "adblock")]
    AdBlock {
        #[command(subcommand)]
        subcommand: commands::ad_block::Command,
    },

    /// Connect automatically when the daemon starts
    AutoConnect {
        #[command(subcommand)]
        subcommand: commands::auto_connect::Command,
    },

    /// Default and Custom DNS settings
    Dns {
        #[command(subcommand)]
        subcommand: commands::dns::Command,
    },

    /// Tunnel configuration (enable or disable ipv6, two-hop mode, bridge mode)
    Tunnel {
        #[command(subcommand)]
        subcommand: commands::tunnel::Command,
    },

    /// Account information
    Account {
        #[command(subcommand)]
        subcommand: commands::account::Command,
    },

    /// Device information
    Device(commands::device::Args),

    /// Nym network configuration
    Network {
        #[command(subcommand)]
        subcommand: commands::network::Command,
    },

    /// Authenticate to authorize account operations (macOS)
    #[cfg(target_os = "macos")]
    Authorize(commands::authorization::AuthorizeArgs),

    /// Revoke the authorization obtained with authorize (macOS)
    #[cfg(target_os = "macos")]
    Deauthorize,

    /// Sentry integration
    Sentry {
        #[command(subcommand)]
        subcommand: commands::sentry::Command,
    },

    /// SOCKS5 proxy
    Socks5 {
        #[command(subcommand)]
        subcommand: commands::socks5::Command,
    },

    /// Geo exclusion configuration
    GeoExclusion {
        #[command(subcommand)]
        subcommand: commands::geo_exclusion::Command,
    },

    /// Anonymous network statistics collection
    NetworkStats {
        #[command(subcommand)]
        subcommand: commands::network_stats::Command,
    },

    /// Diagnostic tool
    Diagnostic {
        #[command(subcommand)]
        subcommand: nym_diagnostic::cli::Command,
    },

    /// Favorites management
    Favorites {
        #[command(subcommand)]
        subcommand: commands::favorites::Command,
    },

    /// Split tunneling
    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    SplitTunnel {
        #[command(subcommand)]
        subcommand: commands::split_tunnel::Command,
    },

    /// Profiles management.
    Profile {
        #[command(subcommand)]
        subcommand: commands::profile::Command,
    },

    /// Generate shell completions and print them to stdout
    Completions {
        /// Target shell
        #[arg(value_enum)]
        shell: Shell,
    },
}

impl Command {
    pub async fn execute(self) -> Result<()> {
        let rpc_client = async {
            RpcClient::new()
                .await
                .context("Failed to create RPC client")
        };

        match self {
            Command::StartSession => commands::session::execute(rpc_client.await?).await,
            Command::ExitSession => Ok(()),
            Command::Connect { wait } => Self::connect(rpc_client.await?, wait).await,
            Command::Reconnect { wait } => Self::reconnect(rpc_client.await?, wait).await,
            Command::Disconnect { wait } => Self::disconnect(rpc_client.await?, wait).await,
            Command::Status { listen } => Self::status(rpc_client.await?, listen).await,
            Command::Info => Self::info(rpc_client.await?).await,
            Command::Gateway(args) => args.execute(rpc_client.await?).await,
            Command::Tunnel { subcommand } => subcommand.execute(rpc_client.await?).await,
            Command::Lan { subcommand } => subcommand.execute(rpc_client.await?).await,
            Command::AdBlock { subcommand } => subcommand.execute(rpc_client.await?).await,
            Command::AutoConnect { subcommand } => subcommand.execute(rpc_client.await?).await,
            Command::Dns { subcommand } => subcommand.execute(rpc_client.await?).await,
            Command::Network { subcommand } => subcommand.execute(rpc_client.await?).await,
            Command::Account { subcommand } => subcommand.execute(rpc_client.await?).await,
            Command::Device(args) => args.execute(rpc_client.await?).await,
            #[cfg(target_os = "macos")]
            Command::Authorize(args) => {
                commands::authorization::authorize(rpc_client.await?, args).await
            }
            #[cfg(target_os = "macos")]
            Command::Deauthorize => commands::authorization::deauthorize(rpc_client.await?).await,
            Command::Sentry { subcommand } => subcommand.execute(rpc_client.await?).await,
            Command::Socks5 { subcommand } => subcommand.execute(rpc_client.await?).await,
            Command::GeoExclusion { subcommand } => subcommand.execute(rpc_client.await?).await,
            Command::NetworkStats { subcommand } => subcommand.execute(rpc_client.await?).await,
            Command::Diagnostic { subcommand } => {
                commands::diagnostic::execute(subcommand, rpc_client.await?).await
            }
            Command::Favorites { subcommand } => subcommand.execute().await,
            #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
            Command::SplitTunnel { subcommand } => subcommand.execute(rpc_client.await?).await,
            Command::Profile { subcommand } => subcommand.execute(rpc_client.await?).await,
            Command::Completions { shell } => Self::completions(shell),
        }
    }

    fn completions(shell: Shell) -> Result<()> {
        let mut command = ProgramArgs::command();
        let bin_name = command.get_name().to_owned();
        let mut script = Vec::new();
        clap_complete::generate(shell, &mut command, bin_name, &mut script);
        std::io::stdout()
            .write_all(&script)
            .context("Failed to write completions")
    }

    async fn connect(mut rpc_client: RpcClient, wait: bool) -> Result<()> {
        let stream = if wait {
            Some(rpc_client.listen_to_events().await?)
        } else {
            None
        };

        let accepted = rpc_client.connect_tunnel().await?;

        match stream {
            Some(stream) => {
                let current_state = if accepted {
                    None
                } else {
                    Some(rpc_client.get_tunnel_state().await?)
                };

                if let Some(state) = &current_state {
                    println!("State: {state}");
                }

                if matches!(current_state, Some(TunnelState::Connected { .. })) {
                    Ok(())
                } else {
                    println!("Waiting for the tunnel to connect...");
                    Self::wait_until_connected(stream).await
                }
            }
            None => Ok(()),
        }
    }

    async fn reconnect(mut rpc_client: RpcClient, wait: bool) -> Result<()> {
        let stream = if wait {
            Some(rpc_client.listen_to_events().await?)
        } else {
            None
        };

        if !rpc_client.reconnect_tunnel().await? {
            bail!("Reconnect was rejected because the tunnel is not set to connect");
        }

        match stream {
            Some(stream) => {
                println!("State: {}", rpc_client.get_tunnel_state().await?);
                println!("Waiting for the tunnel to reconnect...");
                Self::wait_until_connected(stream).await
            }
            None => Ok(()),
        }
    }

    async fn wait_until_connected(
        mut stream: impl Stream<Item = nym_vpn_proto::rpc_client::Result<TunnelEvent>> + Unpin,
    ) -> Result<()> {
        loop {
            let Some(event) = stream.next().await else {
                bail!("Event stream ended before the tunnel connected");
            };
            let TunnelEvent::NewState(new_state) = event? else {
                continue;
            };
            println!("State: {new_state}");

            match new_state {
                TunnelState::Connected { .. } => {
                    break Ok(());
                }
                TunnelState::Offline { reconnect } => {
                    if reconnect {
                        println!("Offline, waiting for network connectivity...");
                    } else {
                        bail!("Device is offline");
                    }
                }
                TunnelState::Error(reason) => {
                    bail!("Tunnel entered error state: {reason}");
                }
                _ => {}
            }
        }
    }

    async fn disconnect(mut rpc_client: RpcClient, wait: bool) -> Result<()> {
        let stream = if wait {
            Some(rpc_client.listen_to_events().await?)
        } else {
            None
        };

        rpc_client.disconnect_tunnel().await?;

        match stream {
            Some(stream) => {
                let current_state = rpc_client.get_tunnel_state().await?;
                println!("State: {current_state}");

                if matches!(
                    current_state,
                    TunnelState::Disconnected | TunnelState::Offline { .. }
                ) {
                    Ok(())
                } else {
                    println!("Waiting for the tunnel to disconnect...");
                    Self::wait_until_disconnected(stream).await
                }
            }
            None => Ok(()),
        }
    }

    async fn wait_until_disconnected(
        mut stream: impl Stream<Item = nym_vpn_proto::rpc_client::Result<TunnelEvent>> + Unpin,
    ) -> Result<()> {
        loop {
            let Some(event) = stream.next().await else {
                bail!("Event stream ended before the tunnel disconnected");
            };
            let TunnelEvent::NewState(new_state) = event? else {
                continue;
            };
            println!("State: {new_state}");

            match new_state {
                TunnelState::Disconnected | TunnelState::Offline { .. } => break Ok(()),
                TunnelState::Error(reason) => bail!("Tunnel entered error state: {reason}"),
                _ => {}
            }
        }
    }

    async fn status(mut rpc_client: RpcClient, listen: bool) -> Result<()> {
        let state = rpc_client.get_tunnel_state().await?;
        println!("State: {state}");

        if !listen {
            return Ok(());
        }

        let mut stream = rpc_client.listen_to_events().await?;

        while let Some(event) = stream.next().await {
            match event {
                Ok(TunnelEvent::NewState(new_state)) => {
                    println!("State: {new_state}");
                }
                Ok(TunnelEvent::ConfigChanged(new_config)) => {
                    let json = serde_json::to_string_pretty(&new_config)
                        .context("failed to convert new config to JSON")?;
                    println!("Configuration changed:\n{json}");
                }
                Ok(TunnelEvent::Pairing(event)) => {
                    let session_id = event.session_id;
                    // The Display impl hides the secrets, but pairing from the CLI needs them
                    match event.result {
                        PairingResult::PinCode(pin_code) => println!(
                            "Pairing session {session_id}: check that the other device shows PIN {pin_code}"
                        ),
                        PairingResult::StoredMnemonic => {
                            println!(
                                "Pairing session {session_id}: received and stored the account mnemonic"
                            )
                        }
                        result => println!("Pairing session {session_id}: {result}"),
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }

    async fn info(mut rpc_client: RpcClient) -> Result<()> {
        let service_info = rpc_client.get_info().await?;
        Self::print_service_info(service_info);
        Ok(())
    }

    fn print_service_info(service_info: VpnServiceInfo) {
        let network = &service_info.nym_network;
        let chain = &network.chain_details;
        let contracts = &network.contracts;

        let mut rows = Rows::new();
        rows.section("Daemon")
            .row("  Version", &service_info.version)
            .row(
                "  Built",
                service_info
                    .build_timestamp
                    .map_or_else(|| "unknown".to_owned(), display_date_time),
            )
            .row("  Target", &service_info.triple)
            .row("  Platform", &service_info.platform)
            .row("  Git commit", &service_info.git_commit)
            .blank()
            .section("Nym network")
            .row("  Name", &network.network_name)
            .row("  Account prefix", &chain.bech32_account_prefix)
            .row("  Mix denom", display_denom(&chain.mix_denom))
            .row("  Stake denom", display_denom(&chain.stake_denom))
            .list(
                "  Validators",
                network.endpoints.iter().map(|validator| {
                    format!(
                        "{} (API: {}, websocket: {})",
                        validator.nyxd_url,
                        or_none(&validator.api_url),
                        or_none(&validator.websocket_url)
                    )
                }),
            )
            .section("  Contracts")
            .row("    Mixnet", or_none(&contracts.mixnet_contract_address))
            .row("    Vesting", or_none(&contracts.vesting_contract_address))
            .row(
                "    Performance",
                or_none(&contracts.performance_contract_address),
            )
            .row("    Ecash", or_none(&contracts.ecash_contract_address))
            .row("    Group", or_none(&contracts.group_contract_address))
            .row(
                "    Multisig",
                or_none(&contracts.multisig_contract_address),
            )
            .row(
                "    Coconut DKG",
                or_none(&contracts.coconut_dkg_contract_address),
            )
            .list(
                "  Nym API URLs",
                network.networking.nym_api_urls.iter().map(display_api_url),
            )
            .list(
                "  NymVPN API URLs",
                network
                    .networking
                    .nym_vpn_api_urls
                    .iter()
                    .map(display_api_url),
            )
            .blank()
            .section("NymVPN network")
            .list(
                "  NymVPN API URLs",
                service_info
                    .nym_vpn_network
                    .nym_vpn_api_urls
                    .iter()
                    .map(display_api_url),
            )
            .print();
    }
}

fn or_none(value: &Option<String>) -> &str {
    value.as_deref().unwrap_or("none")
}

fn display_denom(denom: &DenomDetailsOwned) -> String {
    format!(
        "{} (display: {}, exponent: {})",
        denom.base, denom.display, denom.display_exponent
    )
}

fn display_api_url(api_url: &ApiUrl) -> String {
    match api_url.front_hosts.as_deref() {
        Some(front_hosts) if !front_hosts.is_empty() => {
            format!("{} (fronts: {})", api_url.url, front_hosts.join(", "))
        }
        _ => api_url.url.clone(),
    }
}
