// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;
#[cfg(any(target_os = "macos", target_os = "windows"))]
use clap::builder::ValueParserFactory;

#[cfg(any(target_os = "macos", target_os = "windows"))]
use nym_vpn_lib_types::SplitApp;
use nym_vpn_proto::rpc_client::RpcClient;

#[cfg(any(target_os = "macos", target_os = "windows"))]
use crate::boolean_option::BooleanOption;

#[derive(Debug, Clone, clap::Subcommand)]
pub enum Command {
    /// Get split tunnel status
    Get,

    /// Set split tunnel status
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    Set {
        /// Enable or disable split tunnel
        #[arg(value_parser = BooleanOption::value_parser())]
        enable: BooleanOption,
    },

    /// Add application to exclude from VPN tunnel
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    AddApp {
        /// Path to executable
        path: String,
    },

    /// Remove application from VPN tunnel exclusion
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    RemoveApp {
        /// Path to executable
        path: String,
    },

    /// Remove all applications from VPN tunnel exclusion
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    ClearApps,

    /// Add process to exclusions
    ///
    /// Consider using `nym-exclude` cli tool instead to launch applications outside of tunnel.
    #[cfg(target_os = "linux")]
    AddProcess { pid: i32 },

    /// Remove process from exclusions
    #[cfg(target_os = "linux")]
    RemoveProcess { pid: i32 },

    /// Remove all processes from exclusion
    #[cfg(target_os = "linux")]
    ClearProcesses,

    /// List processes excluded from VPN tunnel
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    ExcludedProcesses,
}

impl Command {
    pub async fn execute(self, mut rpc_client: RpcClient) -> Result<()> {
        match self {
            Command::Get => {
                let is_supported = rpc_client.is_split_tunnel_supported().await?;
                println!(
                    "Split-tunnel is {}",
                    if is_supported {
                        "supported"
                    } else {
                        "not supported"
                    }
                );

                #[cfg(any(target_os = "macos", target_os = "windows"))]
                let config = rpc_client.get_config().await?;
                #[cfg(any(target_os = "macos", target_os = "windows"))]
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

                #[cfg(any(target_os = "macos", target_os = "windows"))]
                {
                    println!("Apps excluded from VPN tunnel:");
                    for app in config.split_tunnel.apps {
                        println!("- {}", app.path);
                    }
                }

                Ok(())
            }
            #[cfg(any(target_os = "macos", target_os = "windows"))]
            Command::Set { enable } => {
                rpc_client.set_enable_split_tunnel(*enable).await?;
                Ok(())
            }
            #[cfg(any(target_os = "macos", target_os = "windows"))]
            Command::AddApp { path } => {
                rpc_client.add_split_tunnel_app(SplitApp::new(path)).await?;
                Ok(())
            }
            #[cfg(any(target_os = "macos", target_os = "windows"))]
            Command::RemoveApp { path } => {
                rpc_client
                    .remove_split_tunnel_app(SplitApp::new(path))
                    .await?;
                Ok(())
            }
            #[cfg(any(target_os = "macos", target_os = "windows"))]
            Command::ClearApps => {
                rpc_client.clear_split_tunnel_apps().await?;
                Ok(())
            }
            #[cfg(target_os = "linux")]
            Command::AddProcess { pid } => {
                rpc_client.add_split_tunnel_process(pid).await?;
                Ok(())
            }
            #[cfg(target_os = "linux")]
            Command::RemoveProcess { pid } => {
                rpc_client.remove_split_tunnel_process(pid).await?;
                Ok(())
            }
            #[cfg(target_os = "linux")]
            Command::ClearProcesses => {
                rpc_client.clear_split_tunnel_processes().await?;
                Ok(())
            }
            #[cfg(any(target_os = "linux", target_os = "macos"))]
            Command::ExcludedProcesses => {
                let proc_list = rpc_client.get_split_tunnel_excluded_processes().await?;

                if proc_list.processes.is_empty() {
                    println!("No excluded processes");
                } else {
                    println!("Excluded processes: {}", proc_list.processes.len());
                    println!();

                    for proc in proc_list.processes {
                        println!("- pid: {}", proc.pid);

                        #[cfg(target_os = "macos")]
                        {
                            println!("  path: {}", proc.exec_path.display());
                            if proc.exec_path != proc.responsible_exec_path {
                                println!(
                                    "  responsible process: {}",
                                    proc.responsible_exec_path.display()
                                );
                                println!();
                            }
                        }
                    }
                }

                Ok(())
            }
        }
    }
}
