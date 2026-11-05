// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;

use nym_secure_enclave::UnlockAuthorization;
use nym_vpn_proto::rpc_client::RpcClient;

#[derive(Debug, Clone, clap::Subcommand)]
pub enum Command {
    /// Authenticate to unlock access to the account stored in the keychain
    Unlock {
        /// Reason shown in the authentication prompt
        #[arg(long, default_value = "access your NymVPN account")]
        reason: String,
    },

    /// Revoke the authentication obtained with unlock
    Lock,
}

impl Command {
    pub async fn execute(self, mut rpc_client: RpcClient) -> Result<()> {
        match self {
            Command::Unlock { reason } => {
                // Kept alive until the daemon has verified the token.
                let authorization = UnlockAuthorization::request(&reason)?;
                rpc_client.unlock_keychain(authorization.token()?).await?;
                println!("Keychain unlocked");
                Ok(())
            }
            Command::Lock => {
                rpc_client.lock_keychain().await?;
                println!("Keychain locked");
                Ok(())
            }
        }
    }
}
