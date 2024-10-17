// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: Apache-2.0

#[cfg(unix)]
use std::os::unix::io::RawFd;
use std::{
    ffi::{c_char, c_void, CString},
    fmt,
};

use super::{
    uapi::UapiConfigBuilder, Error, LoggingCallback, PeerConfig, PeerEndpointUpdate, PrivateKey,
    Result,
};

/// Classic WireGuard interface configuration.
pub struct InterfaceConfig {
    pub listen_port: Option<u16>,
    pub private_key: PrivateKey,
    pub mtu: u16,
    #[cfg(target_os = "linux")]
    pub fwmark: Option<u32>,
    #[cfg(feature = "amnezia")]
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
        d.finish()
    }
}
/// Hold Amnezia-wireguard configuration parameters.
///
/// All parameters should be the same between Client and Server, except Jc - it can vary.
///
/// - Jc — 1 ≤ Jc ≤ 128; recommended range is from 3 to 10 inclusive
/// - Jmin — Jmin < Jmax; recommended value is 50
/// - Jmax — Jmin < Jmax ≤ 1280; recommended value is 1000
/// - S1 — S1 < 1280; S1 + 56 ≠ S2; recommended range is from 15 to 150 inclusive
/// - S2 — S2 < 1280; recommended range is from 15 to 150 inclusive
/// - H1/H2/H3/H4 — must be unique among each other;
///     recommended range is from 5 to 2_147_483_647  (2^31 - 1   i.e. signed 32 bit int) inclusive
#[cfg(feature = "amnezia")]
#[derive(Debug)]
pub struct AmneziaConfig {
    pub junk_packet_count: i32,              // Jc
    pub junk_packet_min_size: i32,           // Jmin
    pub junk_packet_max_size: i32,           // Jmax
    pub init_packet_junk_size: i32,          // S0
    pub response_packet_junk_size: i32,      // S1
    pub init_packet_magic_header: u32,       // H1
    pub response_packet_magic_header: u32,   // H2
    pub under_load_packet_magic_header: u32, // H3
    pub transport_packet_magic_header: u32,  // H4
}

#[cfg(feature = "amnezia")]
impl Default for AmneziaConfig {
    fn default() -> Self {
        Self {
            junk_packet_count: 4_i32,
            junk_packet_min_size: 40_i32,
            junk_packet_max_size: 70_i32,
            init_packet_junk_size: 0_i32,
            response_packet_junk_size: 0_i32,
            init_packet_magic_header: 1_u32,
            response_packet_magic_header: 2_u32,
            under_load_packet_magic_header: 3_u32,
            transport_packet_magic_header: 4_u32,
        }
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

        if !self.peers.is_empty() {
            config_builder.add("replace_peers", "true");
            for peer in self.peers.iter() {
                peer.append_to(&mut config_builder);
            }
        }

        config_builder.into_bytes()
    }
}

/// Classic WireGuard tunnel.
#[derive(Debug)]
pub struct Tunnel {
    handle: i32,
}

impl Tunnel {
    /// Start new WireGuard tunnel
    pub fn start(config: Config, tun_fd: RawFd) -> Result<Self> {
        let settings =
            CString::new(config.as_uapi_config()).map_err(|_| Error::ConfigContainsNulByte)?;
        let handle = unsafe {
            wgTurnOn(
                // note: not all platforms accept mtu = 0
                #[cfg(any(target_os = "linux", target_os = "macos"))]
                i32::from(config.interface.mtu),
                settings.as_ptr(),
                tun_fd,
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

    /// Stop the tunnel.
    pub fn stop(mut self) {
        tracing::info!("Stopping the wg tunnel");
        self.stop_inner();
    }

    /// Re-attach itself to the tun interface.
    ///
    /// Typically used on default route change.
    #[cfg(target_os = "ios")]
    pub fn bump_sockets(&mut self) {
        unsafe { wgBumpSockets(self.handle) }
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
    // Start the tunnel.
    fn wgTurnOn(
        #[cfg(any(target_os = "linux", target_os = "macos"))] mtu: i32,
        settings: *const c_char,
        fd: RawFd,
        logging_callback: LoggingCallback,
        logging_context: *mut c_void,
    ) -> i32;

    // Pass a handle that was created by wgTurnOn to stop a wireguard tunnel.
    fn wgTurnOff(handle: i32);

    // Returns the config of the WireGuard interface.
    #[allow(unused)]
    fn wgGetConfig(handle: i32) -> *mut c_char;

    // Sets the config of the WireGuard interface.
    fn wgSetConfig(handle: i32, settings: *const c_char) -> i32;

    // Frees a pointer allocated by the go runtime - useful to free return value of wgGetConfig
    #[allow(unused)]
    fn wgFreePtr(ptr: *mut c_void);

    // Re-attach wireguard-go to the tunnel interface.
    #[cfg(target_os = "ios")]
    fn wgBumpSockets(handle: i32);
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
        let str = std::ffi::CStr::from_ptr(msg).to_string_lossy();
        tracing::debug!("{}", str);
    }
}
