use std::{
    io,
    net::{IpAddr, Ipv6Addr},
};

use nym_connection_monitor::ConnectionMonitorTask;
use nym_gateway_directory::GatewayClient;
use nym_ip_packet_client::IprClientConnect;
use nym_task::TaskManager;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tun2::{AbstractDevice, AsyncDevice};

use super::{gateway_selector::SelectedGateways, Error, Result};
use crate::{mixnet::SharedMixnetClient, GenericNymVpnConfig, MixnetError};

const DEFAULT_TUN_MTU: u16 = 1500;

pub struct MixnetTunnel {
    task_manager: TaskManager,
    mixnet_client: SharedMixnetClient,
    shutdown_token: CancellationToken,
    processor_handle: JoinHandle<Result<AsyncDevice, MixnetError>>,
}

impl MixnetTunnel {
    pub async fn run(
        nym_config: GenericNymVpnConfig,
        task_manager: TaskManager,
        mixnet_client: SharedMixnetClient,
        gateway_directory_client: GatewayClient,
        selected_gateways: SelectedGateways,
        shutdown_token: CancellationToken,
    ) -> Result<()> {
        // Setup mixnet routing
        let mixnet_client_address = mixnet_client.nym_address().await;
        let gateway_used = mixnet_client_address.gateway().to_base58_string();
        let entry_mixnet_gateway_ip: IpAddr = gateway_directory_client
            .lookup_gateway_ip(&gateway_used)
            .await
            .map_err(|source| Error::LookupGatewayIp {
                gateway_id: gateway_used,
                source,
            })?;

        let exit_mix_addresses = selected_gateways.exit.ipr_address.unwrap();
        let mut ipr_client = IprClientConnect::new_from_inner(mixnet_client.inner()).await;

        // Create tun device
        let tun_addresses = ipr_client
            .connect(exit_mix_addresses.0, nym_config.nym_ips)
            .await
            .map_err(Error::ConnectToIpPacketRouter)?;
        let mut tun_config = tun2::Configuration::default();
        tun_config
            .mtu(nym_config.nym_mtu.unwrap_or(DEFAULT_TUN_MTU))
            .address(tun_addresses.ipv4)
            .up();
        let tun_device = tun2::create_as_async(&tun_config).map_err(Error::CreateTunDevice)?;
        let device_name = tun_device.tun_name().map_err(Error::ObtainTunName)?;
        set_tun_ipv6_addr(&device_name, tun_addresses.ipv6).map_err(Error::SetTunIpv6Addr)?;

        // Create connection monitor
        let connection_monitor = ConnectionMonitorTask::setup();

        // Create packet processor
        let processor_config = crate::mixnet::Config::new(exit_mix_addresses.0);
        let processor_handle = crate::mixnet::start_processor(
            processor_config,
            tun_device,
            mixnet_client.clone(),
            &task_manager,
            tun_addresses,
            &connection_monitor,
        )
        .await;

        let mixnet_client_sender = mixnet_client.split_sender().await;
        connection_monitor.start(
            mixnet_client_sender,
            mixnet_client_address,
            tun_addresses,
            exit_mix_addresses.0,
            &task_manager,
        );

        let tunnel = Self {
            task_manager,
            mixnet_client,
            shutdown_token,
            processor_handle,
        };

        tunnel.wait().await;

        Ok(())
    }

    async fn wait(mut self) {
        let mut shutdown_task_client = self.task_manager.subscribe();

        tokio::select! {
            _ = shutdown_task_client.recv() => {
                tracing::debug!("Task manager received shutdown.");
            }
            _ = self.shutdown_token.cancelled() => {
                tracing::debug!("Received cancellation. Shutting down task manager.");
                _ = self.task_manager.signal_shutdown();
            }
        }

        self.task_manager.wait_for_shutdown().await;
        self.mixnet_client.disconnect().await;
        let _ = self.processor_handle.await;
    }
}

fn set_tun_ipv6_addr(_device_name: &str, _ipv6_addr: Ipv6Addr) -> io::Result<()> {
    #[cfg(target_os = "linux")]
    std::process::Command::new("ip")
        .args([
            "-6",
            "addr",
            "add",
            &_ipv6_addr.to_string(),
            "dev",
            _device_name,
        ])
        .output()?;

    #[cfg(target_os = "macos")]
    std::process::Command::new("ifconfig")
        .args([_device_name, "inet6", "add", &_ipv6_addr.to_string()])
        .output()?;

    Ok(())
}
