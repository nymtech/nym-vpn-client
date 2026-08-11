// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;

use nym_vpn_lib_types::ProfileOptions;
use nym_vpn_proto::rpc_client::RpcClient;

#[derive(Debug, Clone, clap::Subcommand)]
pub enum Profile {
    /// 2 hop configuration with automatic selection with different jurisdictions.
    Safest,
    /// 5 hop configuration with automatic selection with different jurisdictions.
    MostPrivate,
    /// 2 hop configuration with automatic selection regardless of jurisdiction.
    Fastest,
    /// 2 hop configuration with random servers.
    Random,
}

impl From<Profile> for nym_vpn_lib_types::Profile {
    fn from(profile: Profile) -> Self {
        match profile {
            Profile::Safest => Self::Safest,
            Profile::MostPrivate => Self::MostPrivate,
            Profile::Fastest => Self::Fastest,
            Profile::Random => Self::Random,
        }
    }
}

#[derive(Debug, Clone, clap::Subcommand)]
pub enum Command {
    /// Set a configuration profile.
    #[command(subcommand)]
    Set(Profile),
}

impl Command {
    pub async fn execute(self, mut rpc_client: RpcClient) -> Result<()> {
        match self {
            Command::Set(profile) => rpc_client
                .set_profile(ProfileOptions {
                    profile: profile.into(),
                })
                .await
                .map_err(|err| err.into()),
        }
    }
}
