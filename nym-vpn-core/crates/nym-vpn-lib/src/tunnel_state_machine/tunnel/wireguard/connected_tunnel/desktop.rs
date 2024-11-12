// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{error::Error as StdError, net::IpAddr};

use tokio::task::JoinHandle;
use tun::AsyncDevice;

use nym_task::TaskManager;
use nym_wg_gateway_client::WgGatewayClient;
use nym_wg_go::{netstack, wireguard_go};

use crate::{
    tunnel_state_machine::tunnel::{
        wireguard::{connector::ConnectionData, fd::DupFd, two_hop_config::TwoHopConfig},
        Error, Result,
    },
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
        // 1500 - 80 (ipv6+wg header)
        1420
    }

    pub fn exit_mtu(&self) -> u16 {
        // 1420 - 80 (ipv6+wg header)
        1340
    }

    pub fn run(self, options: TunnelOptions) -> Result<TunnelHandle> {
        match options {
            TunnelOptions::TunTun(tuntun_options) => self.run_using_tun_tun(tuntun_options),
            TunnelOptions::Netstack(netstack_options) => self.run_using_netstack(netstack_options),
        }
    }

    fn run_using_tun_tun(self, options: TunTunTunnelOptions) -> Result<TunnelHandle> {
        let wg_entry_config = WgNodeConfig::with_gateway_data(
            self.connection_data.entry.clone(),
            self.entry_gateway_client.keypair().private_key(),
            options.dns.clone(),
            self.entry_mtu(),
        );

        let wg_exit_config = WgNodeConfig::with_gateway_data(
            self.connection_data.exit.clone(),
            self.exit_gateway_client.keypair().private_key(),
            options.dns,
            self.exit_mtu(),
        );

        let entry_tunnel = wireguard_go::Tunnel::start(
            wg_entry_config.into_wireguard_config(),
            #[cfg(unix)]
            options.entry_tun.get_ref().dup_fd().map_err(Error::DupFd)?,
            #[cfg(windows)]
            &options.entry_tun_name,
        )
        .map_err(Error::Wireguard)?;

        let exit_tunnel = wireguard_go::Tunnel::start(
            wg_exit_config.into_wireguard_config(),
            #[cfg(unix)]
            options.exit_tun.get_ref().dup_fd().map_err(Error::DupFd)?,
            #[cfg(windows)]
            &options.exit_tun_name,
        )
        .map_err(Error::Wireguard)?;

        Ok(TunnelHandle {
            task_manager: self.task_manager,
            internal_handle: InternalTunnelHandle::TunTun {
                #[cfg(unix)]
                entry_tun: options.entry_tun,
                #[cfg(unix)]
                exit_tun: options.exit_tun,
                entry_wg_tunnel: Some(entry_tunnel),
                exit_wg_tunnel: Some(exit_tunnel),
            },
            bandwidth_controller_handle: self.bandwidth_controller_handle,
        })
    }

    fn run_using_netstack(self, options: NetstackTunnelOptions) -> Result<TunnelHandle> {
        let wg_entry_config = WgNodeConfig::with_gateway_data(
            self.connection_data.entry.clone(),
            self.entry_gateway_client.keypair().private_key(),
            options.dns.clone(),
            self.entry_mtu(),
        );

        let wg_exit_config = WgNodeConfig::with_gateway_data(
            self.connection_data.exit.clone(),
            self.exit_gateway_client.keypair().private_key(),
            options.dns,
            self.exit_mtu(),
        );

        let two_hop_config = TwoHopConfig::new(wg_entry_config, wg_exit_config);

        let mut entry_tunnel =
            netstack::Tunnel::start(two_hop_config.entry.into_netstack_config())?;

        // Open connection to the exit node via entry node.
        let exit_connection = entry_tunnel.open_connection(
            two_hop_config.forwarder.listen_endpoint.port(),
            two_hop_config.forwarder.client_port,
            two_hop_config.forwarder.exit_endpoint,
        )?;

        #[allow(unused_mut)]
        let mut exit_tunnel = wireguard_go::Tunnel::start(
            two_hop_config.exit.into_wireguard_config(),
            #[cfg(unix)]
            options.exit_tun.get_ref().dup_fd().map_err(Error::DupFd)?,
            #[cfg(windows)]
            &options.exit_tun_name,
        )?;

        Ok(TunnelHandle {
            task_manager: self.task_manager,
            internal_handle: InternalTunnelHandle::Netstack {
                #[cfg(unix)]
                exit_tun: options.exit_tun,
                entry_wg_tunnel: Some(entry_tunnel),
                exit_wg_tunnel: Some(exit_tunnel),
                exit_connection: Some(exit_connection),
            },
            bandwidth_controller_handle: self.bandwidth_controller_handle,
        })
    }
}

pub enum TunnelOptions {
    /// Multihop configured using two tun adapters.
    TunTun(TunTunTunnelOptions),

    /// Multihop using single tun adapter and netstack with local UDP forwarder to wrap tunnels.
    Netstack(NetstackTunnelOptions),
}

/// Multihop configuration using two tun adapters.
pub struct TunTunTunnelOptions {
    /// Entry tunnel device.
    #[cfg(unix)]
    pub entry_tun: AsyncDevice,

    /// Exit tunnel device.
    #[cfg(unix)]
    pub exit_tun: AsyncDevice,

    /// Entry tunnel device name.
    #[cfg(windows)]
    pub entry_tun_name: String,

    /// Exit tunnel device name.
    #[cfg(windows)]
    pub exit_tun_name: String,

    /// In-tunnel DNS addresses
    pub dns: Vec<IpAddr>,
}

/// Multihop configuration based on WireGuard/netstack.
pub struct NetstackTunnelOptions {
    /// Entry tunnel device.
    #[cfg(unix)]
    pub exit_tun: AsyncDevice,

    /// Exit tunnel device name.
    #[cfg(windows)]
    pub exit_tun_name: String,

    /// In-tunnel DNS addresses
    pub dns: Vec<IpAddr>,
}

enum InternalTunnelHandle {
    TunTun {
        #[cfg(unix)]
        entry_tun: AsyncDevice,
        #[cfg(unix)]
        exit_tun: AsyncDevice,
        entry_wg_tunnel: Option<wireguard_go::Tunnel>,
        exit_wg_tunnel: Option<wireguard_go::Tunnel>,
    },
    Netstack {
        #[cfg(unix)]
        exit_tun: AsyncDevice,
        entry_wg_tunnel: Option<netstack::Tunnel>,
        exit_wg_tunnel: Option<wireguard_go::Tunnel>,
        exit_connection: Option<netstack::TunnelConnection>,
    },
}

pub struct TunnelHandle {
    task_manager: TaskManager,
    internal_handle: InternalTunnelHandle,
    bandwidth_controller_handle: JoinHandle<()>,
}

impl TunnelHandle {
    /// Close entry and exit WireGuard tunnels and signal mixnet facilities shutdown.
    pub fn cancel(&mut self) {
        match self.internal_handle {
            InternalTunnelHandle::Netstack {
                ref mut entry_wg_tunnel,
                ref mut exit_wg_tunnel,
                ref mut exit_connection,
                ..
            } => {
                if let Some(exit_wg_tunnel) = exit_wg_tunnel.take() {
                    exit_wg_tunnel.stop();
                }
                if let Some(exit_connection) = exit_connection.take() {
                    exit_connection.close();
                }
                if let Some(entry_wg_tunnel) = entry_wg_tunnel.take() {
                    entry_wg_tunnel.stop();
                }
            }
            InternalTunnelHandle::TunTun {
                ref mut entry_wg_tunnel,
                ref mut exit_wg_tunnel,
                ..
            } => {
                if let Some(entry_wg_tunnel) = entry_wg_tunnel.take() {
                    entry_wg_tunnel.stop();
                }

                if let Some(exit_wg_tunnel) = exit_wg_tunnel.take() {
                    exit_wg_tunnel.stop();
                }
            }
        }

        if let Err(e) = self.task_manager.signal_shutdown() {
            tracing::error!("Failed to signal task manager shutdown: {}", e);
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
    pub async fn wait(self) -> Vec<AsyncDevice> {
        if let Err(e) = self.bandwidth_controller_handle.await {
            tracing::error!("Failed to join on bandwidth controller: {}", e);
        }

        #[cfg(unix)]
        {
            match self.internal_handle {
                InternalTunnelHandle::Netstack { exit_tun, .. } => {
                    vec![exit_tun]
                }
                InternalTunnelHandle::TunTun {
                    entry_tun,
                    exit_tun,
                    ..
                } => {
                    vec![entry_tun, exit_tun]
                }
            }
        }

        #[cfg(windows)]
        vec![]
    }
}
