// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;
use tabled::Table;

use nym_vpn_lib_types::{NymVpnDevice, NymVpnDeviceStatus};
use nym_vpn_proto::rpc_client::RpcClient;

use crate::table_style::TableStyle;

#[derive(Debug, Clone, clap::Args)]
pub struct Args {
    /// Table style output.
    #[arg(global = true, long, value_enum, default_value_t)]
    pub table_style: TableStyle,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Clone, clap::Subcommand)]
pub enum Command {
    Get,
    #[clap(hide = true)]
    List {
        #[clap(long, action = clap::ArgAction::SetTrue)]
        only_active: bool,
    },
    #[clap(hide = true)]
    Reset {
        #[arg(long)]
        seed: Option<String>,
    },
}

impl Args {
    pub async fn execute(self, mut rpc_client: RpcClient) -> Result<()> {
        match self.command.unwrap_or(Command::Get) {
            Command::Get => {
                let identity = rpc_client.get_device_identity().await?;
                if let Some(identity) = identity {
                    println!("Device identity: {identity}");
                }
                Ok(())
            }
            Command::List { only_active } => {
                let devices = if only_active {
                    rpc_client.get_active_devices().await?
                } else {
                    rpc_client.get_devices().await?
                };

                let mut table = Table::new(devices.into_iter().map(DeviceModel::from));
                self.table_style.apply_style(&mut table);

                println!("{table}");

                Ok(())
            }
            Command::Reset { seed } => {
                rpc_client
                    .reset_device_identity(seed.map(|v| v.into_bytes()))
                    .await?;
                println!("Device identity has been reset");
                Ok(())
            }
        }
    }
}

#[derive(tabled::Tabled)]
pub struct DeviceModel {
    #[tabled(rename = "Device ID")]
    pub device_identity_key: String,

    #[tabled(rename = "Created at")]
    pub created_on_utc: String,

    #[tabled(rename = "Updated at")]
    pub last_updated_utc: String,

    #[tabled(rename = "Status")]
    pub status: String,
}

impl From<NymVpnDevice> for DeviceModel {
    fn from(device: NymVpnDevice) -> Self {
        DeviceModel {
            device_identity_key: device.device_identity_key,
            created_on_utc: device.created_on_utc,
            last_updated_utc: device.last_updated_utc,
            status: match device.status {
                NymVpnDeviceStatus::Active => "Active".to_string(),
                NymVpnDeviceStatus::Inactive => "Inactive".to_string(),
                NymVpnDeviceStatus::DeleteMe => "Deleted".to_string(),
            },
        }
    }
}
