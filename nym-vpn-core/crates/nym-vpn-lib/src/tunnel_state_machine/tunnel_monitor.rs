#[cfg(any(target_os = "linux", target_os = "macos"))]
use std::net::Ipv4Addr;
#[cfg(any(target_os = "android", target_os = "ios"))]
use std::os::fd::{AsRawFd, IntoRawFd};
#[cfg(target_os = "android")]
use std::os::fd::{FromRawFd, OwnedFd};
use std::{cmp, time::Duration};
#[cfg(unix)]
use std::{os::fd::RawFd, sync::Arc};

#[cfg(windows)]
use super::wintun::{self, WintunAdapterConfig};
#[cfg(any(target_os = "ios", target_os = "android"))]
use ipnetwork::{IpNetwork, Ipv4Network, Ipv6Network};
use nym_gateway_directory::GatewayMinPerformance;
use nym_vpn_account_controller::AccountControllerCommander;
use time::OffsetDateTime;
use tokio::{sync::mpsc, task::JoinHandle};
use tokio_util::sync::CancellationToken;
use tun::AsyncDevice;
#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
use tun::Device;

#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
use nym_ip_packet_requests::IpPair;

#[cfg(target_os = "linux")]
use super::default_interface::DefaultInterface;
#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
use super::{dns_handler::DnsHandlerHandle, route_handler::RouteHandler};
#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
use super::{route_handler::RoutingConfig, tun_ipv6};
use super::{
    tunnel::{
        self, AnyTunnelHandle, ConnectedMixnet, MixnetConnectOptions, SelectedGateways, Tombstone,
    },
    Error, NymConfig, Result, TunnelSettings,
};
use nym_vpn_lib_types::{
    ConnectionData, ErrorStateReason, Gateway, MixnetConnectionData, MixnetEvent, NymAddress,
    TunnelConnectionData, TunnelType, WireguardConnectionData, WireguardNode,
};

#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
use super::tunnel::wireguard::connected_tunnel::{
    NetstackTunnelOptions, TunTunTunnelOptions, TunnelOptions,
};
#[cfg(any(target_os = "ios", target_os = "android"))]
use crate::tunnel_provider;
#[cfg(target_os = "android")]
use crate::tunnel_provider::android::AndroidTunProvider;
#[cfg(target_os = "ios")]
use crate::tunnel_provider::ios::OSTunProvider;
use crate::tunnel_state_machine::{account, WireguardMultihopMode};

/// Default MTU for mixnet tun device.
const DEFAULT_TUN_MTU: u16 = if cfg!(any(target_os = "ios", target_os = "android")) {
    1280
} else {
    1500
};

/// User-facing tunnel type identifier.
#[cfg(windows)]
const WINTUN_TUNNEL_TYPE: &str = "Nym";

/// The user-facing name of wintun adapter.
///
/// Note that it refers to tunnel type because rust-tun uses the same name for adapter and
/// tunnel type and there is no way to change that.
#[cfg(windows)]
const MIXNET_WINTUN_NAME: &str = WINTUN_TUNNEL_TYPE;

/// The user-facing name of wintun adapter used as entry tunnel.
#[cfg(windows)]
const WG_ENTRY_WINTUN_NAME: &str = "WireGuard (entry)";

/// The user-facing name of wintun adapter used as exit tunnel.
#[cfg(windows)]
const WG_EXIT_WINTUN_NAME: &str = "WireGuard (exit)";

/// WireGuard entry adapter GUID.
#[cfg(windows)]
const WG_ENTRY_WINTUN_GUID: &str = "{AFE43773-E1F8-4EBB-8536-176AB86AFE9B}";

/// WireGuard exit adapter GUID.
#[cfg(windows)]
const WG_EXIT_WINTUN_GUID: &str = "{AFE43773-E1F8-4EBB-8536-176AB86AFE9C}";

pub type TunnelMonitorEventReceiver = mpsc::UnboundedReceiver<TunnelMonitorEvent>;

/// Initial delay between retry attempts.
const INITIAL_WAIT_DELAY: Duration = Duration::from_secs(2);

/// Wait delay multiplier used for each subsequent retry attempt.
const DELAY_MULTIPLIER: u32 = 2;

/// Max wait delay between retry attempts.
const MAX_WAIT_DELAY: Duration = Duration::from_secs(15);

#[derive(Debug, Clone)]
pub enum TunnelMonitorEvent {
    /// Initializing mixnet client
    InitializingClient,

    /// Syncronizing account with vpn-api
    SyncingAccount,

    /// Registering device with vpn-api
    RegisteringDevice,

    /// Requesting and downloading zknym credentials from vpn-api
    RequestingZkNyms,

    /// Selecting gateways
    SelectingGateways,

    /// Selected gateways
    SelectedGateways(Box<SelectedGateways>),

    /// Establishing tunnel connection
    EstablishingTunnel(Box<ConnectionData>),

    /// Tunnel is up
    Up(Box<ConnectionData>),

    /// Tunnel went down
    Down(Option<ErrorStateReason>),
}

pub struct TunnelMonitorHandle {
    cancel_token: CancellationToken,
    join_handle: JoinHandle<Tombstone>,
}

impl TunnelMonitorHandle {
    pub fn cancel(&self) {
        self.cancel_token.cancel();
    }

    pub async fn wait(self) -> Tombstone {
        self.join_handle
            .await
            .inspect_err(|e| {
                tracing::error!("Failed to join on tunnel monitor handle: {}", e);
            })
            .unwrap_or_default()
    }
}

pub struct TunnelMonitor {
    monitor_event_sender: mpsc::UnboundedSender<TunnelMonitorEvent>,
    mixnet_event_sender: mpsc::UnboundedSender<MixnetEvent>,
    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    route_handler: RouteHandler,
    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    dns_handler: DnsHandlerHandle,
    #[cfg(target_os = "ios")]
    tun_provider: Arc<dyn OSTunProvider>,
    #[cfg(target_os = "android")]
    tun_provider: Arc<dyn AndroidTunProvider>,
    nym_config: NymConfig,
    tunnel_settings: TunnelSettings,
    account_controller_tx: AccountControllerCommander,
    cancel_token: CancellationToken,
}

impl TunnelMonitor {
    // todo: fix too many arguments
    #[allow(clippy::too_many_arguments)]
    pub fn start(
        retry_attempt: u32,
        selected_gateways: Option<SelectedGateways>,
        monitor_event_sender: mpsc::UnboundedSender<TunnelMonitorEvent>,
        mixnet_event_sender: mpsc::UnboundedSender<MixnetEvent>,
        #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
        route_handler: RouteHandler,
        #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
        dns_handler: DnsHandlerHandle,
        #[cfg(target_os = "ios")] tun_provider: Arc<dyn OSTunProvider>,
        #[cfg(target_os = "android")] tun_provider: Arc<dyn AndroidTunProvider>,
        nym_config: NymConfig,
        tunnel_settings: TunnelSettings,
        account_controller_tx: AccountControllerCommander,
    ) -> TunnelMonitorHandle {
        let cancel_token = CancellationToken::new();
        let tunnel_monitor = Self {
            monitor_event_sender,
            mixnet_event_sender,
            #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
            route_handler,
            #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
            dns_handler,
            #[cfg(any(target_os = "ios", target_os = "android"))]
            tun_provider,
            nym_config,
            tunnel_settings,
            account_controller_tx,
            cancel_token: cancel_token.clone(),
        };
        let join_handle = tokio::spawn(tunnel_monitor.run(retry_attempt, selected_gateways));

        TunnelMonitorHandle {
            cancel_token,
            join_handle,
        }
    }

    async fn run(
        mut self,
        retry_attempt: u32,
        selected_gateways: Option<SelectedGateways>,
    ) -> Tombstone {
        let (tombstone, reason) = match self.run_inner(retry_attempt, selected_gateways).await {
            Ok(tombstone) => (tombstone, None),
            Err(e) => {
                tracing::error!("Tunnel monitor exited with error: {}", e);
                (Tombstone::default(), e.error_state_reason())
            }
        };

        self.send_event(TunnelMonitorEvent::Down(reason));

        tombstone
    }

    async fn run_inner(
        &mut self,
        retry_attempt: u32,
        selected_gateways: Option<SelectedGateways>,
    ) -> Result<Tombstone> {
        if retry_attempt > 0 {
            let delay = wait_delay(retry_attempt);
            tracing::debug!("Waiting for {}s before connecting.", delay.as_secs());

            self.cancel_token
                .run_until_cancelled(tokio::time::sleep(delay))
                .await
                .ok_or(Error::Tunnel(tunnel::Error::Cancelled))?;
        }

        self.send_event(TunnelMonitorEvent::InitializingClient);

        self.setup_account().await?;

        self.send_event(TunnelMonitorEvent::SelectingGateways);

        let gateway_performance_options = self.tunnel_settings.gateway_performance_options;
        let gateway_min_performance = GatewayMinPerformance::from_percentage_values(
            gateway_performance_options
                .mixnet_min_performance
                .map(u64::from),
            gateway_performance_options
                .vpn_min_performance
                .map(u64::from),
        );

        let mut gateway_config = self.nym_config.gateway_config.clone();
        match gateway_min_performance {
            Ok(gateway_min_performance) => {
                gateway_config =
                    gateway_config.with_min_gateway_performance(gateway_min_performance);
            }
            Err(e) => {
                tracing::error!(
                    "Invalid gateway performance values. Will carry on with initial values. Error: {}"
                , e);
            }
        }

        let selected_gateways = if let Some(selected_gateways) = selected_gateways {
            selected_gateways
        } else {
            let new_gateways = tunnel::select_gateways(
                gateway_config.clone(),
                self.tunnel_settings.tunnel_type,
                self.tunnel_settings.entry_point.clone(),
                self.tunnel_settings.exit_point.clone(),
                self.tunnel_settings.user_agent.clone(),
                self.cancel_token.child_token(),
            )
            .await?;

            self.send_event(TunnelMonitorEvent::SelectedGateways(Box::new(
                new_gateways.clone(),
            )));

            new_gateways
        };

        let connect_options = MixnetConnectOptions {
            data_path: self.nym_config.data_path.clone(),
            gateway_config,
            mixnet_client_config: self.tunnel_settings.mixnet_client_config.clone(),
            tunnel_type: self.tunnel_settings.tunnel_type,
            enable_credentials_mode: self.tunnel_settings.enable_credentials_mode,
            stats_recipient_address: self
                .tunnel_settings
                .statistics_recipient
                .as_deref()
                .copied(),
            selected_gateways: selected_gateways.clone(),
            user_agent: None, // todo: provide user-agent
        };

        #[cfg(target_os = "android")]
        let tun_provider = self.tun_provider.clone();
        #[cfg(unix)]
        let connection_fd_callback = move |_fd: RawFd| {
            #[cfg(target_os = "android")]
            tun_provider.bypass(_fd);
        };
        let mut connected_mixnet = tunnel::connect_mixnet(
            connect_options,
            &self.nym_config.network_env,
            self.cancel_token.child_token(),
            #[cfg(unix)]
            Arc::new(connection_fd_callback),
        )
        .await?;

        // Route mixnet client outside the tunnel.
        #[cfg(target_os = "android")]
        match connected_mixnet.websocket_fd().await {
            Some(fd) => {
                self.tun_provider.bypass(fd);
            }
            None => {
                tracing::error!("Failed to obtain websocket for bypass");
            }
        }

        let status_listener_handle = connected_mixnet
            .start_event_listener(
                self.mixnet_event_sender.clone(),
                self.cancel_token.child_token(),
            )
            .await;

        let selected_gateways = connected_mixnet.selected_gateways().clone();
        let (tunnel_conn_data, mut tunnel_handle) = match self.tunnel_settings.tunnel_type {
            TunnelType::Mixnet => self.start_mixnet_tunnel(connected_mixnet).await?,
            TunnelType::Wireguard => {
                match self.tunnel_settings.wireguard_tunnel_options.multihop_mode {
                    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
                    WireguardMultihopMode::TunTun => {
                        self.start_wireguard_tunnel(connected_mixnet).await?
                    }
                    WireguardMultihopMode::Netstack => {
                        self.start_wireguard_netstack_tunnel(connected_mixnet)
                            .await?
                    }
                }
            }
        };

        let conn_data = ConnectionData {
            entry_gateway: Gateway::from(*selected_gateways.entry),
            exit_gateway: Gateway::from(*selected_gateways.exit),
            connected_at: None,
            tunnel: tunnel_conn_data,
        };
        self.send_event(TunnelMonitorEvent::EstablishingTunnel(Box::new(
            conn_data.clone(),
        )));

        // todo: do initial ping

        let conn_data = ConnectionData {
            connected_at: Some(OffsetDateTime::now_utc()),
            ..conn_data
        };
        self.send_event(TunnelMonitorEvent::Up(Box::new(conn_data)));

        let task_error = self
            .cancel_token
            .run_until_cancelled(tunnel_handle.recv_error())
            .await;

        if let Some(Some(task_error)) = task_error {
            tracing::error!("Task manager quit with error: {}", task_error);
        }

        tracing::debug!("Wait for tunnel to exit");
        tunnel_handle.cancel();

        let tun_devices = tunnel_handle
            .wait()
            .await
            .inspect_err(|e| {
                tracing::error!("Failed to gracefully shutdown the tunnel: {}", e);
            })
            .unwrap_or_default();

        tracing::debug!("Wait for status listener to exit");
        if let Err(e) = status_listener_handle.await {
            tracing::error!("Failed to join on status listener: {}", e);
        }

        Ok(tun_devices)
    }

    fn send_event(&mut self, event: TunnelMonitorEvent) {
        if let Err(e) = self.monitor_event_sender.send(event) {
            tracing::error!("Failed to send event: {}", e);
        }
    }

    async fn setup_account(&mut self) -> Result<()> {
        self.send_event(TunnelMonitorEvent::SyncingAccount);
        account::wait_for_account_sync(
            self.account_controller_tx.clone(),
            self.cancel_token.clone(),
        )
        .await?;

        account::wait_for_device_sync(
            self.account_controller_tx.clone(),
            self.cancel_token.clone(),
        )
        .await?;

        self.send_event(TunnelMonitorEvent::RegisteringDevice);
        account::wait_for_device_register(
            self.account_controller_tx.clone(),
            self.cancel_token.clone(),
        )
        .await?;

        if self.tunnel_settings.enable_credentials_mode {
            self.send_event(TunnelMonitorEvent::RequestingZkNyms);
            account::wait_for_credentials_ready(
                self.account_controller_tx.clone(),
                self.cancel_token.clone(),
            )
            .await?;
        }

        Ok(())
    }

    async fn start_mixnet_tunnel(
        &mut self,
        connected_mixnet: ConnectedMixnet,
    ) -> Result<(TunnelConnectionData, AnyTunnelHandle)> {
        let interface_addrs = self.tunnel_settings.mixnet_tunnel_options.interface_addrs;

        let connected_tunnel = connected_mixnet
            .connect_mixnet_tunnel(interface_addrs, self.cancel_token.clone())
            .await?;
        let assigned_addresses = connected_tunnel.assigned_addresses();

        let mtu: u16 = self
            .tunnel_settings
            .mixnet_tunnel_options
            .mtu
            .unwrap_or(DEFAULT_TUN_MTU);

        #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
        let tun_device = Self::create_mixnet_device(assigned_addresses.interface_addresses, mtu)?;

        #[cfg(any(target_os = "ios", target_os = "android"))]
        let tun_device = {
            let packet_tunnel_settings = tunnel_provider::tunnel_settings::TunnelSettings {
                dns_servers: self.tunnel_settings.dns.ip_addresses().to_vec(),
                interface_addresses: vec![
                    IpNetwork::V4(Ipv4Network::from(
                        assigned_addresses.interface_addresses.ipv4,
                    )),
                    IpNetwork::V6(Ipv6Network::from(
                        assigned_addresses.interface_addresses.ipv6,
                    )),
                ],
                remote_addresses: vec![assigned_addresses.entry_mixnet_gateway_ip],
                mtu,
            };

            let tun_device = self.create_tun_device(packet_tunnel_settings).await?;
            tracing::debug!("Created tun device");
            tun_device
        };

        #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
        {
            let tun_name = tun_device
                .get_ref()
                .name()
                .map_err(Error::GetTunDeviceName)?;

            tracing::debug!("Created tun device: {}", tun_name);

            let routing_config = RoutingConfig::Mixnet {
                tun_name: tun_name.clone(),
                entry_gateway_address: assigned_addresses.entry_mixnet_gateway_ip,
                #[cfg(target_os = "linux")]
                physical_interface: DefaultInterface::current()?,
            };

            self.set_routes(routing_config).await?;
            self.set_dns(&tun_name).await?;
        }

        let tunnel_conn_data = TunnelConnectionData::Mixnet(MixnetConnectionData {
            nym_address: NymAddress::from(assigned_addresses.mixnet_client_address),
            exit_ipr: NymAddress::from(assigned_addresses.exit_mix_addresses.0),
            ipv4: assigned_addresses.interface_addresses.ipv4,
            ipv6: assigned_addresses.interface_addresses.ipv6,
        });

        let tunnel_handle = AnyTunnelHandle::from(connected_tunnel.run(tun_device).await);

        Ok((tunnel_conn_data, tunnel_handle))
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    async fn start_wireguard_netstack_tunnel(
        &mut self,
        connected_mixnet: ConnectedMixnet,
    ) -> Result<(TunnelConnectionData, AnyTunnelHandle)> {
        let connected_tunnel = connected_mixnet
            .connect_wireguard_tunnel(
                &self.nym_config.network_env,
                self.tunnel_settings.enable_credentials_mode,
                self.cancel_token.clone(),
            )
            .await?;
        let conn_data = connected_tunnel.connection_data();

        let exit_tun = Self::create_wireguard_device(
            IpPair {
                ipv4: conn_data.exit.private_ipv4,
                ipv6: conn_data.exit.private_ipv6,
            },
            Some(conn_data.entry.private_ipv4),
            connected_tunnel.exit_mtu(),
        )?;
        let exit_tun_name = exit_tun.get_ref().name().map_err(Error::GetTunDeviceName)?;
        tracing::info!("Created exit tun device: {}", exit_tun_name);

        let routing_config = RoutingConfig::WireguardNetstack {
            exit_tun_name: exit_tun_name.clone(),
            entry_gateway_address: conn_data.entry.endpoint.ip(),
            #[cfg(target_os = "linux")]
            physical_interface: DefaultInterface::current()?,
        };

        self.set_routes(routing_config).await?;
        self.set_dns(&exit_tun_name).await?;

        let tunnel_conn_data = TunnelConnectionData::Wireguard(WireguardConnectionData {
            entry: WireguardNode::from(conn_data.entry.clone()),
            exit: WireguardNode::from(conn_data.exit.clone()),
        });

        let tunnel_options = TunnelOptions::Netstack(NetstackTunnelOptions {
            exit_tun,
            dns: self.tunnel_settings.dns.ip_addresses().to_vec(),
        });

        let tunnel_handle = connected_tunnel.run(tunnel_options).await?;

        let any_tunnel_handle = AnyTunnelHandle::from(tunnel_handle);

        Ok((tunnel_conn_data, any_tunnel_handle))
    }

    #[cfg(windows)]
    async fn start_wireguard_netstack_tunnel(
        &mut self,
        connected_mixnet: ConnectedMixnet,
    ) -> Result<(TunnelConnectionData, AnyTunnelHandle)> {
        let connected_tunnel = connected_mixnet
            .connect_wireguard_tunnel(
                &self.nym_config.network_env,
                self.tunnel_settings.enable_credentials_mode,
                self.cancel_token.clone(),
            )
            .await?;
        let conn_data = connected_tunnel.connection_data();
        let entry_gateway_address = conn_data.entry.endpoint.ip();

        let exit_adapter_config = WintunAdapterConfig {
            interface_ipv4: conn_data.exit.private_ipv4,
            interface_ipv6: conn_data.exit.private_ipv6,
            gateway_ipv4: Some(conn_data.entry.private_ipv4),
            gateway_ipv6: Some(conn_data.entry.private_ipv6),
        };

        let tunnel_conn_data = TunnelConnectionData::Wireguard(WireguardConnectionData {
            entry: WireguardNode::from(conn_data.entry.clone()),
            exit: WireguardNode::from(conn_data.exit.clone()),
        });

        let tunnel_options = TunnelOptions::Netstack(NetstackTunnelOptions {
            exit_tun_name: WG_EXIT_WINTUN_NAME.to_owned(),
            exit_tun_guid: WG_EXIT_WINTUN_GUID.to_owned(),
            wintun_tunnel_type: WINTUN_TUNNEL_TYPE.to_owned(),
            dns: self.tunnel_settings.dns.ip_addresses().to_vec(),
        });

        let tunnel_handle = connected_tunnel
            .run(
                #[cfg(windows)]
                self.route_handler.clone(),
                tunnel_options,
            )
            .await?;

        let wintun_exit_interface = tunnel_handle
            .exit_wintun_interface()
            .expect("failed to obtain wintun exit interface");

        tracing::info!("Created wintun device: {}", wintun_exit_interface.name);

        wintun::setup_wintun_adapter(wintun_exit_interface.windows_luid(), exit_adapter_config)?;

        let routing_config = RoutingConfig::WireguardNetstack {
            exit_tun_name: wintun_exit_interface.name.clone(),
            entry_gateway_address,
            #[cfg(target_os = "linux")]
            physical_interface: DefaultInterface::current()?,
        };
        // todo: make sure to shutdown tunnel_handle on failure!
        self.set_routes(routing_config).await?;
        self.set_dns(&wintun_exit_interface.name).await?;

        let any_tunnel_handle = AnyTunnelHandle::from(tunnel_handle);

        Ok((tunnel_conn_data, any_tunnel_handle))
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    async fn start_wireguard_tunnel(
        &mut self,
        connected_mixnet: ConnectedMixnet,
    ) -> Result<(TunnelConnectionData, AnyTunnelHandle)> {
        let connected_tunnel = connected_mixnet
            .connect_wireguard_tunnel(
                &self.nym_config.network_env,
                self.tunnel_settings.enable_credentials_mode,
                self.cancel_token.clone(),
            )
            .await?;
        let conn_data = connected_tunnel.connection_data();

        let entry_tun = Self::create_wireguard_device(
            IpPair {
                ipv4: conn_data.entry.private_ipv4,
                ipv6: conn_data.entry.private_ipv6,
            },
            None,
            connected_tunnel.entry_mtu(),
        )?;
        let entry_tun_name = entry_tun
            .get_ref()
            .name()
            .map_err(Error::GetTunDeviceName)?;
        tracing::info!("Created entry tun device: {}", entry_tun_name);

        let exit_tun = Self::create_wireguard_device(
            IpPair {
                ipv4: conn_data.exit.private_ipv4,
                ipv6: conn_data.exit.private_ipv6,
            },
            Some(conn_data.entry.private_ipv4),
            connected_tunnel.exit_mtu(),
        )?;
        let exit_tun_name = exit_tun.get_ref().name().map_err(Error::GetTunDeviceName)?;
        tracing::info!("Created exit tun device: {}", exit_tun_name);

        let routing_config = RoutingConfig::Wireguard {
            entry_tun_name,
            exit_tun_name: exit_tun_name.clone(),
            entry_gateway_address: conn_data.entry.endpoint.ip(),
            exit_gateway_address: conn_data.exit.endpoint.ip(),
            #[cfg(target_os = "linux")]
            physical_interface: DefaultInterface::current()?,
        };
        self.set_routes(routing_config).await?;
        self.set_dns(&exit_tun_name).await?;

        let tunnel_conn_data = TunnelConnectionData::Wireguard(WireguardConnectionData {
            entry: WireguardNode::from(conn_data.entry.clone()),
            exit: WireguardNode::from(conn_data.exit.clone()),
        });

        let tunnel_options = TunnelOptions::TunTun(TunTunTunnelOptions {
            entry_tun,
            exit_tun,
            dns: self.tunnel_settings.dns.ip_addresses().to_vec(),
        });

        let tunnel_handle = connected_tunnel.run(tunnel_options).await?;
        let any_tunnel_handle = AnyTunnelHandle::from(tunnel_handle);

        Ok((tunnel_conn_data, any_tunnel_handle))
    }

    #[cfg(windows)]
    async fn start_wireguard_tunnel(
        &mut self,
        connected_mixnet: ConnectedMixnet,
    ) -> Result<(TunnelConnectionData, AnyTunnelHandle)> {
        let connected_tunnel = connected_mixnet
            .connect_wireguard_tunnel(
                &self.nym_config.network_env,
                self.tunnel_settings.enable_credentials_mode,
                self.cancel_token.clone(),
            )
            .await?;
        let conn_data = connected_tunnel.connection_data();

        let entry_gateway_address = conn_data.entry.endpoint.ip();
        let exit_gateway_address = conn_data.exit.endpoint.ip();

        let entry_adapter_config = WintunAdapterConfig {
            interface_ipv4: conn_data.entry.private_ipv4,
            interface_ipv6: conn_data.entry.private_ipv6,
            gateway_ipv4: None,
            gateway_ipv6: None,
        };
        let exit_adapter_config = WintunAdapterConfig {
            interface_ipv4: conn_data.exit.private_ipv4,
            interface_ipv6: conn_data.exit.private_ipv6,
            gateway_ipv4: Some(conn_data.entry.private_ipv4),
            gateway_ipv6: Some(conn_data.entry.private_ipv6),
        };

        let tunnel_conn_data = TunnelConnectionData::Wireguard(WireguardConnectionData {
            entry: WireguardNode::from(conn_data.entry.clone()),
            exit: WireguardNode::from(conn_data.exit.clone()),
        });

        let tunnel_options = TunnelOptions::TunTun(TunTunTunnelOptions {
            entry_tun_name: WG_ENTRY_WINTUN_NAME.to_owned(),
            entry_tun_guid: WG_ENTRY_WINTUN_GUID.to_owned(),
            exit_tun_name: WG_EXIT_WINTUN_NAME.to_owned(),
            exit_tun_guid: WG_EXIT_WINTUN_GUID.to_owned(),
            wintun_tunnel_type: WINTUN_TUNNEL_TYPE.to_owned(),
            dns: self.tunnel_settings.dns.ip_addresses().to_vec(),
        });

        let tunnel_handle = connected_tunnel
            .run(
                #[cfg(windows)]
                self.route_handler.clone(),
                tunnel_options,
            )
            .await?;

        let wintun_entry_interface = tunnel_handle
            .entry_wintun_interface()
            .expect("failed to obtain wintun entry interface");
        let wintun_exit_interface = tunnel_handle
            .exit_wintun_interface()
            .expect("failed to obtain wintun exit interface");

        tracing::info!(
            "Created entry wintun device: {}",
            wintun_entry_interface.name
        );
        tracing::info!("Created exit wintun device: {}", wintun_exit_interface.name);

        wintun::setup_wintun_adapter(wintun_entry_interface.windows_luid(), entry_adapter_config)?;
        wintun::setup_wintun_adapter(wintun_exit_interface.windows_luid(), exit_adapter_config)?;

        let routing_config = RoutingConfig::Wireguard {
            entry_tun_name: wintun_entry_interface.name.clone(),
            exit_tun_name: wintun_exit_interface.name.clone(),
            entry_gateway_address,
            exit_gateway_address,
        };
        // todo: make sure to shutdown tunnel_handle on failure!
        self.set_routes(routing_config).await?;
        self.set_dns(&wintun_exit_interface.name).await?;

        let any_tunnel_handle = AnyTunnelHandle::from(tunnel_handle);

        Ok((tunnel_conn_data, any_tunnel_handle))
    }

    #[cfg(any(target_os = "ios", target_os = "android"))]
    async fn start_wireguard_netstack_tunnel(
        &self,
        connected_mixnet: ConnectedMixnet,
    ) -> Result<(TunnelConnectionData, AnyTunnelHandle)> {
        let connected_tunnel = connected_mixnet
            .connect_wireguard_tunnel(
                &self.nym_config.network_env,
                self.tunnel_settings.enable_credentials_mode,
                self.cancel_token.clone(),
            )
            .await?;

        let conn_data = connected_tunnel.connection_data();

        let packet_tunnel_settings = tunnel_provider::tunnel_settings::TunnelSettings {
            dns_servers: self.tunnel_settings.dns.ip_addresses().to_vec(),
            interface_addresses: vec![
                IpNetwork::V4(Ipv4Network::from(conn_data.exit.private_ipv4)),
                IpNetwork::V6(Ipv6Network::from(conn_data.exit.private_ipv6)),
            ],
            remote_addresses: vec![conn_data.entry.endpoint.ip()],
            mtu: connected_tunnel.exit_mtu(),
        };

        let tun_device = self.create_tun_device(packet_tunnel_settings).await?;

        tracing::info!("Created tun device");

        let tunnel_conn_data = TunnelConnectionData::Wireguard(WireguardConnectionData {
            entry: WireguardNode::from(conn_data.entry.clone()),
            exit: WireguardNode::from(conn_data.exit.clone()),
        });

        let tunnel_handle = connected_tunnel
            .run(
                tun_device,
                self.tunnel_settings.dns.ip_addresses().to_vec(),
                #[cfg(target_os = "android")]
                self.tun_provider.clone(),
            )
            .await?;

        let any_tunnel_handle = AnyTunnelHandle::from(tunnel_handle);

        Ok((tunnel_conn_data, any_tunnel_handle))
    }

    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    async fn set_dns(&mut self, tun_name: &str) -> Result<()> {
        let dns_servers = self.tunnel_settings.dns.ip_addresses().to_vec();

        self.dns_handler
            .set(tun_name.to_owned(), dns_servers)
            .await
            .map_err(Error::SetDns)
    }

    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    async fn set_routes(&mut self, routing_config: RoutingConfig) -> Result<()> {
        self.route_handler
            .add_routes(routing_config)
            .await
            .map_err(Error::AddRoutes)?;

        Ok(())
    }

    #[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
    fn create_mixnet_device(interface_addresses: IpPair, mtu: u16) -> Result<AsyncDevice> {
        let mut tun_config = tun::Configuration::default();

        // rust-tun uses the same name for tunnel type.
        #[cfg(windows)]
        tun_config.name(MIXNET_WINTUN_NAME);

        tun_config
            .address(interface_addresses.ipv4)
            .mtu(i32::from(mtu))
            .up();

        #[cfg(target_os = "linux")]
        tun_config.platform(|platform_config| {
            platform_config.packet_information(false);
        });

        let tun_device = tun::create_as_async(&tun_config).map_err(Error::CreateTunDevice)?;

        let tun_name = tun_device
            .get_ref()
            .name()
            .map_err(Error::GetTunDeviceName)?;

        tun_ipv6::set_ipv6_addr(&tun_name, interface_addresses.ipv6)
            .map_err(Error::SetTunDeviceIpv6Addr)?;

        Ok(tun_device)
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    fn create_wireguard_device(
        interface_addresses: IpPair,
        destination: Option<Ipv4Addr>,
        mtu: u16,
    ) -> Result<AsyncDevice> {
        let mut tun_config = tun::Configuration::default();

        tun_config
            .address(interface_addresses.ipv4)
            .netmask(Ipv4Addr::BROADCAST)
            .mtu(i32::from(mtu))
            .up();

        if let Some(destination) = destination {
            tun_config.destination(destination);
        }

        #[cfg(target_os = "linux")]
        tun_config.platform(|platform_config| {
            platform_config.packet_information(false);
        });

        let tun_device = tun::create_as_async(&tun_config).map_err(Error::CreateTunDevice)?;

        let tun_name = tun_device
            .get_ref()
            .name()
            .map_err(Error::GetTunDeviceName)?;

        tun_ipv6::set_ipv6_addr(&tun_name, interface_addresses.ipv6)
            .map_err(Error::SetTunDeviceIpv6Addr)?;

        Ok(tun_device)
    }

    #[cfg(any(target_os = "ios", target_os = "android"))]
    async fn create_tun_device(
        &self,
        packet_tunnel_settings: tunnel_provider::tunnel_settings::TunnelSettings,
    ) -> Result<AsyncDevice> {
        #[cfg(target_os = "ios")]
        let owned_tun_fd =
            tunnel_provider::ios::interface::get_tun_fd().map_err(Error::LocateTunDevice)?;

        #[cfg(target_os = "android")]
        let owned_tun_fd = {
            let raw_tun_fd = self
                .tun_provider
                .configure_tunnel(packet_tunnel_settings.into_tunnel_network_settings())
                .map_err(|e| Error::ConfigureTunnelProvider(e.to_string()))?;
            unsafe { OwnedFd::from_raw_fd(raw_tun_fd) }
        };

        let mut tun_config = tun::Configuration::default();
        tun_config.raw_fd(owned_tun_fd.as_raw_fd());

        #[cfg(target_os = "ios")]
        {
            self.tun_provider
                .set_tunnel_network_settings(packet_tunnel_settings.into_tunnel_network_settings())
                .await
                .map_err(|e| Error::ConfigureTunnelProvider(e.to_string()))?
        }

        let device = tun::create_as_async(&tun_config).map_err(Error::CreateTunDevice)?;

        // Consume the owned fd, since the device is now responsible for closing the underlying raw fd.
        let _ = owned_tun_fd.into_raw_fd();

        Ok(device)
    }
}

fn wait_delay(retry_attempt: u32) -> Duration {
    let multiplier = retry_attempt.saturating_mul(DELAY_MULTIPLIER);
    let delay = INITIAL_WAIT_DELAY.saturating_mul(multiplier);
    cmp::min(delay, MAX_WAIT_DELAY)
}
