// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;
use clap::builder::ValueParserFactory;
use nym_vpn_proto::rpc_client::RpcClient;

use crate::boolean_option::BooleanOption;

#[derive(Debug, Clone, clap::Subcommand)]
pub enum Command {
    /// Get the Geo Exclusion configuration
    Get,

    /// Set Geo Exclusion configuration
    Set {
        #[command(subcommand)]
        subcommand: SetCommand,
    },
}

#[derive(Debug, Clone, clap::Subcommand)]
pub enum SetCommand {
    /// Enable or disable Geo Exclusion
    Enabled {
        /// Enable or disable Geo Exclusion
        #[arg(value_parser = BooleanOption::value_parser())]
        enable: BooleanOption,
    },

    /// Set the listen port for Geo Exclusion
    ListenPort {
        /// Port number (1–65535)
        port: u16,
    },

    /// Set the list of excluded country codes for Geo Exclusion
    ///
    /// Traffic destined for these countries will bypass the proxy.
    /// Provide zero or more two-letter uppercase ISO 3166-1 alpha-2 codes.
    /// Passing no codes clears the list.
    ///
    /// Example: nym-vpnc geoexclusion set excluded-countries CN RU
    ExcludedCountries {
        /// Two-letter uppercase ISO country codes (e.g. CN RU US)
        countries: Vec<String>,
    },
}

impl Command {
    pub async fn execute(self, mut rpc_client: RpcClient) -> Result<()> {
        match self {
            Command::Get => {
                let config = rpc_client.get_config().await?;
                let a = &config.geo_exclusion;
                println!(
                    "Geo Exclusion enabled:    {}",
                    if a.enabled { "yes" } else { "no" }
                );
                println!("Listen port:      {}", a.listen_port);
                if a.excluded_countries.is_empty() {
                    println!("Excluded countries: (none)");
                } else {
                    println!("Excluded countries: {}", a.excluded_countries.join(", "));
                }
                Ok(())
            }
            Command::Set { subcommand } => subcommand.execute(rpc_client).await,
        }
    }
}

impl SetCommand {
    pub async fn execute(self, mut rpc_client: RpcClient) -> Result<()> {
        match self {
            SetCommand::Enabled { enable } => {
                rpc_client.set_geo_exclusion_enabled(*enable).await?;
                Ok(())
            }
            SetCommand::ListenPort { port } => {
                rpc_client.set_geo_exclusion_listen_port(port).await?;
                Ok(())
            }
            SetCommand::ExcludedCountries { countries } => {
                rpc_client
                    .set_geo_exclusion_excluded_countries(countries)
                    .await?;
                Ok(())
            }
        }
    }
}
