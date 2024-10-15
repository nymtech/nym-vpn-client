// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{error::Error as StdError, os::fd::AsRawFd};

use tokio::task::JoinHandle;
use tun::AsyncDevice;

use nym_task::TaskManager;
use nym_wg_gateway_client::WgGatewayClient;
use nym_wg_go::wireguard_go;

use super::connector::ConnectionData;
use crate::{
    tunnel_state_machine::tunnel::{Error, Result},
    wg_config::WgNodeConfig,
};

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
        // 1500 - 60 (ipv4+wg header)
        1440
    }

    pub fn exit_mtu(&self) -> u16 {
        // 1440 - 80 (ipv6+wg header)
        1360
    }

    pub fn run(self, entry_tun: AsyncDevice, exit_tun: AsyncDevice) -> Result<TunnelHandle> {
        let mut wg_entry_config = WgNodeConfig::with_gateway_data(
            self.connection_data.entry.clone(),
            self.entry_gateway_client.keypair().private_key(),
        );
        wg_entry_config.interface.mtu = self.entry_mtu();

        let mut wg_exit_config = WgNodeConfig::with_gateway_data(
            self.connection_data.exit.clone(),
            self.exit_gateway_client.keypair().private_key(),
        );
        wg_exit_config.interface.mtu = self.exit_mtu();

        let entry_tunnel = wireguard_go::Tunnel::start(
            wg_entry_config.into_wireguard_config(),
            entry_tun.get_ref().as_raw_fd(),
        )
        .map_err(Error::StartWireguard)?;

        let exit_tunnel = wireguard_go::Tunnel::start(
            wg_exit_config.into_wireguard_config(),
            exit_tun.get_ref().as_raw_fd(),
        )
        .map_err(Error::StartWireguard)?;

        Ok(TunnelHandle {
            task_manager: self.task_manager,
            entry_tun,
            exit_tun,
            entry_wg_tunnel: Some(entry_tunnel),
            exit_wg_tunnel: Some(exit_tunnel),
            bandwidth_controller_handle: self.bandwidth_controller_handle,
        })
    }
}

pub struct TunnelHandle {
    task_manager: TaskManager,
    entry_tun: AsyncDevice,
    exit_tun: AsyncDevice,
    entry_wg_tunnel: Option<wireguard_go::Tunnel>,
    exit_wg_tunnel: Option<wireguard_go::Tunnel>,
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
    pub async fn wait(self) -> WaitResult {
        if let Err(e) = self.bandwidth_controller_handle.await {
            tracing::error!("Failed to join on bandwidth controller: {}", e);
        }

        WaitResult {
            entry_tun: self.entry_tun,
            exit_tun: self.exit_tun,
        }
    }
}

pub struct WaitResult {
    pub entry_tun: AsyncDevice,
    pub exit_tun: AsyncDevice,
}
