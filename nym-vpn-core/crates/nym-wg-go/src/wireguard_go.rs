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
}

impl std::fmt::Debug for InterfaceConfig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("InterfaceConfig")
            .field("listen_port", &self.listen_port)
            .field("private_key", &"(hidden)")
            .field("mtu", &self.mtu)
            .finish()
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
    boxed_logger_ptr: *mut Box<dyn Fn(&str)>,
}

// *mut Box is safe to send
unsafe impl Send for Tunnel {}

impl Tunnel {
    /// Start new WireGuard tunnel
    pub fn start<F>(config: Config, tun_fd: RawFd, logger: F) -> Result<Self>
    where
        F: Fn(&str) + 'static + Send,
    {
        let settings =
            CString::new(config.as_uapi_config()).map_err(|_| Error::ConfigContainsNulByte)?;

        let boxed_logger_ptr = unsafe { super::logging::create_logger_callback(logger) };

        // todo: pass logger
        let handle = unsafe {
            wgTurnOn(
                settings.as_ptr(),
                tun_fd,
                Some(super::logging::wg_logger_callback),
                boxed_logger_ptr as *mut _,
            )
        };

        if handle >= 0 {
            Ok(Self {
                handle,
                boxed_logger_ptr,
            })
        } else {
            Err(Error::StartTunnel(handle))
        }
    }

    /// Stop the tunnel.
    pub fn stop(mut self) {
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

    #[cfg(any(target_os = "ios", target_os = "android"))]
    pub fn disable_roaming(&mut self) {
        unsafe { wgDisableSomeRoamingForBrokenMobileSemantics(self.handle) }
    }

    fn stop_inner(&mut self) {
        if self.handle >= 0 {
            unsafe { wgTurnOff(self.handle) };
            self.handle = -1;
        }

        if self.boxed_logger_ptr.is_null() {
            unsafe {
                let _ = Box::from_raw(self.boxed_logger_ptr);
            }
            self.boxed_logger_ptr = std::ptr::null_mut();
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
    #[cfg(any(target_os = "android", target_os = "ios"))]
    fn wgTurnOn(
        settings: *const c_char,
        fd: RawFd,
        logging_callback: Option<LoggingCallback>,
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

    // Disable roaming.
    #[cfg(any(target_os = "ios", target_os = "android"))]
    fn wgDisableSomeRoamingForBrokenMobileSemantics(handle: i32);
}
