// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::{Result, anyhow};
use tabled::Table;

use nym_vpn_lib_types::{
    EntryPoint, ExitPoint, GatewayFilter, GetRecentGatewaysParams, ListGatewaysOptions,
    LookupGatewayFilters, NodeIdentity, Recipient,
};
use nym_vpn_proto::rpc_client::RpcClient;

use crate::{
    boolean_option::BooleanOption, display_helpers::display_on_off, table_style::TableStyle,
};

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
    ListFiltered {
        /// Gateway type
        #[arg(value_enum)]
        gateway_type: GatewayType,

        #[command(flatten)]
        filters: FilterArgs,
    },

    /// List recently connected gateways
    Recents {
        /// Tunnel type of the recent connection to the gateways
        tunnel_type: TunnelType,
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

    /// Auto-select entry. If argument is "on", the device's jurisdiction will be excluded from seleciton.
    /// Otherwise, if the argument is "off", that jurisdiction will not be excluded.
    #[arg(long, group = "entry")]
    pub entry_auto_exclude_jurisdiction: Option<BooleanOption>,

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

    /// Auto-select entry. If argument is "on", the device's jurisdiction and the entry gateway's jurisdiction will be excluded from seleciton.
    /// Otherwise, if the argument is "off", those jurisdictions will not be excluded.
    #[arg(long, group = "exit")]
    pub exit_auto_exclude_jurisdiction: Option<BooleanOption>,

    /// Only select residential exit nodes.
    #[arg(long, value_parser = clap::value_parser!(BooleanOption))]
    pub residential_exit: Option<BooleanOption>,
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
pub enum Score {
    Low,
    Medium,
    High,
    Offline,
}

impl From<Score> for nym_vpn_lib_types::Score {
    fn from(value: Score) -> Self {
        match value {
            Score::High => Self::High,
            Score::Medium => Self::Medium,
            Score::Low => Self::Low,
            Score::Offline => Self::Offline,
        }
    }
}

#[derive(Debug, Clone, clap::Args)]
pub struct FilterArgs {
    /// Minimum score
    #[arg(long)]
    pub min_score: Option<Score>,

    /// Filter by country (two-letter ISO code)
    #[arg(long)]
    pub country: Option<String>,

    /// Filter by region/state
    #[arg(long)]
    pub region: Option<String>,

    /// Filter for residential gateways only
    #[arg(long)]
    pub residential: bool,

    /// Filter for QUIC enabled gateways only
    #[arg(long)]
    pub quic_enabled: bool,
}

impl Args {
    pub async fn execute(self, mut rpc_client: RpcClient) -> Result<()> {
        match self.command {
            Command::Get => {
                let config = rpc_client.get_config().await?;
                println!("Entry point: {:?}", config.entry_point);
                println!("Exit point: {:?}", config.exit_point);
                println!(
                    "Residential exit: {}",
                    display_on_off(config.residential_exit)
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
                self.list_gateways(rpc_client, gateway_type).await?;
                Ok(())
            }
            Command::ListFiltered {
                gateway_type,
                ref filters,
            } => {
                self.list_filtered_gateways(rpc_client, gateway_type, filters.clone())
                    .await?;
                Ok(())
            }
            Command::Recents { tunnel_type } => {
                self.recent_gateways(rpc_client, tunnel_type).await?;
                Ok(())
            }
        }
    }

    async fn list_gateways(&self, mut rpc_client: RpcClient, gw_type: GatewayType) -> Result<()> {
        let gateways = rpc_client
            .list_gateways(ListGatewaysOptions {
                gw_type: gw_type.into(),
                user_agent: None,
            })
            .await?;

        println!("Gateways available for: {gw_type:?} ({})", gateways.len());
        let models = gateways
            .into_iter()
            .map(|gateway| GatewayModel::new(gateway, gw_type))
            .collect::<Vec<_>>();
        let mut table = Table::new(models);
        self.table_style.apply_style(&mut table);
        println!("{table}");

        Ok(())
    }

    async fn list_filtered_gateways(
        &self,
        mut rpc_client: RpcClient,
        gw_type: GatewayType,
        filters: FilterArgs,
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
        let mut table = Table::new(models);
        self.table_style.apply_style(&mut table);
        println!("{table}");

        Ok(())
    }

    async fn recent_gateways(
        &self,
        mut rpc_client: RpcClient,
        tunnel_type: TunnelType,
    ) -> Result<()> {
        let recent_gateways = rpc_client
            .get_recent_gateways(GetRecentGatewaysParams {
                tunnel_type: tunnel_type.into(),
            })
            .await?;

        println!(
            "Recent entry gateways for:  {tunnel_type:?} ({})",
            recent_gateways.entry.len(),
        );
        let gw_type = match tunnel_type {
            TunnelType::Mixnet => GatewayType::MixnetEntry,
            TunnelType::Wg => GatewayType::Wg,
        };
        let models = recent_gateways
            .entry
            .into_iter()
            .map(|gateway| GatewayModel::new(gateway, gw_type))
            .collect::<Vec<_>>();
        let mut table = Table::new(models);
        self.table_style.apply_style(&mut table);
        println!("{table}");

        println!(
            "Recent exit gateways for:  {tunnel_type:?} ({})",
            recent_gateways.exit.len(),
        );
        let gw_type = match tunnel_type {
            TunnelType::Mixnet => GatewayType::MixnetExit,
            TunnelType::Wg => GatewayType::Wg,
        };
        let models = recent_gateways
            .exit
            .into_iter()
            .map(|gateway| GatewayModel::new(gateway, gw_type))
            .collect::<Vec<_>>();
        let mut table = Table::new(models);
        self.table_style.apply_style(&mut table);
        println!("{table}");

        Ok(())
    }

    fn build_filters(gw_type: GatewayType, filters: FilterArgs) -> LookupGatewayFilters {
        let mut gateway_filters = LookupGatewayFilters {
            gw_type: gw_type.into(),
            filters: Vec::new(),
        };

        if let Some(min_score) = filters.min_score.map(nym_vpn_lib_types::Score::from) {
            gateway_filters
                .filters
                .push(GatewayFilter::MinScore(min_score));
        }

        if let Some(country) = filters.country {
            gateway_filters
                .filters
                .push(GatewayFilter::Country(country));
        }

        if let Some(region) = filters.region {
            gateway_filters.filters.push(GatewayFilter::Region(region));
        }

        if filters.residential {
            gateway_filters.filters.push(GatewayFilter::Residential);
        }

        if filters.quic_enabled {
            gateway_filters.filters.push(GatewayFilter::QuicEnabled);
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
        } else if let Some(exclude_user_country) = self.entry_auto_exclude_jurisdiction {
            Ok(Some(EntryPoint::Auto {
                exclude_user_country: *exclude_user_country,
            }))
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
        } else if let Some(exclude_jurisdictions) = self.exit_auto_exclude_jurisdiction {
            Ok(Some(ExitPoint::Auto {
                exclude_entry_point_country: *exclude_jurisdictions,
                exclude_user_country: *exclude_jurisdictions,
            }))
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

impl From<GatewayType> for nym_vpn_lib_types::GatewayType {
    fn from(value: GatewayType) -> Self {
        match value {
            GatewayType::MixnetEntry => Self::MixnetEntry,
            GatewayType::MixnetExit => Self::MixnetExit,
            GatewayType::Wg => Self::Wg,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, clap::ValueEnum)]
pub enum TunnelType {
    /// Mixnet tunnel
    Mixnet,
    /// Wireguard tunnel
    Wg,
}

impl From<TunnelType> for nym_vpn_lib_types::TunnelType {
    fn from(value: TunnelType) -> Self {
        match value {
            TunnelType::Mixnet => nym_vpn_lib_types::TunnelType::Mixnet,
            TunnelType::Wg => nym_vpn_lib_types::TunnelType::Wireguard,
        }
    }
}

#[derive(tabled::Tabled)]
pub struct GatewayModel {
    #[tabled(rename = "ID")]
    pub id: String,
    #[tabled(rename = "Name", display("tabled::derive::display::wrap", 40))]
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
            name: gateway.name,
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
                    .performance
                    .map(|p| {
                        format!(
                            "{:?} (load: {:?}, uptime: {:?}%)",
                            p.mixnet_score,
                            p.load,
                            (p.uptime_percentage_last_24_hours * 100f32) as u8,
                        )
                    })
                    .unwrap_or("N/A".to_owned()),
                GatewayType::Wg => gateway
                    .performance
                    .map(|p| {
                        format!(
                            "{:?} (load: {:?}, uptime: {:?}%)",
                            p.score,
                            p.load,
                            (p.uptime_percentage_last_24_hours * 100f32) as u8,
                        )
                    })
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
