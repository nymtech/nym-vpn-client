// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{error::Error as StdError, os::fd::AsRawFd};

use tokio::task::JoinHandle;
use tun::AsyncDevice;

use nym_task::TaskManager;
use nym_wg_gateway_client::WgGatewayClient;
use nym_wg_go::{netstack, wireguard_go};

use super::super::connector::ConnectionData;
use crate::{
    tunnel_state_machine::tunnel::{wireguard::two_hop_config::TwoHopConfig, Error, Result},
    wg_config::{WgNodeConfig, WgPeer},
};

#[cfg(target_os = "ios")]
use crate::tunnel_state_machine::tunnel::wireguard::ios::dns64::Dns64Resolution;

pub struct ConnectedTunnel {
    task_manager: TaskManager,
    entry_gateway_client: WgGatewayClient,
    exit_gateway_client: WgGatewayClient,
    connection_data: ConnectionData,
    bandwidth_controller_handle: JoinHandle<()>,
}

impl ConnectedTunnel {
    pub fn new(
        task_manager: TaskManager,
        entry_gateway_client: WgGatewayClient,
        exit_gateway_client: WgGatewayClient,
        connection_data: ConnectionData,
        bandwidth_controller_handle: JoinHandle<()>,
    ) -> Self {
        Self {
            task_manager,
            entry_gateway_client,
            exit_gateway_client,
            connection_data,
            bandwidth_controller_handle,
        }
    }

    pub fn connection_data(&self) -> &ConnectionData {
        &self.connection_data
    }

    pub fn entry_mtu(&self) -> u16 {
        1360
    }

    pub fn exit_mtu(&self) -> u16 {
        1280
    }

    pub fn run(self, tun_device: AsyncDevice) -> Result<TunnelHandle> {
        let mut wg_entry_config = WgNodeConfig::with_gateway_data(
            self.connection_data.entry.clone(),
            self.entry_gateway_client.keypair().private_key(),
            self.entry_mtu(),
        );

        let wg_exit_config = WgNodeConfig::with_gateway_data(
            self.connection_data.exit.clone(),
            self.exit_gateway_client.keypair().private_key(),
            self.exit_mtu(),
        );

        // Save entry peer so that we can re-resolve it and update wg config on network changes.
        let orig_entry_peer = wg_entry_config.peer.clone();

        let mut two_hop_config = TwoHopConfig::new(wg_entry_config, wg_exit_config);

        // iOS does not perform dns64 resolution by default. Do that manually.
        two_hop_config
            .entry
            .peer
            .resolve_in_place()
            .expect("map to error");

        let entry_tunnel = netstack::Tunnel::start(two_hop_config.entry.into_netstack_config())
            .map_err(Error::StartWireguard)?;

        let exit_tunnel = wireguard_go::Tunnel::start(
            two_hop_config.exit.into_wireguard_config(),
            tun_device.get_ref().as_raw_fd(),
        )
        .map_err(Error::StartWireguard)?;

        Ok(TunnelHandle {
            task_manager: self.task_manager,
            tun_device,
            entry_wg_tunnel: Some(entry_tunnel),
            exit_wg_tunnel: Some(exit_tunnel),
            orig_entry_peer,
            bandwidth_controller_handle: self.bandwidth_controller_handle,
        })
    }
}

pub struct TunnelHandle {
    task_manager: TaskManager,
    tun_device: AsyncDevice,
    entry_wg_tunnel: Option<netstack::Tunnel>,
    exit_wg_tunnel: Option<wireguard_go::Tunnel>,
    orig_entry_peer: WgPeer,
    bandwidth_controller_handle: JoinHandle<()>,
}

impl TunnelHandle {
    /// Close entry and exit WireGuard tunnels and signal mixnet facilities shutdown.
    pub fn cancel(&mut self) {
        if let Some(entry_wg_tunnel) = self.entry_wg_tunnel.take() {
            entry_wg_tunnel.stop();
        }

        if let Some(exit_wg_tunnel) = self.exit_wg_tunnel.take() {
            exit_wg_tunnel.stop();
        }

        if let Err(e) = self.task_manager.signal_shutdown() {
            tracing::error!("Failed to signal shutdown: {}", e);
        }
    }

    /// Wait for the next mixnet error.
    ///
    /// This method is cancel safe.
    /// Returns `None` if the underlying channel has been closed.
    pub async fn recv_error(&mut self) -> Option<Box<dyn StdError + 'static + Send + Sync>> {
        self.task_manager.wait_for_error().await
    }

    /// Wait until the tunnel finished execution.
    ///
    /// Returns a pair of tun devices no longer in use.
    pub async fn wait(self) -> AsyncDevice {
        if let Err(e) = self.bandwidth_controller_handle.await {
            tracing::error!("Failed to join on bandwidth controller: {}", e);
        }

        self.tun_device
    }
}
