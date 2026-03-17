// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;

use nym_vpn_lib_types::SplitApp;
use nym_vpn_proto::rpc_client::RpcClient;

use crate::boolean_option::BooleanOption;

#[derive(Debug, Clone, clap::Subcommand)]
pub enum Command {
    /// Get split tunnel status
    Get,

    /// Set split tunnel status
    Set {
        /// Enable or disable split tunnel
        #[arg(value_parser = BooleanOption::custom_parser("on", "off"))]
        enable: BooleanOption,
    },

    /// Add application to exclude from VPN tunnel
    AddApp {
        /// Path to executable
        path: String,
    },

    /// Remove application from VPN tunnel exclusion
    RemoveApp {
        /// Path to executable
        path: String,
    },

    /// Remove all applications from VPN tunnel exclusion
    ClearApps,

    /// List processes excluded from VPN tunnel
    ExcludedProcesses,
}

impl Command {
    pub async fn execute(self, mut rpc_client: RpcClient) -> Result<()> {
        match self {
            Command::Get => {
                let config = rpc_client.get_config().await?;
                println!(
                    "Split-tunnel: {}",
                    if config.split_tunnel.enabled {
                        "enabled"
                    } else {
                        "disabled"
                    }
                );

                #[cfg(target_os = "macos")]
                {
                    let needs_fda = rpc_client.need_full_disk_permissions().await?;
                    println!(
                        "Full disk access: {}",
                        if needs_fda { "disallowed" } else { "allowed" }
                    );
                }

                println!("Apps excluded from VPN tunnel:");
                for app in config.split_tunnel.apps {
                    println!("- {}", app.path);
                }
                Ok(())
            }
            Command::Set { enable } => {
                rpc_client.set_enable_split_tunnel(*enable).await?;
                Ok(())
            }
            Command::AddApp { path } => {
                rpc_client.add_split_tunnel_app(SplitApp::new(path)).await?;
                Ok(())
            }
            Command::RemoveApp { path } => {
                rpc_client
                    .remove_split_tunnel_app(SplitApp::new(path))
                    .await?;
                Ok(())
            }
            Command::ClearApps => {
                rpc_client.clear_split_tunnel_apps().await?;
                Ok(())
            }
            Command::ExcludedProcesses => {
                let proc_list = rpc_client.get_split_tunnel_excluded_processes().await?;

                if proc_list.processes.is_empty() {
                    println!("No excluded processes");
                } else {
                    println!("Excluded processes: {}", proc_list.processes.len());
                    println!();

                    for proc in proc_list.processes {
                        println!("- pid: {}", proc.pid);
                        println!("  path: {}", proc.exec_path.display());
                        if proc.exec_path != proc.responsible_exec_path {
                            println!(
                                "  responsible process: {}",
                                proc.responsible_exec_path.display()
                            );
                        }
                        println!();
                    }
                }

                Ok(())
            }
        }
    }
}
