// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! The Uniffi generated bindings for the Nym VPN library. The API is designed to be used by
//! frontends to interact with the Nym VPN library. The API is designed to be platform-agnostic and
//! should work on any platform that supports the Uniffi FFI bindings.
//!
//! ## Library initialization
//!
//! Before calling any functions or creating objects exposed by this library, you must initialize the library and configure logging.
//! Do this as early as possible in your application's lifecycle.
//!
//! ```swift
//! import NymVpnLib
//!
//! // Somewhere in main()
//! // Setup tokio runtime used by Rust library
//! initializeTokioRuntime()
//!
//! // Configure logging
//! await initLogger("/path/to/log/dir", LogLevel::Info, /* enable sentry */ true)
//! ```
//!
//! ## Environment initialization
//!
//! Initialize the environment: `NymEnvironment::new_with_cache_dir(cache_dir, network_name)` or `NymEnvironment::new_with_mainnet_fallback()`.
//!
//! ## Query gateways
//!
//! You can query gateways using `NymGatewayCache` which can be used directly in absence of `NymVpnService`.
//!
//! ```swift
//! import NymVpnLib
//!
//! let environment = try! await NymEnvironment("/path/to/config/dir", "mainnet");
//! let offlineMonitor = try! await OfflineMonitor);
//!
//! let userAgent = UserAgent {
//!     application: "MyApp",
//!     version: "1.0",
//!     platform: "ios",
//!     git_commit: "",
//! }
//! let gatewayCache = await NymGatewayCache(userAgent, environment, offlineMonitor);
//! let gateways = try! await gatewayCache.getGateways(GatewayType::Wg);
//!
//! // Destroy gateway cache when no longer needed
//! // Or simply release all references to it
//! await gatewayCache.shutdownAndWait();
//! ```
//!
//! ## Interact with VPN account storage
//!
//! You can directly manipulate the VPN account storage using `NymVpnAccountStorage` when neither `NymAccountController` nor `NymVpnService` are available.
//!
//! ```swift
//! import NymVpnLib
//!
//! let environment = try! await NymEnvironment("/path/to/config/dir", "mainnet")
//! let accountStorage = try! await NymVpnAccountStorage("/path/to/config/dir", environment)
//!
//! try! await accountStorage.login(.vpn(mnemonic: "my awesome mnemonic!"))
//! ```
//!
//! ## Interact with account controller
//!
//! You can use `NymAccountController` to manage VPN accounts in absence of `NymVpnService` which runs account controller behind the scenes.
//!
//! ```swift
//! import NymVpnLib
//!
//! let environment = try! await NymEnvironment("/path/to/config/dir", "mainnet")
//! let offlineMonitor = try! await OfflineMonitor()
//!
//! let userAgent = UserAgent {
//!     application: "MyApp",
//!     version: "1.0",
//!     platform: "ios",
//!     git_commit: "",
//! }
//! let accountController = try! await NymAccountController("/path/to/data/dir", userAgent, environment, offlineMonitor);
//!
//! // Log in
//! try! await accountController.login(.vpn(mnemonic: "my awesome mnemonic!"))
//!
//! // Wait for account to be ready to connect
//! try! await accountController.waitForAccountReadyToConnect()
//!
//! // Destroy account controller when no longer needed
//! await accountController.shutdownAndWait()
//! ```
//!
//! ## Start `NymVpnService` to control the tunnel
//!
//! ```swift
//! // Define event handler
//! class TunnelEventHandler: NSObject, TunnelStatusListener {
//!     init() {}
//!
//!     func onEvent(event: TunnelEvent) {
//!         // todo: handle event
//!     }
//! }
//!
//! let config = VPNConfig {
//!    // omitted for brevity
//! }
//!
//! let environment = try! await NymEnvironment("/path/to/config/dir", "mainnet");
//! let tunnelEventHandler = TunnelStatusListener()
//!
//! // Create VPN service and retain it throughout the application lifecycle
//! let vpnService = NymVpnService(config, environment, tunnelEventHandler)
//!
//! // Create command sender
//! let commandSender = vpnService.getCommandSender()
//!
//! // Manage VPN service
//! try! await commandSender.setEnableTwoHop(true)
//! try! await commandSender.connectTunnel()
//!
//! // When no longer needed, release the VPN service
//! await vpnService.shutdownAndWait()
//! ```

#![cfg(any(target_os = "android", target_os = "ios"))]

uniffi::setup_scaffolding!();

pub(crate) mod error;

mod account;
#[cfg(target_os = "android")]
mod android_connectivity_monitor;
mod environment;
mod gateway_cache;
mod logging;
mod offline_monitor;
mod tunnel_provider;
#[cfg(target_os = "android")]
mod vpn_account_storage;
mod vpn_service;
mod vpn_service_command_sender;

use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    path::PathBuf,
    sync::{Arc, OnceLock},
};

use ipnetwork::{IpNetwork, Ipv4Network, Ipv6Network};
use tokio::runtime::Runtime;

use nym_vpn_lib_types::{
    EntryPoint, ExitPoint, MixnetTrafficConfig, NetworkStatisticsConfig, PrivyDerivationMessage,
    UserAgent, VpnServiceConfig,
};

#[cfg(target_os = "android")]
use android_connectivity_monitor::AndroidConnectivityMonitor;
use environment::NymEnvironment;
use error::VpnError;
#[cfg(target_os = "android")]
use tunnel_provider::android::AndroidTunProvider;
#[cfg(target_os = "ios")]
use tunnel_provider::ios::OSTunProvider;

uniffi::use_remote_type!(nym_vpn_lib_types::IpAddr);
uniffi::use_remote_type!(nym_vpn_lib_types::Ipv4Addr);
uniffi::use_remote_type!(nym_vpn_lib_types::Ipv6Addr);
uniffi::use_remote_type!(nym_vpn_lib_types::IpNetwork);
uniffi::use_remote_type!(nym_vpn_lib_types::Ipv4Network);
uniffi::use_remote_type!(nym_vpn_lib_types::Ipv6Network);
uniffi::use_remote_type!(nym_vpn_lib_types::PathBuf);

static TOKIO_RUNTIME: OnceLock<Runtime> = OnceLock::new();

/// Initialize tokio runtime once before interacting with nym-vpn-lib.
/// Repeat calls do nothing.
#[allow(non_snake_case)]
#[uniffi::export]
pub fn initializeTokioRuntime() {
    let _rt = TOKIO_RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(10)
            .enable_all()
            .build()
            .expect("failed to initialize tokio runtime")
    });
}

/// Get the message to be signed using the Privy signing API.
#[allow(non_snake_case)]
#[uniffi::export]
pub fn getPrivyDerivationMessage() -> PrivyDerivationMessage {
    PrivyDerivationMessage {
        message: nym_vpn_lib::login::privy::message_to_sign(),
    }
}

#[derive(uniffi::Record)]
pub struct VPNConfig {
    /// Path to configuration directory on disk
    pub config_dir: PathBuf,
    /// Path to data directory on disk
    pub data_dir: PathBuf,
    pub entry_gateway: EntryPoint,
    pub exit_router: ExitPoint,
    pub enable_two_hop: bool,
    pub enable_bridges: bool,
    pub enable_lewes_protocol: bool,
    pub residential_exit: bool,
    /// Custom DNS used when set.
    /// Leave empty to use default DNS servers.
    pub custom_dns: Vec<IpAddr>,
    pub user_agent: UserAgent,
    #[cfg(target_os = "ios")]
    tun_provider: Arc<dyn OSTunProvider>,
    #[cfg(target_os = "android")]
    tun_provider: Arc<dyn AndroidTunProvider>,
    #[cfg(target_os = "android")]
    connectivity_monitor: Arc<dyn AndroidConnectivityMonitor>,
}

impl VPNConfig {
    fn as_vpn_service_config(&self) -> Box<VpnServiceConfig> {
        Box::new(VpnServiceConfig {
            entry_point: self.entry_gateway.clone(),
            exit_point: self.exit_router.clone(),
            // Does not have effect on mobile platforms
            allow_lan: true,
            disable_ipv6: false,
            enable_two_hop: self.enable_two_hop,
            enable_bridges: self.enable_bridges,
            enable_lewes_protocol: self.enable_lewes_protocol,
            // Always true on mobile platforms
            netstack: true,
            residential_exit: self.residential_exit,
            enable_custom_dns: !self.custom_dns.is_empty(),
            custom_dns: self.custom_dns.clone(),
            min_gateway_vpn_performance: None,
            mixnet_traffic: MixnetTrafficConfig::default(),
            network_stats: NetworkStatisticsConfig::default(),
        })
    }
}
