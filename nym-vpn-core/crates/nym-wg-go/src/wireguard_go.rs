// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(unix)]
use std::os::fd::{IntoRawFd, OwnedFd, RawFd};
use std::{
    ffi::{c_char, c_void, CStr, CString},
    fmt,
};

#[cfg(windows)]
use nym_windows::net::AddressFamily;
#[cfg(windows)]
use windows::Win32::NetworkManagement::Ndis::NET_LUID_LH;

use super::{
    uapi::UapiConfigBuilder, Error, LoggingCallback, PeerConfig, PeerEndpointUpdate, PrivateKey,
    Result,
};
#[cfg(feature = "amnezia")]
use crate::amnezia::AmneziaConfig;

/// Classic WireGuard interface configuration.
pub struct InterfaceConfig {
    pub listen_port: Option<u16>,
    pub private_key: PrivateKey,
    pub mtu: u16,
    #[cfg(target_os = "linux")]
    pub fwmark: Option<u32>,
    #[cfg(feature = "amnezia")]
    /// Amnezia wireguard configuration disabled by default
    pub azwg_config: Option<AmneziaConfig>,
}

impl fmt::Debug for InterfaceConfig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut d = f.debug_struct("InterfaceConfig");
        d.field("listen_port", &self.listen_port)
            .field("private_key", &"(hidden)")
            .field("mtu", &self.mtu);
        #[cfg(target_os = "linux")]
        d.field("fwmark", &self.fwmark);
        #[cfg(feature = "amnezia")]
        d.field("azwg_config", &self.azwg_config);
        d.finish()
    }
}

/// Classic WireGuard configuration.
#[derive(Debug)]
pub struct Config {
    pub interface: InterfaceConfig,
    pub peers: Vec<PeerConfig>,
}

impl Config {
    fn as_uapi_config(&self) -> Vec<u8> {
        let mut config_builder = UapiConfigBuilder::new();
        config_builder.add(
            "private_key",
            self.interface.private_key.to_bytes().as_ref(),
        );

        if let Some(listen_port) = self.interface.listen_port {
            config_builder.add("listen_port", listen_port.to_string().as_str());
        }

        #[cfg(target_os = "linux")]
        if let Some(fwmark) = self.interface.fwmark {
            config_builder.add("fwmark", fwmark.to_string().as_str());
        }

        #[cfg(feature = "amnezia")]
        if let Some(azwg_config) = &self.interface.azwg_config {
            azwg_config.append_to(&mut config_builder);
        }

        if !self.peers.is_empty() {
            config_builder.add("replace_peers", "true");
            for peer in self.peers.iter() {
                peer.append_to(&mut config_builder);
            }
        }

        config_builder.into_bytes()
    }
}

/// Wintun interface created by wireguard-go
#[cfg(windows)]
#[derive(Debug, Clone)]
pub struct WintunInterface {
    /// Interface name.
    pub name: String,

    /// Interface LUID.
    pub luid: u64,
}

#[cfg(windows)]
impl WintunInterface {
    pub fn windows_luid(&self) -> NET_LUID_LH {
        // SAFETY: this is safe since NET_LUID_LH is a union represented by u64 value.
        unsafe { std::mem::transmute(self.luid) }
    }
}

/// Classic WireGuard tunnel.
#[derive(Debug)]
pub struct Tunnel {
    handle: i32,
    #[cfg(windows)]
    wintun_interface: WintunInterface,
}

impl Tunnel {
    /// Start new WireGuard tunnel
    #[cfg(not(windows))]
    pub fn start(config: Config, tun_fd: OwnedFd) -> Result<Self> {
        let settings =
            CString::new(config.as_uapi_config()).map_err(|_| Error::ConfigContainsNulByte)?;

        let handle = unsafe {
            wgTurnOn(
                // note: not all platforms accept mtu = 0
                #[cfg(any(target_os = "linux", target_os = "macos"))]
                i32::from(config.interface.mtu),
                settings.as_ptr(),
                tun_fd.into_raw_fd(),
                wg_logger_callback,
                std::ptr::null_mut(),
            )
        };

        if handle >= 0 {
            Ok(Self { handle })
        } else {
            Err(Error::StartTunnel(handle))
        }
    }

    #[cfg(windows)]
    /// Start new WireGuard tunnel
    pub fn start(
        config: Config,
        interface_name: &str,
        requested_guid: &str,
        wintun_tunnel_type: &str,
    ) -> Result<Self> {
        let settings =
            CString::new(config.as_uapi_config()).map_err(|_| Error::ConfigContainsNulByte)?;
        let interface_name_cstr =
            CString::new(interface_name).map_err(|_| Error::InterfaceNameContainsNulByte)?;
        let requested_guid_cstr =
            CString::new(requested_guid).map_err(|_| Error::RequestedGuidContainsNulByte)?;
        let wintun_tunnel_type_cstr =
            CString::new(wintun_tunnel_type).map_err(|_| Error::WintunTunnelTypeContainsNulByte)?;

        let mut out_interface_name: *mut c_char = std::ptr::null_mut();
        let out_interface_name_ptr: *mut *mut c_char = &mut out_interface_name;

        let mut out_interface_luid: u64 = 0;
        let out_interface_luid_ptr: *mut u64 = &mut out_interface_luid;

        let handle = unsafe {
            wgTurnOn(
                interface_name_cstr.as_ptr(),
                requested_guid_cstr.as_ptr(),
                wintun_tunnel_type_cstr.as_ptr(),
                i32::from(config.interface.mtu),
                settings.as_ptr(),
                out_interface_name_ptr,
                out_interface_luid_ptr,
                wg_logger_callback,
                std::ptr::null_mut(),
            )
        };

        if handle >= 0 {
            // SAFETY: libwg is expected to set a non-null value upon successful return.
            let wintun_iface_name_cstr = unsafe { CStr::from_ptr(out_interface_name) };

            // SAFETY: conversion must never fail.
            let wintun_iface_name = wintun_iface_name_cstr
                .to_str()
                .expect("failed to convert cstring to str")
                .to_owned();

            let wintun_interface = WintunInterface {
                name: wintun_iface_name,
                luid: out_interface_luid,
            };

            // SAFETY: free C string allocated in Go using the correct deallocator.
            unsafe { wgFreePtr(out_interface_name as *mut _) };

            Ok(Self {
                handle,
                wintun_interface,
            })
        } else {
            Err(Error::StartTunnel(handle))
        }
    }

    /// Stop the tunnel.
    pub fn stop(mut self) {
        tracing::info!("Stopping the wg tunnel");
        self.stop_inner();
    }

    #[cfg(windows)]
    pub fn wintun_interface(&self) -> &WintunInterface {
        &self.wintun_interface
    }

    /// Re-attach itself to the tun interface.
    ///
    /// Typically used on default route change.
    #[cfg(target_os = "ios")]
    pub fn bump_sockets(&mut self) {
        unsafe { wgBumpSockets(self.handle) }
    }

    /// Re-bind tunnel socket to the new network interface.
    /// Pass 0 for the interface to bind to blackhole.
    #[cfg(windows)]
    pub fn rebind_tunnel_socket(&mut self, address_family: AddressFamily, interface_index: u32) {
        unsafe { wgRebindTunnelSocket(address_family.to_af_family(), interface_index) }
    }

    /// Update the endpoints of peers matched by public key.
    pub fn update_peers(&mut self, peer_updates: &[PeerEndpointUpdate]) -> Result<()> {
        let mut config_builder = UapiConfigBuilder::new();
        for peer_update in peer_updates {
            peer_update.append_to(&mut config_builder);
        }
        let settings =
            CString::new(config_builder.into_bytes()).map_err(|_| Error::ConfigContainsNulByte)?;
        let ret_code = unsafe { wgSetConfig(self.handle, settings.as_ptr()) };

        if ret_code == 0 {
            Ok(())
        } else {
            Err(Error::SetUapiConfig(i64::from(ret_code)))
        }
    }

    fn stop_inner(&mut self) {
        if self.handle >= 0 {
            unsafe { wgTurnOff(self.handle) };
            self.handle = -1;
        }
    }
}

impl Drop for Tunnel {
    fn drop(&mut self) {
        self.stop_inner()
    }
}

extern "C" {
    /// Start the tunnel.
    #[cfg(not(windows))]
    fn wgTurnOn(
        #[cfg(any(target_os = "linux", target_os = "macos"))] mtu: i32,
        settings: *const c_char,
        fd: RawFd,
        logging_callback: LoggingCallback,
        logging_context: *mut c_void,
    ) -> i32;

    /// Start the tunnel.
    #[cfg(windows)]
    fn wgTurnOn(
        interface_name: *const c_char,
        requested_guid: *const c_char,
        wintun_tunnel_type: *const c_char,
        mtu: i32,
        settings: *const c_char,
        iface_name: *mut *mut c_char,
        iface_luid: *mut u64,
        logging_callback: LoggingCallback,
        logging_context: *mut c_void,
    ) -> i32;

    /// Pass a handle that was created by wgTurnOn to stop the wireguard tunnel.
    fn wgTurnOff(handle: i32);

    /// Returns the config of the WireGuard interface.
    #[allow(unused)]
    fn wgGetConfig(handle: i32) -> *mut c_char;

    /// Sets the config of the WireGuard interface.
    fn wgSetConfig(handle: i32, settings: *const c_char) -> i32;

    /// Frees a pointer allocated by the go runtime - useful to free return value of wgGetConfig
    #[allow(unused)]
    fn wgFreePtr(ptr: *mut c_void);

    /// Re-attach wireguard-go to the tunnel interface.
    #[cfg(target_os = "ios")]
    fn wgBumpSockets(handle: i32);

    /// Re-bind tunnel socket to the new interface.
    ///
    /// - `family` - address family
    /// - `interface_index` - index of network interface to which the tunnel socket should be bound to. Pass 0 to bind to blackhole.
    #[cfg(windows)]
    fn wgRebindTunnelSocket(address_family: u16, interface_index: u32);
}

/// Callback used by libwg to pass wireguard-go logs.
///
/// # Safety
/// Do not call this method directly.
#[doc(hidden)]
pub unsafe extern "system" fn wg_logger_callback(
    _log_level: u32,
    msg: *const c_char,
    _ctx: *mut c_void,
) {
    if !msg.is_null() {
        let str = CStr::from_ptr(msg).to_string_lossy();
        let trimmed_str = str.trim_end();
        tracing::debug!("{}", trimmed_str);
    }
}
