// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;

use nym_vpn_proto::rpc_client::RpcClient;

#[derive(Debug, Clone, clap::Subcommand)]
pub enum Command {
    /// Start pairing this account via QR code
    Start,
    /// Start pairing to a remote account via a public key scanned from a QR code
    Pair {
        /// Remote QR public key
        #[arg(long)]
        pub_key: String,
    },
    /// Confirm that the pairing PIN matches on both devices, sharing this account with the other device
    Verified {
        /// The session id returned by the scanned key call
        #[arg(long)]
        session_id: u32,
    },
    /// Cancel the running pairing process
    Cancel,
}

impl Command {
    pub async fn execute(self, mut rpc_client: RpcClient) -> Result<()> {
        match self {
            Command::Start => {
                let response = rpc_client.start_pairing().await?;
                println!(
                    "Pairing process {} started with URL: {}",
                    response.session_id, response.url
                );
                Ok(())
            }
            Command::Pair { pub_key } => {
                let response = rpc_client.scanned_key(pub_key).await?;
                println!(
                    "Pairing process {} needs confirmation for PIN: {}",
                    response.session_id, response.pin_code
                );
                Ok(())
            }
            Command::Verified { session_id } => {
                rpc_client.verified_pairing_pin(session_id).await?;
                println!("Pairing PIN verified, the account is shared with the other device");
                Ok(())
            }
            Command::Cancel => {
                rpc_client.cancel_pairing_process().await?;
                println!("Pairing process cancelled");
                Ok(())
            }
        }
    }
}
