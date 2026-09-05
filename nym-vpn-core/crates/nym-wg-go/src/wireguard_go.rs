// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(unix)]
use std::os::fd::{IntoRawFd, OwnedFd, RawFd};
use std::{
    ffi::{CStr, CString, c_char, c_void},
    fmt,
    mem::replace,
    net::SocketAddr,
    sync::{Arc, PoisonError, RwLock},
    time::{Duration, SystemTime},
};

use nym_crypto::asymmetric::x25519;
#[cfg(windows)]
use nym_windows::net::AddressFamily;
use typed_builder::TypedBuilder;
#[cfg(windows)]
use windows::Win32::NetworkManagement::Ndis::NET_LUID_LH;

use super::{
    Error, LoggingCallback, PeerConfig, PeerEndpointUpdate, Result, uapi::UapiConfigBuilder,
};
#[cfg(feature = "amnezia")]
use crate::amnezia::AmneziaConfig;

/// Classic WireGuard interface configuration.
pub struct InterfaceConfig {
    pub listen_port: Option<u16>,
    pub keypair: Arc<x25519::KeyPair>,
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
    fn into_uapi_config(self) -> Vec<u8> {
        let mut config_builder = UapiConfigBuilder::new();
        config_builder.add(
            "private_key",
            self.interface.keypair.private_key().to_bytes().as_ref(),
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
            for peer in self.peers.into_iter() {
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

/// Tunnel file descriptor
#[cfg(not(windows))]
#[derive(Debug)]
pub enum TunnelFd {
    /// Tunnel device
    Tun(OwnedFd),

    /// Proxy device using unix domain socket
    #[cfg(target_os = "android")]
    Proxy(OwnedFd),
}

#[derive(Debug, TypedBuilder)]
pub struct TunnelConfig {
    /// Tunnel file descriptor
    #[cfg(not(windows))]
    tun_fd: TunnelFd,

    /// Interface name
    #[cfg(windows)]
    interface_name: String,

    /// Interface GUID
    #[cfg(windows)]
    requested_guid: String,

    /// Tunnel type identifier
    /// Passed during WinTun initialization
    #[cfg(windows)]
    wintun_tunnel_type: String,
}

/// Classic WireGuard tunnel.
#[derive(Debug)]
pub struct Tunnel {
    tunnel_handle: Arc<RwLock<i32>>, // Handle is shared between Tunnel and TunnelStatsReader
    #[cfg(windows)]
    wintun_interface: WintunInterface,
}

impl Tunnel {
    /// Start new WireGuard tunnel
    pub fn start(wg_config: Config, tun_config: TunnelConfig) -> Result<Self> {
        #[cfg(not(windows))]
        {
            match tun_config.tun_fd {
                TunnelFd::Tun(tun_fd) => Self::start_with_tun(wg_config, tun_fd),
                #[cfg(target_os = "android")]
                TunnelFd::Proxy(proxy_fd) => Self::start_with_proxy_fd(wg_config, proxy_fd),
            }
        }

        #[cfg(windows)]
        {
            Self::start_inner(
                wg_config,
                &tun_config.interface_name,
                &tun_config.requested_guid,
                &tun_config.wintun_tunnel_type,
            )
        }
    }

    /// Start new WireGuard tunnel with TUN fd.
    #[cfg(not(windows))]
    fn start_with_tun(config: Config, tun_fd: OwnedFd) -> Result<Self> {
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        let interface_mtu = config.interface.mtu;
        let settings = CString::new(config.into_uapi_config())
            .map_err(|_| Error::ConvertToCString("uapi config"))?;

        let tunnel_handle = unsafe {
            wgTurnOn(
                // note: not all platforms accept mtu = 0
                #[cfg(any(target_os = "linux", target_os = "macos"))]
                i32::from(interface_mtu),
                settings.as_ptr(),
                tun_fd.into_raw_fd(),
                wg_logger_callback,
                std::ptr::null_mut(),
            )
        };

        if tunnel_handle >= 0 {
            Ok(Self {
                tunnel_handle: Arc::new(RwLock::new(tunnel_handle)),
            })
        } else {
            Err(Error::StartTunnel(tunnel_handle))
        }
    }

    /// Start new WireGuard tunnel with wintun interface.
    #[cfg(windows)]
    fn start_inner(
        config: Config,
        interface_name: &str,
        requested_guid: &str,
        wintun_tunnel_type: &str,
    ) -> Result<Self> {
        let interface_mtu = config.interface.mtu;
        let settings = CString::new(config.into_uapi_config())
            .map_err(|_| Error::ConvertToCString("settings"))?;
        let interface_name_cstr =
            CString::new(interface_name).map_err(|_| Error::ConvertToCString("interface name"))?;
        let requested_guid_cstr =
            CString::new(requested_guid).map_err(|_| Error::ConvertToCString("requested guid"))?;
        let wintun_tunnel_type_cstr =
            CString::new(wintun_tunnel_type).map_err(|_| Error::ConvertToCString("tunnel type"))?;

        let mut out_interface_name: *mut c_char = std::ptr::null_mut();
        let out_interface_name_ptr: *mut *mut c_char = &mut out_interface_name;

        let mut out_interface_luid: u64 = 0;
        let out_interface_luid_ptr: *mut u64 = &mut out_interface_luid;

        let tunnel_handle = unsafe {
            wgTurnOn(
                interface_name_cstr.as_ptr(),
                requested_guid_cstr.as_ptr(),
                wintun_tunnel_type_cstr.as_ptr(),
                i32::from(interface_mtu),
                settings.as_ptr(),
                out_interface_name_ptr,
                out_interface_luid_ptr,
                wg_logger_callback,
                std::ptr::null_mut(),
            )
        };

        if tunnel_handle >= 0 {
            // SAFETY: libwg is expected to set a non-null value upon successful return.
            let wintun_iface_name_cstr = unsafe { CStr::from_ptr(out_interface_name) };

            // SAFETY: conversion must never fail.
            let wintun_iface_name = wintun_iface_name_cstr
                .to_str()
                .map_err(|_| Error::ConvertToString("wintun interface name"))
                .map(|s| s.to_owned());

            // SAFETY: free C string allocated in Go using the correct deallocator.
            unsafe { wgFreePtr(out_interface_name as *mut _) };

            let wintun_interface = WintunInterface {
                name: wintun_iface_name?,
                luid: out_interface_luid,
            };

            Ok(Self {
                tunnel_handle: Arc::new(RwLock::new(tunnel_handle)),
                wintun_interface,
            })
        } else {
            Err(Error::StartTunnel(tunnel_handle))
        }
    }

    /// Start a new WireGuard tunnel backed by a raw proxy socket fd (Android only).
    ///
    /// Unlike [`start`], this calls `wgTurnOnWithProxyFd` which wraps the fd as a "dumb" I/O
    /// device without calling `TUNGETIFF`, making it safe to use with a Unix socket fd.
    #[cfg(target_os = "android")]
    fn start_with_proxy_fd(config: Config, proxy_fd: OwnedFd) -> Result<Self> {
        let mtu = config.interface.mtu;
        let settings = CString::new(config.into_uapi_config())
            .map_err(|_| Error::ConvertToCString("uapi config"))?;

        let tunnel_handle = unsafe {
            wgTurnOnWithProxyFd(
                settings.as_ptr(),
                proxy_fd.into_raw_fd(),
                i32::from(mtu),
                wg_logger_callback,
                std::ptr::null_mut(),
            )
        };

        if tunnel_handle >= 0 {
            Ok(Self {
                tunnel_handle: Arc::new(RwLock::new(tunnel_handle)),
            })
        } else {
            Err(Error::StartTunnel(tunnel_handle))
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
        let handle = self
            .tunnel_handle
            .read()
            .unwrap_or_else(PoisonError::into_inner);
        if *handle >= 0 {
            unsafe { wgBumpSockets(*handle) }
        }
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
        let settings = CString::new(config_builder.into_bytes())
            .map_err(|_| Error::ConvertToCString("peer update config"))?;
        let handle = self
            .tunnel_handle
            .read()
            .unwrap_or_else(PoisonError::into_inner);
        if *handle < 0 {
            return Err(Error::TunnelStopped);
        }
        let ret_code = unsafe { wgSetConfig(*handle, settings.as_ptr()) };

        if ret_code == 0 {
            Ok(())
        } else {
            Err(Error::SetUapiConfig(i64::from(ret_code)))
        }
    }

    /// Create a stats reader that can query live peer stats without owning the tunnel.
    pub fn stats_reader(&self) -> TunnelStatsReader {
        TunnelStatsReader {
            tunnel_handle: self.tunnel_handle.clone(),
        }
    }

    fn stop_inner(&mut self) {
        let handle = replace(
            &mut *self
                .tunnel_handle
                .write()
                .unwrap_or_else(PoisonError::into_inner),
            -1,
        );
        if handle >= 0 {
            unsafe { wgTurnOff(handle) };
        }
    }
}

impl Drop for Tunnel {
    fn drop(&mut self) {
        self.stop_inner()
    }
}

#[derive(Debug, Clone)]
pub struct PeerStats {
    pub public_key: [u8; 32],
    pub endpoint: Option<SocketAddr>,
    pub last_handshake_time: Option<SystemTime>,
    pub rx_bytes: u64,
    pub tx_bytes: u64,
}

impl PeerStats {
    pub fn has_completed_handshake(&self) -> bool {
        self.last_handshake_time.is_some()
    }
}

#[derive(Debug, Clone)]
pub struct TunnelStats {
    pub listen_port: Option<u16>,
    pub peers: Vec<PeerStats>,
}

impl TunnelStats {
    pub fn all_peers_connected(&self) -> bool {
        !self.peers.is_empty() && self.peers.iter().all(|p| p.has_completed_handshake())
    }

    /// Parse the textual UAPI `get=1` response of a WireGuard device.
    pub fn parse(response: &str) -> Self {
        TunnelStatsReader::parse_tunnel_stats(response)
    }
}

pub struct TunnelStatsReader {
    tunnel_handle: Arc<RwLock<i32>>, // Handle is shared between Tunnel and TunnelStatsReader
}

impl TunnelStatsReader {
    pub fn get_stats(&self) -> Result<TunnelStats> {
        let handle = self
            .tunnel_handle
            .read()
            .unwrap_or_else(PoisonError::into_inner);
        if *handle < 0 {
            return Err(Error::TunnelStopped);
        }
        let raw = unsafe { wgGetConfig(*handle) };
        if raw.is_null() {
            return Err(Error::GetUapiConfig);
        }
        let s = unsafe { CStr::from_ptr(raw).to_string_lossy().into_owned() };
        unsafe { wgFreePtr(raw as *mut c_void) };
        tracing::trace!("TunnelStats: '{s}'");
        Ok(Self::parse_tunnel_stats(&s))
    }

    fn parse_tunnel_stats(response: &str) -> TunnelStats {
        let mut listen_port: Option<u16> = None;
        let mut peers: Vec<PeerStats> = Vec::new();
        let mut current: Option<PeerBuilder> = None;

        for line in response.lines() {
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            match key {
                "listen_port" => listen_port = value.parse().ok(),
                "public_key" => {
                    if let Some(builder) = current.take() {
                        peers.push(builder.build());
                    }
                    current = hex::decode(value)
                        .ok()
                        .and_then(|b| b.try_into().ok())
                        .map(PeerBuilder::new);
                }
                "endpoint" => {
                    if let Some(ref mut b) = current {
                        b.endpoint = value.parse().ok();
                    }
                }
                "last_handshake_time_sec" => {
                    if let Some(ref mut b) = current {
                        b.handshake_sec = value.parse().unwrap_or(0);
                    }
                }
                "last_handshake_time_nsec" => {
                    if let Some(ref mut b) = current {
                        b.handshake_nsec = value.parse().unwrap_or(0);
                    }
                }
                "rx_bytes" => {
                    if let Some(ref mut b) = current {
                        b.rx_bytes = value.parse().unwrap_or(0);
                    }
                }
                "tx_bytes" => {
                    if let Some(ref mut b) = current {
                        b.tx_bytes = value.parse().unwrap_or(0);
                    }
                }
                _ => {}
            }
        }
        if let Some(builder) = current {
            peers.push(builder.build());
        }

        TunnelStats { listen_port, peers }
    }
}

struct PeerBuilder {
    public_key: [u8; 32],
    endpoint: Option<SocketAddr>,
    handshake_sec: u64,
    handshake_nsec: u32,
    rx_bytes: u64,
    tx_bytes: u64,
}

impl PeerBuilder {
    fn new(public_key: [u8; 32]) -> Self {
        Self {
            public_key,
            endpoint: None,
            handshake_sec: 0,
            handshake_nsec: 0,
            rx_bytes: 0,
            tx_bytes: 0,
        }
    }

    fn build(self) -> PeerStats {
        let last_handshake_time = if self.handshake_sec > 0 {
            SystemTime::UNIX_EPOCH
                .checked_add(Duration::new(self.handshake_sec, self.handshake_nsec))
        } else {
            None
        };
        PeerStats {
            public_key: self.public_key,
            endpoint: self.endpoint,
            last_handshake_time,
            rx_bytes: self.rx_bytes,
            tx_bytes: self.tx_bytes,
        }
    }
}

unsafe extern "C" {
    /// Start a WireGuard tunnel using a raw proxy socket fd (Android only).
    /// Wraps the fd without calling TUNGETIFF so it works with Unix socket fds.
    #[cfg(target_os = "android")]
    unsafe fn wgTurnOnWithProxyFd(
        settings: *const c_char,
        fd: RawFd,
        mtu: i32,
        logging_callback: LoggingCallback,
        logging_context: *mut c_void,
    ) -> i32;

    /// Start the tunnel.
    #[cfg(not(windows))]
    unsafe fn wgTurnOn(
        #[cfg(any(target_os = "linux", target_os = "macos"))] mtu: i32,
        settings: *const c_char,
        fd: RawFd,
        logging_callback: LoggingCallback,
        logging_context: *mut c_void,
    ) -> i32;

    /// Start the tunnel.
    #[cfg(windows)]
    unsafe fn wgTurnOn(
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
    unsafe fn wgTurnOff(handle: i32);

    /// Returns the config of the WireGuard interface.
    #[allow(unused)]
    unsafe fn wgGetConfig(handle: i32) -> *mut c_char;

    /// Sets the config of the WireGuard interface.
    unsafe fn wgSetConfig(handle: i32, settings: *const c_char) -> i32;

    /// Frees a pointer allocated by the go runtime - useful to free return value of wgGetConfig
    #[allow(unused)]
    unsafe fn wgFreePtr(ptr: *mut c_void);

    /// Re-attach wireguard-go to the tunnel interface.
    #[cfg(target_os = "ios")]
    unsafe fn wgBumpSockets(handle: i32);

    /// Re-bind tunnel socket to the new interface.
    ///
    /// - `family` - address family
    /// - `interface_index` - index of network interface to which the tunnel socket should be bound to. Pass 0 to bind to blackhole.
    #[cfg(windows)]
    unsafe fn wgRebindTunnelSocket(address_family: u16, interface_index: u32);
}

/// Callback used by libwg to pass wireguard-go logs.
///
/// # Safety
/// Do not call this method directly.
#[doc(hidden)]
pub unsafe extern "system" fn wg_logger_callback(
    log_level: u32,
    msg: *const c_char,
    _ctx: *mut c_void,
) {
    if let Some(wg_log_level) = WgLogLevel::from_u32(log_level)
        && !msg.is_null()
    {
        let str = unsafe { CStr::from_ptr(msg).to_string_lossy() };
        let trimmed_str = str.trim_end();

        match wg_log_level {
            WgLogLevel::Error => tracing::error!("{trimmed_str}"),
            WgLogLevel::Verbose => tracing::trace!("{trimmed_str}"),
        }
    }
}

#[repr(u32)]
enum WgLogLevel {
    Error = 1,
    Verbose = 2,
}

impl WgLogLevel {
    fn from_u32(value: u32) -> Option<Self> {
        match value {
            x if x == WgLogLevel::Error as u32 => Some(WgLogLevel::Error),
            x if x == WgLogLevel::Verbose as u32 => Some(WgLogLevel::Verbose),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const UAPI_TWO_PEERS: &str = "private_key=00\n\
listen_port=51820\n\
public_key=1111111111111111111111111111111111111111111111111111111111111111\n\
endpoint=1.2.3.4:51820\n\
last_handshake_time_sec=1700000000\n\
last_handshake_time_nsec=5\n\
rx_bytes=10\n\
tx_bytes=20\n\
public_key=2222222222222222222222222222222222222222222222222222222222222222\n\
endpoint=5.6.7.8:51820\n\
last_handshake_time_sec=0\n\
last_handshake_time_nsec=0\n\
rx_bytes=0\n\
tx_bytes=30\n\
errno=0\n";

    #[test]
    fn parse_reports_per_peer_handshake_state() {
        let stats = TunnelStats::parse(UAPI_TWO_PEERS);

        assert_eq!(stats.listen_port, Some(51820));
        assert_eq!(stats.peers.len(), 2);
        assert!(stats.peers[0].has_completed_handshake());
        assert!(
            !stats.peers[1].has_completed_handshake(),
            "a zero handshake timestamp means the peer never handshaked"
        );
        assert!(!stats.all_peers_connected());
    }
}
