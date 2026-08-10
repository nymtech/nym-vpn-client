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
//! // Configure logging
//! initLogger(logDir: "/path/to/log/dir", logLevel: LogLevel::Info, sentryMonitoring: true)
//! ```
//!
//! ## Environment initialization
//!
//! Initialize the environment: `NymEnvironment::new_with_cache_dir(cache_dir, network_name)` or `NymEnvironment::new_with_mainnet_fallback()`.
//!
//! ```swift
//! import NymVpnLib
//!
//! // create mainnet fallback environment
//! let mainnetFallback = try! await NymEnvironment.newWithMainnetFallback()
//!
//! // create environment with cache dir
//! let userAgent = UserAgent {
//!     application: "MyApp",
//!     version: "1.0",
//!     platform: "ios",
//!     git_commit: "",
//! }
//! let environment = try! await NymEnvironment.newWithCacheDir(
//!     cacheDir: "/path/to/cache/dir",
//!     networkName: "mainnet",
//!     userAgent: userAgent
//! )
//! ```
//!
//! ## Query gateways
//!
//! You can query gateways using `NymGatewayCache` which can be used directly in absence of `NymVpnService`.
//!
//! ```swift
//! import NymVpnLib
//!
//! let userAgent = UserAgent {
//!     application: "MyApp",
//!     version: "1.0",
//!     platform: "ios",
//!     git_commit: "",
//! }
//! let environment = try! await NymEnvironment.newWithCacheDir(
//!     cacheDir: "/path/to/cache/dir",
//!     networkName: "mainnet",
//!     userAgent: userAgent
//! )
//! let offlineMonitor = await NymOfflineMonitor()
//!
//! let gatewayCache = try await NymGatewayCache(
//!     userAgent: userAgent,
//!     environment: environment,
//!     offlineMonitor: offlineMonitor
//! )
//! let gateways = try! await gatewayCache.getGateways(gwType: .wg);
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
//! let userAgent = UserAgent {
//!     application: "MyApp",
//!     version: "1.0",
//!     platform: "ios",
//!     git_commit: "",
//! }
//! let environment = try! await NymEnvironment.newWithCacheDir(
//!     cacheDir: "/path/to/cache/dir",
//!     networkName: "mainnet",
//!     userAgent: userAgent
//! )
//! let accountStorage = try! await NymVpnAccountStorage(dataDir: "/path/to/config/dir", environment: environment)
//!
//! try! await accountStorage.login(request: .vpn(mnemonic: "my awesome mnemonic!"))
//! ```
//!
//! ## Interact with account controller
//!
//! You can use `NymAccountController` to manage VPN accounts in absence of `NymVpnService` which runs account controller behind the scenes.
//!
//! ```swift
//! import NymVpnLib
//!
//! let userAgent = UserAgent {
//!     application: "MyApp",
//!     version: "1.0",
//!     platform: "ios",
//!     git_commit: "",
//! }
//! let environment = try! await NymEnvironment.newWithCacheDir(
//!     cacheDir: "/path/to/cache/dir",
//!     networkName: "mainnet",
//!     userAgent: userAgent
//! )
//! let offlineMonitor = await NymOfflineMonitor()
//! let accountController = try! await NymAccountController(
//!     data_dir: "/path/to/data/dir",
//!     userAgent: userAgent,
//!     environment: environment,
//!     offlineMonitor: offlineMonitor
//! );
//!
//! // Log in
//! try! await accountController.login(request: .vpn(mnemonic: "my awesome mnemonic!"))
//!
//! // Wait for account to be ready to connect
//! try! await accountController.waitForAccountReadyToConnect(timeout: 3600)
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
//! let vpnService = NymVpnService.newService(
//!     config: config,
//!     environment: environment,
//!     eventListener: tunnelEventHandler
//! )
//!
//! // Create command sender
//! let commandSender = vpnService.getCommandSender()
//!
//! // Manage VPN service
//! try! await commandSender.setEnableTwoHop(enableTwoHop: true)
//! try! await commandSender.connectTunnel()
//!
//! // When no longer needed, release the VPN service
//! await vpnService.shutdownAndWait()
//! ```

#![cfg(any(target_os = "android", target_os = "ios", target_os = "macos"))]
uniffi::setup_scaffolding!();

use std::{net::IpAddr, path::PathBuf};

uniffi::use_remote_type!(nym_vpn_lib_types::IpAddr);
uniffi::use_remote_type!(nym_vpn_lib_types::PathBuf);

mod environment;
pub(crate) mod error;
mod favorites;
mod gateway_cache;
mod offline_monitor;

#[cfg(target_os = "macos")]
mod rpc;

#[cfg(target_os = "ios")]
mod account;
#[cfg(target_os = "android")]
mod android_connectivity_monitor;
#[cfg(target_os = "ios")]
mod deeplink;
#[cfg(any(target_os = "android", target_os = "ios"))]
mod logging;
#[cfg(any(target_os = "android", target_os = "ios"))]
mod mobile;
#[cfg(any(target_os = "android", target_os = "ios"))]
mod tunnel_provider;
#[cfg(target_os = "ios")]
mod vpn_account_storage;
#[cfg(any(target_os = "android", target_os = "ios"))]
mod vpn_service;
#[cfg(any(target_os = "android", target_os = "ios"))]
mod vpn_service_command_sender;

#[cfg(any(target_os = "android", target_os = "ios"))]
pub use mobile::*;
