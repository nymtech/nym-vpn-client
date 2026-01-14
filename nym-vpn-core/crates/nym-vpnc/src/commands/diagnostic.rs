// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;

use nym_diagnostic::cli::Command;
use nym_vpn_proto::rpc_client::RpcClient;

// This subcommand is a bit different from the others because the CLI is shared with nym-diagnostic binary

pub(crate) async fn execute(subcommand: Command, mut rpc_client: RpcClient) -> Result<()> {
    match subcommand {
        Command::Run(params) => {
            let report = rpc_client.run_diagnostic(params.into()).await?;
            println!("{}", serde_json::to_string_pretty(&report)?);
            Ok(())
        }
        Command::Register(params) => {
            let report = rpc_client.register_diagnostic(params.into()).await?;
            println!("{}", serde_json::to_string_pretty(&report)?);
            Ok(())
        }
    }
}
