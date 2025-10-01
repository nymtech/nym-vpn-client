// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::{Result, anyhow};
use tabled::Table;

use nym_gateway_directory::{
    EntryPoint, ExitPoint, GatewayFilter, GatewayFilters, NodeIdentity, Recipient,
};
use nym_http_api_client::UserAgent;
use nym_vpn_lib_types::ListGatewaysOptions;
use nym_vpn_proto::rpc_client::RpcClient;

use crate::{boolean_option::BooleanOption, table_style::TableStyle};

#[derive(Debug, Clone, clap::Args)]
pub struct Args {
    /// Table style output.
    #[arg(global = true, long, value_enum, default_value_t)]
    pub table_style: TableStyle,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Clone, clap::Subcommand)]
pub enum Command {
    /// Get currently set gateways
    Get,

    /// Set new gateway constraints
    ///
    /// Examples:
    ///
    /// 1. Set only entry gateway, leaving exit as is
    ///
    /// nym-vpnc gateway set --entry-country US
    ///
    /// 2. Set both entry and exit gateways
    ///
    /// nym-vpnc gateway set --entry-country US --exit-country CA
    Set(Box<SetArgs>),

    /// List available gateways (mixnet-entry, mixnet-exit, wg)
    List {
        /// Gateway type
        #[arg(value_enum)]
        gateway_type: GatewayType,
    },

    /// List filtered gateways
    Filter {
        /// Gateway type
        #[arg(value_enum)]
        gateway_type: GatewayType,

        #[command(flatten)]
        filters: FilterArgs,
    },

    /// Choose a random gateway
    ChooseRandom {
        /// Gateway type
        #[arg(value_enum)]
        gateway_type: GatewayType,

        #[command(flatten)]
        filters: FilterArgs,
    },
}

#[derive(Debug, Clone, clap::Args)]
#[command(group = clap::ArgGroup::new("entry").multiple(false))]
#[command(group = clap::ArgGroup::new("exit").multiple(false))]
pub struct SetArgs {
    /// Mixnet public ID of the entry gateway.
    #[arg(long, group = "entry")]
    pub entry_id: Option<String>,

    /// Auto-select entry gateway by country ISO.
    #[arg(long, group = "entry")]
    pub entry_country: Option<celes::Country>,

    /// Auto-select entry gateway randomly.
    #[arg(long, action = clap::ArgAction::SetTrue, group = "entry")]
    pub entry_random: bool,

    /// Mixnet recipient address of the IPR connecting to, if specified directly. This is only
    /// useful when connecting to standalone IPRs.
    #[arg(long, group = "exit", hide = true)]
    pub exit_ipr_address: Option<String>,

    /// Mixnet public ID of the exit gateway.
    #[arg(long, group = "exit")]
    pub exit_id: Option<String>,

    /// Auto-select exit gateway by country ISO.
    #[arg(long, group = "exit")]
    pub exit_country: Option<celes::Country>,

    /// Auto-select exit gateway by region.
    #[arg(long, group = "exit")]
    pub exit_region: Option<String>,

    /// Auto-select exit gateway randomly.
    #[arg(long, action = clap::ArgAction::SetTrue, group = "exit")]
    pub exit_random: bool,

    /// Only select residential exit nodes.
    #[arg(long, value_parser = BooleanOption::custom_parser("on", "off"))]
    pub residential_exit: Option<BooleanOption>,
}

#[derive(Debug, Clone, clap::Args)]
pub struct FilterArgs {
    /// Minimum WireGuard performance (0-100)
    #[arg(long)]
    pub min_wg_performance: Option<u8>,

    /// Minimum mixnet performance (0-100)
    #[arg(long)]
    pub min_mixnet_performance: Option<u8>,

    /// Filter by country (two-letter ISO code)
    #[arg(long)]
    pub country: Option<String>,

    /// Filter by region/state
    #[arg(long)]
    pub region: Option<String>,

    /// Filter for residential gateways only
    #[arg(long)]
    pub residential: bool,
}

impl Args {
    pub async fn execute(self, mut rpc_client: RpcClient, user_agent: UserAgent) -> Result<()> {
        match self.command {
            Command::Get => {
                let config = rpc_client.get_config().await?;
                println!("Entry point: {}", config.entry_point);
                println!("Exit point: {}", config.exit_point);
                println!(
                    "Residential exit: {}",
                    if config.residential_exit { "on" } else { "off" }
                );
                Ok(())
            }
            Command::Set(args) => {
                let entry_point = args.entry_point()?;
                let exit_point = args.exit_point()?;

                if let Some(entry_point) = entry_point {
                    rpc_client.set_entry_point(entry_point).await?;
                }

                if let Some(exit_point) = exit_point {
                    rpc_client.set_exit_point(exit_point).await?;
                }

                if let Some(residential_exit) = args.residential_exit {
                    rpc_client.set_residential_exit(*residential_exit).await?;
                }

                Ok(())
            }
            Command::List { gateway_type } => {
                self.list_gateways(rpc_client, gateway_type, user_agent)
                    .await?;
                Ok(())
            }
            Command::Filter {
                gateway_type,
                ref filters,
            } => {
                self.list_filtered_gateways(rpc_client, gateway_type, filters)
                    .await?;
                Ok(())
            }
            Command::ChooseRandom {
                gateway_type,
                ref filters,
            } => {
                self.choose_random_gateway(rpc_client, gateway_type, filters)
                    .await?;
                Ok(())
            }
        }
    }

    async fn list_gateways(
        &self,
        mut rpc_client: RpcClient,
        gw_type: GatewayType,
        user_agent: UserAgent,
    ) -> Result<()> {
        let gateways = rpc_client
            .list_gateways(ListGatewaysOptions {
                gw_type: gw_type.into(),
                user_agent: Some(user_agent),
            })
            .await?;

        println!("Gateways available for: {gw_type:?} ({})", gateways.len());
        let models = gateways
            .into_iter()
            .map(|gateway| GatewayModel::new(gateway, gw_type))
            .collect::<Vec<_>>();
        let mut table = Table::new(models.into_iter());
        self.table_style.apply_style(&mut table);
        println!("{table}");

        Ok(())
    }

    async fn list_filtered_gateways(
        &self,
        mut rpc_client: RpcClient,
        gw_type: GatewayType,
        filters: &FilterArgs,
    ) -> Result<()> {
        let gateway_filters = Self::build_filters(gw_type, filters);

        let gateways = rpc_client.list_filtered_gateways(gateway_filters).await?;

        println!(
            "Filtered gateways available for: {gw_type:?} ({})",
            gateways.len()
        );
        let models = gateways
            .into_iter()
            .map(|gateway| GatewayModel::new(gateway, gw_type))
            .collect::<Vec<_>>();
        let mut table = Table::new(models.into_iter());
        self.table_style.apply_style(&mut table);
        println!("{table}");

        Ok(())
    }

    async fn choose_random_gateway(
        &self,
        mut rpc_client: RpcClient,
        gw_type: GatewayType,
        filters: &FilterArgs,
    ) -> Result<()> {
        let gateway_filters = Self::build_filters(gw_type, filters);

        match rpc_client.choose_random_gateway(gateway_filters).await? {
            Some(gateway) => {
                let model = GatewayModel::new(gateway, gw_type);
                let mut table = Table::new(vec![model]);
                self.table_style.apply_style(&mut table);
                println!("{table}");
            }
            None => {
                println!("No gateway found matching the specified criteria");
            }
        }

        Ok(())
    }

    fn build_filters(gw_type: GatewayType, filters: &FilterArgs) -> GatewayFilters {
        let mut gateway_filters = GatewayFilters {
            gw_type: gw_type.into(),
            filters: Vec::new(),
        };

        if filters.min_wg_performance.is_some() || filters.min_mixnet_performance.is_some() {
            gateway_filters.filters.push(GatewayFilter::MinPerformance {
                min_wg_performance: filters.min_wg_performance,
                min_mixnet_performance: filters.min_mixnet_performance,
            });
        }

        if let Some(ref country) = filters.country {
            gateway_filters
                .filters
                .push(GatewayFilter::Country(country.clone()));
        }

        if let Some(ref region) = filters.region {
            gateway_filters
                .filters
                .push(GatewayFilter::Region(region.clone()));
        }

        if filters.residential {
            gateway_filters.filters.push(GatewayFilter::Residential);
        }

        gateway_filters
    }
}

impl SetArgs {
    pub fn entry_point(&self) -> Result<Option<EntryPoint>> {
        if let Some(ref entry_gateway_id) = self.entry_id {
            Ok(Some(EntryPoint::Gateway {
                identity: NodeIdentity::from_base58_string(entry_gateway_id)
                    .map_err(|_| anyhow!("Failed to parse gateway id"))?,
            }))
        } else if let Some(ref entry_gateway_country) = self.entry_country {
            Ok(Some(EntryPoint::Country {
                two_letter_iso_country_code: entry_gateway_country.alpha2.to_string(),
            }))
        } else if self.entry_random {
            Ok(Some(EntryPoint::Random))
        } else {
            Ok(None)
        }
    }

    pub fn exit_point(&self) -> Result<Option<ExitPoint>> {
        if let Some(ref exit_router_address) = self.exit_ipr_address {
            Ok(Some(ExitPoint::Address {
                address: Box::new(
                    Recipient::try_from_base58_string(exit_router_address)
                        .map_err(|_| anyhow!("Failed to parse exit node address"))?,
                ),
            }))
        } else if let Some(ref exit_router_id) = self.exit_id {
            Ok(Some(ExitPoint::Gateway {
                identity: NodeIdentity::from_base58_string(exit_router_id.clone())
                    .map_err(|_| anyhow!("Failed to parse gateway id"))?,
            }))
        } else if let Some(ref exit_gateway_country) = self.exit_country {
            Ok(Some(ExitPoint::Country {
                two_letter_iso_country_code: exit_gateway_country.alpha2.to_string(),
            }))
        } else if let Some(ref exit_gateway_region) = self.exit_region {
            Ok(Some(ExitPoint::Region {
                region: exit_gateway_region.to_string(),
            }))
        } else if self.exit_random {
            Ok(Some(ExitPoint::Random))
        } else {
            Ok(None)
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, clap::ValueEnum)]
pub enum GatewayType {
    /// Mixnet entry gateway
    MixnetEntry,
    /// Mixnet exit gateway
    MixnetExit,
    /// Wireguard gateway
    Wg,
}

impl From<GatewayType> for nym_gateway_directory::GatewayType {
    fn from(value: GatewayType) -> Self {
        match value {
            GatewayType::MixnetEntry => Self::MixnetEntry,
            GatewayType::MixnetExit => Self::MixnetExit,
            GatewayType::Wg => Self::Wg,
        }
    }
}

#[derive(tabled::Tabled)]
pub struct GatewayModel {
    #[tabled(rename = "ID")]
    pub id: String,
    #[tabled(rename = "Name")]
    pub name: String,
    #[tabled(rename = "Location")]
    pub location: String,
    #[tabled(rename = "Performance")]
    pub performance: String,
    #[tabled(rename = "Exit IPv4")]
    pub exit_ipv4: String,
    #[tabled(rename = "Exit IPv6")]
    pub exit_ipv6: String,
    #[tabled(rename = "Build Version")]
    pub build_version: String,
}

impl GatewayModel {
    fn new(gateway: nym_vpn_lib_types::Gateway, gw_type: GatewayType) -> Self {
        Self {
            id: gateway.identity_key,
            name: gateway.moniker,
            location: gateway
                .location
                .map(|s| {
                    if s.city == s.region || s.region.contains(&s.city) {
                        format!("{} [{}]", s.city, s.two_letter_iso_country_code)
                    } else {
                        format!(
                            "{}, {} [{}]",
                            s.city, s.region, s.two_letter_iso_country_code
                        )
                    }
                })
                .unwrap_or("N/A".to_owned()),
            performance: match gw_type {
                GatewayType::MixnetEntry | GatewayType::MixnetExit => gateway
                    .mixnet_performance
                    .map(|p| format!("{p}%"))
                    .unwrap_or("N/A".to_owned()),
                GatewayType::Wg => gateway
                    .wg_performance
                    .map(|p| format!("{}%", (p.uptime_percentage_last_24_hours * 100f32) as u8))
                    .unwrap_or("N/A".to_owned()),
            },
            exit_ipv4: gateway
                .exit_ipv4s
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(","),
            exit_ipv6: gateway
                .exit_ipv6s
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(","),
            build_version: gateway.build_version.unwrap_or("N/A".to_owned()),
        }
    }
}
