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

mod environment;
pub(crate) mod error;
mod favorites;
mod gateway_cache;
mod offline_monitor;

cfg_if::cfg_if! {
    if #[cfg(target_os = "macos")] {
        use std::{net::IpAddr, path::PathBuf};

        uniffi::use_remote_type!(nym_vpn_lib_types::IpAddr);
        uniffi::use_remote_type!(nym_vpn_lib_types::PathBuf);

        mod rpc;
    } else {
        #[cfg(target_os = "ios")]
        mod account;
        #[cfg(target_os = "android")]
        mod android_connectivity_monitor;
        #[cfg(target_os = "ios")]
        mod deeplink;
        mod logging;
        mod tunnel_provider;
        #[cfg(target_os = "ios")]
        mod vpn_account_storage;
        mod vpn_service;
        mod vpn_service_command_sender;

        use std::{
            net::{IpAddr, Ipv4Addr, Ipv6Addr},
            path::PathBuf,
            sync::{Arc, LazyLock},
        };

        use ipnetwork::{IpNetwork, Ipv4Network, Ipv6Network};
        use tokio::runtime::Runtime;

        use nym_vpn_lib_types::{
            DiagnosticRunParams, EntryPoint, ExitPoint, FrontingMode, GatewayIndependence,
            GatewaySelectionAlgorithmConfig, GeoExclusionSettings, MixnetTrafficConfig,
            NetworkStatisticsConfig, PrivyDerivationMessage, SplitTunnelSettings, UserAgent,
            VpnServiceConfig,
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

        static TOKIO_RUNTIME: LazyLock<Runtime> = LazyLock::new(|| {
            tokio::runtime::Builder::new_multi_thread()
                .worker_threads(10)
                .enable_all()
                .build()
                .expect("failed to initialize tokio runtime")
        });

        /// Get the message to be signed using the Privy signing API.
        #[allow(non_snake_case)]
        #[uniffi::export]
        pub fn getPrivyDerivationMessage() -> PrivyDerivationMessage {
            PrivyDerivationMessage {
                message: nym_vpn_lib::privy::message_to_sign(),
            }
        }

        #[allow(non_snake_case)]
        #[uniffi::export(async_runtime = "tokio")]
        pub async fn runDiagnostic(
            params: DiagnosticRunParams,
            environment: Arc<NymEnvironment>,
        ) -> Result<String, VpnError> {
            let network = environment.inner().clone();
            let report = TOKIO_RUNTIME
                .spawn(async move { nym_diagnostic::DiagnosticHandler::run(network, params).await })
                .await
                .map_err(VpnError::internal)?;
            serde_json::to_string_pretty(&report).map_err(VpnError::internal)
        }

        #[derive(uniffi::Record)]
        pub struct VPNConfig {
            /// Path to configuration directory on disk
            pub config_dir: PathBuf,

            /// Path to data directory on disk
            pub data_dir: PathBuf,

            /// Path to log directory on disk
            pub log_dir: PathBuf,

            pub entry_gateway: EntryPoint,
            pub exit_router: ExitPoint,
            pub enable_two_hop: bool,
            pub enable_bridges: bool,
            pub residential_exit: bool,
            pub enable_ad_blocking: bool,

            pub fronting_mode: FrontingMode,

            /// Custom DNS used when set.
            /// Leave empty to use default DNS servers.
            pub custom_dns: Vec<IpAddr>,

            pub mixnet_traffic: Option<MixnetTrafficConfig>,
            pub network_stats: Option<NetworkStatisticsConfig>,
            pub gateway_selection_algorithm_config: GatewaySelectionAlgorithmConfig,
            pub gateway_independence: GatewayIndependence,
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

                    enable_ad_blocking: self.enable_ad_blocking,
                    fronting_mode: self.fronting_mode,

                    // Always true on mobile platforms
                    netstack: true,

                    residential_exit: self.residential_exit,
                    enable_custom_dns: !self.custom_dns.is_empty(),
                    custom_dns: self.custom_dns.clone(),
                    min_gateway_vpn_performance: None,
                    mixnet_traffic: self.mixnet_traffic.clone().unwrap_or_default(),
                    network_stats: self.network_stats.unwrap_or_default(),
                    gateway_selection_algorithm_config: self.gateway_selection_algorithm_config.clone(),
                    gateway_independence: self.gateway_independence,

                    // Not available via vpn service on mobile platforms
                    split_tunnel: SplitTunnelSettings::default(),
                    geo_exclusion: GeoExclusionSettings::default(),
                })
            }
        }

        /// Initialize JNI global Android context.
        ///
        /// This is necessary for NDK libraries which use the global context.
        #[cfg(target_os = "android")]
        #[unsafe(no_mangle)]
        pub extern "C" fn Java_net_nymtech_nymvpn_NymVpnLib_initContext<'caller>(
            mut unowned_env: jni::EnvUnowned<'caller>,
            _class: jni::objects::JClass,
            ctx: jni::objects::JObject<'caller>,
        ) {
            unowned_env
                .with_env(|env| -> jni::errors::Result<_> {
                    let vm = env.get_java_vm()?;
                    // `ctx` is a local JNI reference, only valid for the duration of this call.
                    // Here we create a globally valid one and leak it.
                    let global_ctx = std::mem::ManuallyDrop::new(env.new_global_ref(ctx)?);
                    unsafe {
                        ndk_context::initialize_android_context(
                            vm.get_raw() as *mut std::ffi::c_void,
                            global_ctx.as_raw() as *mut std::ffi::c_void,
                        );
                    }
                    Ok(())
                })
                .resolve::<jni::errors::ThrowRuntimeExAndDefault>();
        }

        #[cfg(target_os = "android")]
        mod android_tls {
            use nym_http_api_client::{ReqwestClientBuilder, registry::ConfigRecord};
            use rustls::{ClientConfig, RootCertStore};
            use std::sync::Arc;

            fn configure_webpki_tls(builder: ReqwestClientBuilder) -> ReqwestClientBuilder {
                let root_store = RootCertStore {
                    roots: webpki_roots::TLS_SERVER_ROOTS.to_vec(),
                };

                let crypto_provider = rustls::crypto::CryptoProvider::get_default()
                    .unwrap_or(&Arc::new(rustls::crypto::ring::default_provider()))
                    .clone();

                let tls_config = ClientConfig::builder_with_provider(crypto_provider)
                    .with_safe_default_protocol_versions()
                    .expect("ring supports TLS 1.2 and 1.3")
                    .with_root_certificates(root_store)
                    .with_no_client_auth();

                builder.tls_backend_preconfigured(tls_config)
            }

            inventory::submit! {
                ConfigRecord {
                    priority: -100,
                    apply: configure_webpki_tls,
                }
            }
        }
    }
}
