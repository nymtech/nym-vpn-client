// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use anyhow::Result;

use nym_authorization_service::Authorization;
use nym_vpn_proto::rpc_client::RpcClient;

#[derive(Debug, Clone, clap::Args)]
pub struct AuthorizeArgs {
    /// Reason shown in the authentication prompt
    #[arg(long, default_value = "Access your NymVPN account via nym-vpnc")]
    reason: String,
}

pub async fn authorize(mut rpc_client: RpcClient, args: AuthorizeArgs) -> Result<()> {
    // Kept alive until the daemon has verified the token.
    let authorization = Authorization::request(&args.reason)?;
    rpc_client.authorize(authorization.token()?).await?;
    println!("Authorized");
    Ok(())
}

pub async fn deauthorize(mut rpc_client: RpcClient) -> Result<()> {
    rpc_client.deauthorize().await?;
    println!("Deauthorized");
    Ok(())
}
