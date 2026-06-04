// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::time::Duration;

use anyhow::{Result, anyhow};
use tokio::time::timeout;
use tokio_stream::StreamExt;

use nym_vpn_lib_types::{
    EntryPoint, ExitPoint, GatewayType, ListGatewaysOptions, NodeIdentity, TunnelEvent, TunnelState,
};
use nym_vpn_proto::rpc_client::RpcClient;

const DISCONNECT_TIMEOUT_SECS: u64 = 30;

#[derive(Debug, Clone, clap::Args)]
pub struct Args {
    /// Identity key of the entry gateway to use for all connection attempts.
    #[arg(long)]
    pub entry_id: String,
}

struct Outcome {
    exit_id: String,
    exit_name: String,
    exit_location: String,
    success: bool,
    reason: String,
}

impl Args {
    pub async fn execute(self, mut rpc_client: RpcClient) -> Result<()> {
        let entry_identity = NodeIdentity::from_base58_string(&self.entry_id)
            .map_err(|_| anyhow!("Failed to parse entry gateway id"))?;

        rpc_client
            .set_entry_point(EntryPoint::Gateway {
                identity: entry_identity,
            })
            .await?;

        let mut gateways = rpc_client
            .list_gateways(ListGatewaysOptions {
                gw_type: GatewayType::Wg,
                user_agent: None,
            })
            .await?
            .into_iter()
            .filter(|g| g.identity_key != self.entry_id)
            .collect::<Vec<_>>();

        gateways.sort_by(|a, b| a.name.cmp(&b.name));

        println!("Entry gateway : {}", self.entry_id);
        println!("WG exit nodes : {}", gateways.len());
        println!();

        let mut outcomes: Vec<Outcome> = Vec::with_capacity(gateways.len());

        for (i, gateway) in gateways.iter().enumerate() {
            let exit_location = gateway
                .location
                .as_ref()
                .map(|l| {
                    if l.city == l.region || l.region.contains(&l.city) {
                        format!("{} [{}]", l.city, l.two_letter_iso_country_code)
                    } else {
                        format!(
                            "{}, {} [{}]",
                            l.city, l.region, l.two_letter_iso_country_code
                        )
                    }
                })
                .unwrap_or_else(|| "N/A".to_string());

            print!(
                "[{:>3}/{:>3}] {} ({}) {} ... ",
                i + 1,
                gateways.len(),
                gateway.identity_key,
                gateway.name,
                exit_location,
            );

            let exit_identity = match NodeIdentity::from_base58_string(&gateway.identity_key) {
                Ok(id) => id,
                Err(_) => {
                    println!("SKIP (invalid identity key)");
                    outcomes.push(Outcome {
                        exit_id: gateway.identity_key.clone(),
                        exit_name: gateway.name.clone(),
                        exit_location,
                        success: false,
                        reason: "invalid identity key".to_string(),
                    });
                    continue;
                }
            };

            if let Err(e) = rpc_client
                .set_exit_point(ExitPoint::Gateway {
                    identity: exit_identity,
                })
                .await
            {
                println!("SKIP (set exit failed: {e})");
                outcomes.push(Outcome {
                    exit_id: gateway.identity_key.clone(),
                    exit_name: gateway.name.clone(),
                    exit_location,
                    success: false,
                    reason: format!("set exit failed: {e}"),
                });
                continue;
            }

            let outcome = attempt_connection(rpc_client.clone()).await;

            let (success, reason) = match outcome {
                Ok(()) => {
                    println!("OK");
                    (true, String::new())
                }
                Err(e) => {
                    println!("FAIL ({e})");
                    (false, e.to_string())
                }
            };

            outcomes.push(Outcome {
                exit_id: gateway.identity_key.clone(),
                exit_name: gateway.name.clone(),
                exit_location,
                success,
                reason,
            });

            wait_disconnected(rpc_client.clone()).await;
        }

        print_summary(&outcomes);
        Ok(())
    }
}

async fn attempt_connection(mut rpc_client: RpcClient) -> Result<()> {
    // Subscribe before connecting so we don't miss the state transition.
    let mut stream = rpc_client.clone().listen_to_events().await?;
    rpc_client.connect_tunnel().await?;

    while let Some(event) = stream.next().await {
        let TunnelEvent::NewState(state) = event? else {
            continue;
        };
        match state {
            TunnelState::Connected { .. } => return Ok(()),
            TunnelState::Error(reason) => {
                return Err(anyhow!("error state: {:?}", reason));
            }
            TunnelState::Offline { .. } => {
                return Err(anyhow!("device offline"));
            }
            _ => {}
        }
    }
    Err(anyhow!("event stream ended unexpectedly"))
}

async fn wait_disconnected(mut rpc_client: RpcClient) {
    if rpc_client.disconnect_tunnel().await.is_err() {
        return;
    }

    let Ok(mut stream) = rpc_client.listen_to_events().await else {
        return;
    };

    let _ = timeout(Duration::from_secs(DISCONNECT_TIMEOUT_SECS), async {
        while let Some(event) = stream.next().await {
            let Ok(TunnelEvent::NewState(state)) = event else {
                continue;
            };
            if matches!(
                state,
                TunnelState::Disconnected | TunnelState::Offline { .. }
            ) {
                return;
            }
        }
    })
    .await;
}

fn print_summary(outcomes: &[Outcome]) {
    let successes = outcomes.iter().filter(|o| o.success).count();
    println!();
    println!("=== Network Check Summary ===");
    println!("{}/{} exit gateways reachable", successes, outcomes.len());
    println!();

    for o in outcomes {
        let status = if o.success { "OK  " } else { "FAIL" };
        let suffix = if o.reason.is_empty() {
            String::new()
        } else {
            format!(" ({})", o.reason)
        };
        println!(
            "  {} {} ({}) {}{}",
            status, o.exit_id, o.exit_name, o.exit_location, suffix
        );
    }
}
