// Copyright 2016-2025 Mullvad VPN AB. All Rights Reserved.
// Copyright 2025 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[macro_use]
mod ffi;

use crate::TunnelInterface;
use nym_dns::ResolvedDnsConfig;

use std::{ffi::CStr, net::IpAddr, ptr, sync::LazyLock};

use nym_common::ErrorExt;
use widestring::WideCString;
use windows::Win32::Globalization::{MultiByteToWideChar, CP_ACP, MULTI_BYTE_TO_WIDE_CHAR_FLAGS};

use self::winfw::*;
use super::{
    net::{AllowedEndpoint, AllowedTunnelTraffic},
    FirewallArguments, FirewallPolicy, InitialFirewallState,
};
use crate::FirewallPolicyError;

mod hyperv;

const HYPERV_LEAK_WARNING_MSG: &str = "Hyper-V (e.g. WSL machines) may leak in blocked states.";

// `COMLibrary` must be initialized for per thread, so use TLS
thread_local! {
    static WMI: Option<wmi::WMIConnection> = {
        let result = hyperv::init_wmi();
        if matches!(&result, Err(hyperv::Error::ObtainHyperVClass(_))) {
            tracing::warn!("The Hyper-V firewall is not available. {HYPERV_LEAK_WARNING_MSG}");
            return None;
        }
        consume_and_log_hyperv_err(
            "Initialize COM and WMI",
            result,
        )
    };
}

/// Enable or disable blocking Hyper-V rule
static BLOCK_HYPERV: LazyLock<bool> = LazyLock::new(|| {
    let enable = std::env::var("NYM_FIREWALL_BLOCK_HYPERV")
        .map(|v| v != "0")
        .unwrap_or(true);

    if !enable {
        tracing::debug!("Hyper-V block rule disabled by NYM_FIREWALL_BLOCK_HYPERV");
    }

    enable
});

/// Errors that can happen when configuring the Windows firewall.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Failure to initialize windows firewall module
    #[error("Failed to initialize windows firewall module")]
    Initialization,

    /// Failure to deinitialize windows firewall module
    #[error("Failed to deinitialize windows firewall module")]
    Deinitialization,

    /// Failure to apply a firewall _connecting_ policy
    #[error("Failed to apply connecting firewall policy")]
    ApplyingConnectingPolicy(#[source] FirewallPolicyError),

    /// Failure to apply a firewall _connected_ policy
    #[error("Failed to apply connected firewall policy")]
    ApplyingConnectedPolicy(#[source] FirewallPolicyError),

    /// Failure to apply firewall _blocked_ policy
    #[error("Failed to apply blocked firewall policy")]
    ApplyingBlockedPolicy(#[source] FirewallPolicyError),

    /// Failure to reset firewall policies
    #[error("Failed to reset firewall policies")]
    ResettingPolicy(#[source] FirewallPolicyError),
}

/// Timeout for acquiring the WFP transaction lock
const WINFW_TIMEOUT_SECONDS: u32 = 5;

const LOGGING_CONTEXT: &[u8] = b"WinFw\0";

/// The Windows implementation for the firewall.
pub struct Firewall(());

impl Firewall {
    pub fn from_args(args: FirewallArguments) -> Result<Self, Error> {
        if let InitialFirewallState::Blocked(allowed_endpoints) = args.initial_state {
            Self::initialize_blocked(&allowed_endpoints, args.allow_lan)
        } else {
            Self::new()
        }
    }

    pub fn new() -> Result<Self, Error> {
        unsafe {
            WinFw_Initialize(
                WINFW_TIMEOUT_SECONDS,
                Some(log_sink),
                LOGGING_CONTEXT.as_ptr(),
            )
            .into_result()?
        };

        tracing::trace!("Successfully initialized windows firewall module");
        Ok(Firewall(()))
    }

    fn initialize_blocked(
        allowed_endpoints: &[AllowedEndpoint],
        allow_lan: bool,
    ) -> Result<Self, Error> {
        let cfg = &WinFwSettings::new(allow_lan);
        let allowed_endpoint_containers = allowed_endpoints
            .iter()
            .cloned()
            .map(AllowedEndpointBridge::from)
            .collect::<Vec<_>>();
        let winfw_allowed_endpoints = allowed_endpoint_containers
            .iter()
            .map(|allowed_endpoint| allowed_endpoint.as_endpoint())
            .collect::<Vec<_>>();
        // todo: verify that this is correct way to pass array of pointers.
        let allowed_endpoints_refs = winfw_allowed_endpoints.iter().collect::<Vec<_>>();

        unsafe {
            WinFw_InitializeBlocked(
                WINFW_TIMEOUT_SECONDS,
                cfg,
                allowed_endpoints_refs.as_ptr() as _,
                allowed_endpoints_refs.len(),
                Some(log_sink),
                LOGGING_CONTEXT.as_ptr(),
            )
            .into_result()?
        };
        tracing::trace!("Successfully initialized windows firewall module to a blocking state");

        with_wmi_if_enabled(|wmi| {
            let result = hyperv::add_blocking_hyperv_firewall_rules(wmi);
            consume_and_log_hyperv_err("Add block-all Hyper-V filter", result);
        });

        Ok(Firewall(()))
    }

    pub fn apply_policy(&mut self, policy: FirewallPolicy) -> Result<(), Error> {
        let should_block_hyperv = matches!(
            policy,
            FirewallPolicy::Connecting { .. } | FirewallPolicy::Blocked { .. }
        );

        let apply_result = match policy {
            FirewallPolicy::Connecting {
                peer_endpoints,
                tunnel,
                allow_lan,
                dns_config,
                allowed_endpoints,
                allowed_entry_tunnel_traffic,
                allowed_exit_tunnel_traffic,
            } => {
                let cfg = WinFwSettings::new(allow_lan);

                self.set_connecting_state(
                    &peer_endpoints,
                    &cfg,
                    tunnel.as_ref(),
                    &dns_config,
                    &allowed_endpoints,
                    allowed_entry_tunnel_traffic,
                    allowed_exit_tunnel_traffic,
                )
            }
            FirewallPolicy::Connected {
                peer_endpoints,
                tunnel,
                allow_lan,
                dns_config,
                allowed_endpoints,
            } => {
                let cfg = &WinFwSettings::new(allow_lan);
                self.set_connected_state(
                    &peer_endpoints,
                    cfg,
                    &tunnel,
                    &dns_config,
                    &allowed_endpoints,
                )
            }
            FirewallPolicy::Blocked {
                allow_lan,
                allowed_endpoints,
            } => {
                let cfg = &WinFwSettings::new(allow_lan);

                let winfw_allowed_endpoint_containers = allowed_endpoints
                    .into_iter()
                    .map(AllowedEndpointBridge::from)
                    .collect::<Vec<_>>();

                self.set_blocked_state(cfg, &winfw_allowed_endpoint_containers)
            }
        };

        with_wmi_if_enabled(|wmi| {
            if should_block_hyperv {
                let result = hyperv::add_blocking_hyperv_firewall_rules(wmi);
                consume_and_log_hyperv_err("Add block-all Hyper-V filter", result);
            } else {
                let result = hyperv::remove_blocking_hyperv_firewall_rules(wmi);
                consume_and_log_hyperv_err("Remove block-all Hyper-V filter", result);
            }
        });

        apply_result
    }

    pub fn reset_policy(&mut self) -> Result<(), Error> {
        unsafe { WinFw_Reset().into_result().map_err(Error::ResettingPolicy) }?;

        with_wmi_if_enabled(|wmi| {
            let result = hyperv::remove_blocking_hyperv_firewall_rules(wmi);
            consume_and_log_hyperv_err("Remove block-all Hyper-V filter", result);
        });

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn set_connecting_state(
        &mut self,
        endpoints: &[AllowedEndpoint],
        winfw_settings: &WinFwSettings,
        tunnel_interface: Option<&TunnelInterface>,
        dns_config: &ResolvedDnsConfig,
        allowed_endpoints: &[AllowedEndpoint],
        allowed_entry_tunnel_traffic: AllowedTunnelTraffic,
        allowed_exit_tunnel_traffic: AllowedTunnelTraffic,
    ) -> Result<(), Error> {
        tracing::trace!("Applying 'connecting' firewall policy");

        let winfw_endpoint_containers = endpoints
            .iter()
            .cloned()
            .map(AllowedEndpointBridge::from)
            .collect::<Vec<_>>();
        let winfw_endpoints = winfw_endpoint_containers
            .iter()
            .map(|ep| ep.as_endpoint())
            .collect::<Vec<_>>();

        // todo: verify that this is correct way to pass array of pointers.
        let endpoint_refs = winfw_endpoints.iter().collect::<Vec<_>>();

        let (entry_interface_wstr, exit_interface_wstr) = match tunnel_interface {
            Some(TunnelInterface::One(tunnel)) => (
                None,
                Some(WideCString::from_str_truncate(&tunnel.interface)),
            ),
            Some(TunnelInterface::Two { entry, exit }) => {
                let entry_interface = WideCString::from_str_truncate(&entry.interface);
                let exit_interface = WideCString::from_str_truncate(&exit.interface);
                (Some(entry_interface), Some(exit_interface))
            }
            None => (None, None),
        };

        let allowed_entry_tunnel_traffic_bridge =
            AllowedTunnelTrafficBridge::from(allowed_entry_tunnel_traffic);
        let allowed_exit_tunnel_traffic_bridge =
            AllowedTunnelTrafficBridge::from(allowed_exit_tunnel_traffic);

        let allowed_endpoint_containers = allowed_endpoints
            .iter()
            .cloned()
            .map(AllowedEndpointBridge::from)
            .collect::<Vec<_>>();
        let winfw_allowed_endpoints = allowed_endpoint_containers
            .iter()
            .map(|allowed_endpoint| allowed_endpoint.as_endpoint())
            .collect::<Vec<_>>();
        // todo: verify that this is correct way to pass array of pointers.
        let allowed_endpoints_refs = winfw_allowed_endpoints.iter().collect::<Vec<_>>();

        let non_tunnel_dns_servers: Vec<WideCString> = dns_config
            .non_tunnel_config()
            .iter()
            .cloned()
            .map(widestring_ip)
            .collect();
        let non_tunnel_dns_servers_refs: Vec<*const u16> = non_tunnel_dns_servers
            .iter()
            .map(|ip| ip.as_ptr())
            .collect();

        unsafe {
            WinFw_ApplyPolicyConnecting(
                winfw_settings,
                endpoint_refs.as_ptr() as _,
                endpoint_refs.len(),
                entry_interface_wstr
                    .as_ref()
                    .map(|s| s.as_ptr())
                    .unwrap_or(ptr::null()),
                exit_interface_wstr
                    .as_ref()
                    .map(|s| s.as_ptr())
                    .unwrap_or(ptr::null()),
                allowed_endpoints_refs.as_ptr() as _,
                allowed_endpoints_refs.len(),
                allowed_entry_tunnel_traffic_bridge.as_inner_ref(),
                allowed_exit_tunnel_traffic_bridge.as_inner_ref(),
                non_tunnel_dns_servers_refs.as_ptr(),
                non_tunnel_dns_servers_refs.len(),
            )
            .into_result()
            .map_err(Error::ApplyingConnectingPolicy)
        }
    }

    fn set_connected_state(
        &mut self,
        endpoints: &[AllowedEndpoint],
        winfw_settings: &WinFwSettings,
        tunnel_interface: &TunnelInterface,
        dns_config: &ResolvedDnsConfig,
        allowed_endpoints: &[AllowedEndpoint],
    ) -> Result<(), Error> {
        tracing::trace!("Applying 'connected' firewall policy");

        let winfw_endpoint_containers = endpoints
            .iter()
            .cloned()
            .map(AllowedEndpointBridge::from)
            .collect::<Vec<_>>();
        let winfw_endpoints = winfw_endpoint_containers
            .iter()
            .map(|ep| ep.as_endpoint())
            .collect::<Vec<_>>();

        // todo: verify that this is correct way to pass array of pointers.
        let endpoint_refs = winfw_endpoints.iter().collect::<Vec<_>>();

        let (entry_interface_wstr, exit_interface_wstr) = match tunnel_interface {
            TunnelInterface::One(tunnel) => (
                None,
                Some(WideCString::from_str_truncate(&tunnel.interface)),
            ),
            TunnelInterface::Two { entry, exit } => {
                let entry_interface = WideCString::from_str_truncate(&entry.interface);
                let exit_interface = WideCString::from_str_truncate(&exit.interface);
                (Some(entry_interface), Some(exit_interface))
            }
        };

        let tunnel_dns_servers: Vec<WideCString> = dns_config
            .tunnel_config()
            .iter()
            .cloned()
            .map(widestring_ip)
            .collect();
        let tunnel_dns_servers_refs: Vec<*const u16> =
            tunnel_dns_servers.iter().map(|ip| ip.as_ptr()).collect();
        let non_tunnel_dns_servers: Vec<WideCString> = dns_config
            .non_tunnel_config()
            .iter()
            .cloned()
            .map(widestring_ip)
            .collect();
        let non_tunnel_dns_servers_refs: Vec<*const u16> = non_tunnel_dns_servers
            .iter()
            .map(|ip| ip.as_ptr())
            .collect();

        let winfw_allowed_endpoint_containers = allowed_endpoints
            .iter()
            .cloned()
            .map(AllowedEndpointBridge::from)
            .collect::<Vec<_>>();
        let winfw_allowed_endpoints = winfw_allowed_endpoint_containers
            .iter()
            .map(|ep| ep.as_endpoint())
            .collect::<Vec<_>>();

        // todo: verify that this is correct way to pass array of pointers.
        let allowed_endpoint_refs = winfw_allowed_endpoints.iter().collect::<Vec<_>>();

        unsafe {
            WinFw_ApplyPolicyConnected(
                winfw_settings,
                endpoint_refs.as_ptr() as _,
                endpoint_refs.len(),
                entry_interface_wstr
                    .as_ref()
                    .map(|s| s.as_ptr())
                    .unwrap_or(ptr::null()),
                exit_interface_wstr
                    .as_ref()
                    .map(|s| s.as_ptr())
                    .unwrap_or(ptr::null()),
                tunnel_dns_servers_refs.as_ptr(),
                tunnel_dns_servers_refs.len(),
                non_tunnel_dns_servers_refs.as_ptr(),
                non_tunnel_dns_servers_refs.len(),
                allowed_endpoint_refs.as_ptr() as _,
                allowed_endpoint_refs.len(),
            )
            .into_result()
            .map_err(Error::ApplyingConnectedPolicy)
        }
    }

    fn set_blocked_state(
        &mut self,
        winfw_settings: &WinFwSettings,
        allowed_endpoints: &[AllowedEndpointBridge],
    ) -> Result<(), Error> {
        tracing::trace!("Applying 'blocked' firewall policy");
        let endpoints = allowed_endpoints
            .iter()
            .map(AllowedEndpointBridge::as_endpoint)
            .collect::<Vec<_>>();
        let endpoint_refs = endpoints.iter().collect::<Vec<_>>();

        unsafe {
            WinFw_ApplyPolicyBlocked(winfw_settings, endpoint_refs.as_ptr() as _, endpoints.len())
                .into_result()
                .map_err(Error::ApplyingBlockedPolicy)
        }
    }
}

impl Drop for Firewall {
    fn drop(&mut self) {
        if unsafe {
            WinFw_Deinitialize(WinFwCleanupPolicy::ContinueBlocking)
                .into_result()
                .is_ok()
        } {
            tracing::trace!("Successfully deinitialized windows firewall module");
        } else {
            tracing::error!("Failed to deinitialize windows firewall module");
        };
    }
}

fn widestring_ip(ip: IpAddr) -> WideCString {
    WideCString::from_str_truncate(ip.to_string())
}

/// Logging callback implementation.
pub extern "system" fn log_sink(
    level: log::Level,
    msg: *const std::ffi::c_char,
    context: *mut std::ffi::c_void,
) {
    if msg.is_null() {
        tracing::error!("Log message from FFI boundary is NULL");
    } else {
        let target = if context.is_null() {
            "UNKNOWN".into()
        } else {
            unsafe { CStr::from_ptr(context as *const _).to_string_lossy() }
        };

        let mb_string = unsafe { CStr::from_ptr(msg) };

        let managed_msg = match multibyte_to_wide(mb_string, CP_ACP) {
            Ok(wide_str) => String::from_utf16_lossy(&wide_str),
            // Best effort:
            Err(_) => mb_string.to_string_lossy().into_owned(),
        };

        log::logger().log(
            &log::Record::builder()
                .level(level)
                .target(&target)
                .args(format_args!("{}", managed_msg))
                .build(),
        );
    }
}

/// Convert `mb_string`, with the given character encoding `codepage`, to a UTF-16 string.
fn multibyte_to_wide(mb_string: &CStr, codepage: u32) -> Result<Vec<u16>, windows::core::Error> {
    if mb_string.is_empty() {
        return Ok(vec![]);
    }

    // SAFETY: `mb_string` is null-terminated and valid.
    let wc_size = unsafe {
        MultiByteToWideChar(
            codepage,
            MULTI_BYTE_TO_WIDE_CHAR_FLAGS::default(),
            mb_string.to_bytes_with_nul(),
            None,
        )
    };

    if wc_size == 0 {
        return Err(windows::core::Error::from_win32());
    }

    let wc_buffer_len = usize::try_from(wc_size).unwrap();
    let mut wc_buffer = vec![0u16; wc_buffer_len];

    // SAFETY: `wc_buffer` can contain up to `wc_size` characters, including a null
    // terminator.
    let chars_written = unsafe {
        MultiByteToWideChar(
            codepage,
            MULTI_BYTE_TO_WIDE_CHAR_FLAGS::default(),
            mb_string.to_bytes_with_nul(),
            Some(&mut wc_buffer),
        )
    };

    if chars_written == 0 {
        return Err(windows::core::Error::from_win32());
    }

    wc_buffer.truncate(usize::try_from(chars_written - 1).unwrap());

    Ok(wc_buffer)
}

#[cfg(test)]
mod test {
    use super::multibyte_to_wide;
    use windows::Win32::Globalization::CP_UTF8;

    #[test]
    fn test_multibyte_to_wide() {
        // € = 0x20AC in UTF-16
        let converted = multibyte_to_wide(c"€€", CP_UTF8);
        const EXPECTED: &[u16] = &[0x20AC, 0x20AC];
        assert!(
            matches!(converted.as_deref(), Ok(EXPECTED)),
            "expected Ok({EXPECTED:?}), got {converted:?}",
        );

        // boundary case
        let converted = multibyte_to_wide(c"", CP_UTF8);
        assert!(
            matches!(converted.as_deref(), Ok([])),
            "unexpected result {converted:?}"
        );
    }
}

// Convert `result` into an option and log the error, if any.
fn consume_and_log_hyperv_err<T>(
    action: &'static str,
    result: Result<T, hyperv::Error>,
) -> Option<T> {
    result
        .inspect_err(|error| {
            tracing::error!(
                "{}",
                error.display_chain_with_msg(&format!("{action}. {HYPERV_LEAK_WARNING_MSG}"))
            );
        })
        .ok()
}

// Run a closure with the current thread's WMI connection, if available
fn with_wmi_if_enabled(f: impl FnOnce(&wmi::WMIConnection)) {
    if !*BLOCK_HYPERV {
        return;
    }
    WMI.with(|wmi| {
        if let Some(con) = wmi {
            f(con)
        }
    })
}

#[allow(non_snake_case)]
mod winfw {
    use super::{widestring_ip, AllowedEndpoint, AllowedTunnelTraffic, Error, WideCString};
    use crate::net::TransportProtocol;
    use std::{
        ffi::{c_char, c_void},
        ptr,
    };

    type LogSink = extern "system" fn(level: log::Level, msg: *const c_char, context: *mut c_void);

    pub struct AllowedEndpointBridge {
        _clients: Box<[WideCString]>,
        clients_ptrs: Box<[*const u16]>,
        ip: WideCString,
        port: u16,
        protocol: WinFwProt,
    }

    impl From<AllowedEndpoint> for AllowedEndpointBridge {
        fn from(endpoint: AllowedEndpoint) -> Self {
            let clients = endpoint
                .clients
                .iter()
                .map(WideCString::from_os_str_truncate)
                .collect::<Box<_>>();
            let clients_ptrs = clients
                .iter()
                .map(|client| client.as_ptr())
                .collect::<Box<_>>();
            let ip = widestring_ip(endpoint.endpoint.address.ip());

            AllowedEndpointBridge {
                _clients: clients,
                clients_ptrs,
                ip,
                port: endpoint.endpoint.address.port(),
                protocol: WinFwProt::from(endpoint.endpoint.protocol),
            }
        }
    }

    impl AllowedEndpointBridge {
        pub fn as_endpoint(&self) -> WinFwAllowedEndpoint<'_> {
            WinFwAllowedEndpoint {
                num_clients: self.clients_ptrs.len() as u32,
                clients: self.clients_ptrs.as_ptr(),
                endpoint: WinFwEndpoint {
                    ip: self.ip.as_ptr(),
                    port: self.port,
                    protocol: self.protocol,
                },

                _phantom: std::marker::PhantomData,
            }
        }
    }

    /// Bridging type used that ensures memory safety when converting from `crate::Endpoint` to `WinFwEndpoint`.
    pub struct EndpointBridge {
        address: WideCString,
        inner: WinFwEndpoint,
    }

    impl From<crate::Endpoint> for EndpointBridge {
        fn from(value: crate::Endpoint) -> Self {
            let mut ep = Self {
                address: WideCString::from_str_truncate(value.address.ip().to_string()),
                inner: WinFwEndpoint {
                    ip: ptr::null(),
                    port: value.address.port(),
                    protocol: WinFwProt::from(value.protocol),
                },
            };
            ep.inner.ip = ep.address.as_ptr();
            ep
        }
    }

    impl EndpointBridge {
        fn as_ptr(&self) -> *const WinFwEndpoint {
            &self.inner
        }
    }

    pub struct AllowedTunnelTrafficBridge {
        endpoint1: Option<EndpointBridge>,
        endpoint2: Option<EndpointBridge>,
        inner: WinFwAllowedTunnelTraffic,
    }

    impl AllowedTunnelTrafficBridge {
        pub fn as_inner_ref(&self) -> &WinFwAllowedTunnelTraffic {
            &self.inner
        }
    }

    impl From<AllowedTunnelTraffic> for AllowedTunnelTrafficBridge {
        fn from(value: AllowedTunnelTraffic) -> Self {
            let (endpoint1, endpoint2) = match value {
                AllowedTunnelTraffic::None | AllowedTunnelTraffic::All => (None, None),
                AllowedTunnelTraffic::One(endpoint) => (Some(EndpointBridge::from(endpoint)), None),
                AllowedTunnelTraffic::Two(endpoint1, endpoint2) => (
                    Some(EndpointBridge::from(endpoint1)),
                    Some(EndpointBridge::from(endpoint2)),
                ),
            };

            let mut allowed_tunnel_traffic_bridge = Self {
                endpoint1,
                endpoint2,
                inner: WinFwAllowedTunnelTraffic {
                    type_: WinFwAllowedTunnelTrafficType::from(&value),
                    endpoint1: ptr::null(),
                    endpoint2: ptr::null(),
                },
            };
            allowed_tunnel_traffic_bridge.inner.endpoint1 = allowed_tunnel_traffic_bridge
                .endpoint1
                .as_ref()
                .map(|s| s.as_ptr())
                .unwrap_or(ptr::null());
            allowed_tunnel_traffic_bridge.inner.endpoint2 = allowed_tunnel_traffic_bridge
                .endpoint2
                .as_ref()
                .map(|s| s.as_ptr())
                .unwrap_or(ptr::null());

            allowed_tunnel_traffic_bridge
        }
    }

    #[repr(C)]
    pub struct WinFwAllowedEndpoint<'a> {
        num_clients: u32,
        clients: *const *const libc::wchar_t,
        endpoint: WinFwEndpoint,

        _phantom: std::marker::PhantomData<&'a AllowedEndpointBridge>,
    }

    #[repr(C)]
    pub struct WinFwAllowedTunnelTraffic {
        pub type_: WinFwAllowedTunnelTrafficType,
        pub endpoint1: *const WinFwEndpoint,
        pub endpoint2: *const WinFwEndpoint,
    }

    #[repr(u8)]
    #[derive(Clone, Copy)]
    pub enum WinFwAllowedTunnelTrafficType {
        None,
        All,
        One,
        Two,
    }

    impl From<&AllowedTunnelTraffic> for WinFwAllowedTunnelTrafficType {
        fn from(traffic: &AllowedTunnelTraffic) -> Self {
            match traffic {
                AllowedTunnelTraffic::None => WinFwAllowedTunnelTrafficType::None,
                AllowedTunnelTraffic::All => WinFwAllowedTunnelTrafficType::All,
                AllowedTunnelTraffic::One(..) => WinFwAllowedTunnelTrafficType::One,
                AllowedTunnelTraffic::Two(..) => WinFwAllowedTunnelTrafficType::Two,
            }
        }
    }

    #[repr(C)]
    pub struct WinFwEndpoint {
        pub ip: *const libc::wchar_t,
        pub port: u16,
        pub protocol: WinFwProt,
    }

    #[repr(u8)]
    #[derive(Clone, Copy)]
    pub enum WinFwProt {
        Tcp = 0u8,
        Udp = 1u8,
    }

    impl From<TransportProtocol> for WinFwProt {
        fn from(prot: TransportProtocol) -> WinFwProt {
            match prot {
                TransportProtocol::Tcp => WinFwProt::Tcp,
                TransportProtocol::Udp => WinFwProt::Udp,
            }
        }
    }

    #[repr(C)]
    pub struct WinFwSettings {
        permitDhcp: bool,
        permitLan: bool,
    }

    impl WinFwSettings {
        pub fn new(permit_lan: bool) -> WinFwSettings {
            WinFwSettings {
                permitDhcp: true,
                permitLan: permit_lan,
            }
        }
    }

    #[allow(dead_code)]
    #[repr(u32)]
    #[derive(Clone, Copy)]
    pub enum WinFwCleanupPolicy {
        ContinueBlocking = 0,
        ResetFirewall = 1,
    }

    ffi_error!(InitializationResult, Error::Initialization);
    ffi_error!(DeinitializationResult, Error::Deinitialization);

    #[derive(Debug)]
    #[allow(dead_code)]
    #[repr(u32)]
    pub enum WinFwPolicyStatus {
        Success = 0,
        GeneralFailure = 1,
        LockTimeout = 2,
    }

    impl WinFwPolicyStatus {
        pub fn into_result(self) -> Result<(), super::FirewallPolicyError> {
            match self {
                WinFwPolicyStatus::Success => Ok(()),
                WinFwPolicyStatus::GeneralFailure => Err(super::FirewallPolicyError::Generic),
                WinFwPolicyStatus::LockTimeout => {
                    // TODO: Obtain application name and string from WinFw
                    Err(super::FirewallPolicyError::Locked(None))
                }
            }
        }
    }

    impl From<WinFwPolicyStatus> for Result<(), super::FirewallPolicyError> {
        fn from(val: WinFwPolicyStatus) -> Self {
            val.into_result()
        }
    }

    extern "system" {
        #[link_name = "WinFw_Initialize"]
        pub fn WinFw_Initialize(
            timeout: libc::c_uint,
            sink: Option<LogSink>,
            sink_context: *const u8,
        ) -> InitializationResult;

        #[link_name = "WinFw_InitializeBlocked"]
        pub fn WinFw_InitializeBlocked(
            timeout: libc::c_uint,
            settings: &WinFwSettings,
            allowedEndpoints: *const *const WinFwAllowedEndpoint<'_>,
            numAllowedEndpoints: usize,
            sink: Option<LogSink>,
            sink_context: *const u8,
        ) -> InitializationResult;

        #[link_name = "WinFw_Deinitialize"]
        pub fn WinFw_Deinitialize(cleanupPolicy: WinFwCleanupPolicy) -> DeinitializationResult;

        #[link_name = "WinFw_ApplyPolicyConnecting"]
        pub fn WinFw_ApplyPolicyConnecting(
            settings: &WinFwSettings,
            relays: *const *const WinFwAllowedEndpoint,
            numRelays: usize,
            entryTunnelIfaceAlias: *const libc::wchar_t,
            exitTunnelIfaceAlias: *const libc::wchar_t,
            allowedEndpoints: *const *const WinFwAllowedEndpoint<'_>,
            numAllowedEndpoints: usize,
            allowedEntryTunnelTraffic: &WinFwAllowedTunnelTraffic,
            allowedExitTunnelTraffic: &WinFwAllowedTunnelTraffic,
            nonTunnelDnsServers: *const *const libc::wchar_t,
            numNonTunnelDnsServers: usize,
        ) -> WinFwPolicyStatus;

        #[link_name = "WinFw_ApplyPolicyConnected"]
        pub fn WinFw_ApplyPolicyConnected(
            settings: &WinFwSettings,
            relays: *const *const WinFwAllowedEndpoint,
            numRelays: usize,
            entryTunnelIfaceAlias: *const libc::wchar_t,
            exitTunnelIfaceAlias: *const libc::wchar_t,
            tunnelDnsServers: *const *const libc::wchar_t,
            numTunnelDnsServers: usize,
            nonTunnelDnsServers: *const *const libc::wchar_t,
            numNonTunnelDnsServers: usize,
            allowedEndpoints: *const *const WinFwAllowedEndpoint<'_>,
            numAllowedEndpoints: usize,
        ) -> WinFwPolicyStatus;

        #[link_name = "WinFw_ApplyPolicyBlocked"]
        pub fn WinFw_ApplyPolicyBlocked(
            settings: &WinFwSettings,
            allowedEndpoints: *const *const WinFwAllowedEndpoint<'_>,
            numAllowedEndpoints: usize,
        ) -> WinFwPolicyStatus;

        #[link_name = "WinFw_Reset"]
        pub fn WinFw_Reset() -> WinFwPolicyStatus;
    }
}
