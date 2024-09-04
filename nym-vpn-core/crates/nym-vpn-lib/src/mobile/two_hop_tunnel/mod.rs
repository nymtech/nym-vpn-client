#[cfg(target_os = "android")]
#[path = "android.rs"]
mod imp;

#[cfg(target_os = "ios")]
#[path = "ios.rs"]
mod imp;

use std::sync::Arc;

use tokio_util::sync::CancellationToken;

#[cfg(target_os = "ios")]
use super::ios::tun_provider::OSTunProvider;
use super::{wg_config::WgNodeConfig, Result};
#[cfg(target_os = "android")]
use crate::platform::android::AndroidTunProvider;

/// Start two-hop WireGuard tunnel.
///
/// ## Abstract
///
/// In principle the two-hop WireGuard is implemented in the following way:
///
/// * The tunnel to the entry node is established using wg/netstack.
/// * The UDP connection to the exit node is established over the entry tunnel.
/// * The exit traffic is captured on tun interface and directed towards local UDP forwarding proxy.
/// * The local UDP forwarding proxy injects all received UDP datagrams into the UDP connection to the exit node.
pub async fn start(
    entry_node_config: WgNodeConfig,
    exit_node_config: WgNodeConfig,
    #[cfg(target_os = "android")] tun_provider: Arc<dyn AndroidTunProvider>,
    #[cfg(target_os = "ios")] tun_provider: Arc<dyn OSTunProvider>,
    shutdown_token: CancellationToken,
) -> Result<()> {
    imp::TwoHopTunnelImp::start(
        entry_node_config,
        exit_node_config,
        tun_provider,
        shutdown_token,
    )
    .await
}
