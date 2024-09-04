// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: Apache-2.0

#[cfg(target_os = "android")]
use std::os::fd::RawFd;
use std::{
    ffi::{c_char, c_void, CString},
    fmt,
    net::{IpAddr, SocketAddr},
};

use super::{
    uapi::UapiConfigBuilder, Error, LoggingCallback, PeerConfig, PeerEndpointUpdate, PrivateKey,
    Result,
};

/// Netstack interface configuration.
pub struct InterfaceConfig {
    pub private_key: PrivateKey,
    pub local_addrs: Vec<IpAddr>,
    pub dns_addrs: Vec<IpAddr>,
    pub mtu: u16,
}

impl fmt::Debug for InterfaceConfig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("InterfaceConfig")
            .field("private_key", &"(hidden)")
            .field("local_addrs", &self.local_addrs)
            .field("dns_addrs", &self.dns_addrs)
            .field("mtu", &self.mtu)
            .finish()
    }
}

/// Netstack configuration.
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

        if !self.peers.is_empty() {
            config_builder.add("replace_peers", "true");
            for peer in self.peers.iter() {
                peer.append_to(&mut config_builder);
            }
        }

        config_builder.into_bytes()
    }
}

/// Netstack/WireGuard tunnel
#[derive(Debug)]
pub struct Tunnel {
    handle: i32,
    boxed_logger_ptr: *mut Box<dyn Fn(&str)>,
}

// *mut Box is safe to send
unsafe impl Send for Tunnel {}

impl Tunnel {
    pub fn start<F>(config: Config, logger: F) -> Result<Self>
    where
        F: Fn(&str) + 'static,
    {
        let local_addrs = CString::new(to_comma_separated_addrs(&config.interface.local_addrs))
            .map_err(|_| Error::IpAddrToCstr)?;
        let dns_addrs = CString::new(to_comma_separated_addrs(&config.interface.dns_addrs))
            .map_err(|_| Error::IpAddrToCstr)?;
        let settings =
            CString::new(config.as_uapi_config()).map_err(|_| Error::ConfigContainsNulByte)?;

        let boxed_logger_ptr = unsafe { super::logging::create_logger_callback(logger) };
        let handle = unsafe {
            wgNetTurnOn(
                local_addrs.as_ptr(),
                dns_addrs.as_ptr(),
                i32::from(config.interface.mtu),
                settings.as_ptr(),
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

    /// Update the endpoints of peers matched by public key.
    pub fn update_peers(&mut self, peer_updates: &[PeerEndpointUpdate]) -> Result<()> {
        let mut config_builder = UapiConfigBuilder::new();
        for peer_update in peer_updates {
            peer_update.append_to(&mut config_builder);
        }
        let settings =
            CString::new(config_builder.into_bytes()).map_err(|_| Error::ConfigContainsNulByte)?;
        let ret_code = unsafe { wgNetSetConfig(self.handle, settings.as_ptr()) };

        if ret_code == 0 {
            Ok(())
        } else {
            Err(Error::SetUapiConfig(ret_code))
        }
    }

    /// Get socket descriptor for IPv4 tunnel connection.
    #[cfg(target_os = "android")]
    pub fn get_socket_v4(&self) -> Result<RawFd> {
        let fd = unsafe { wgNetGetSocketV4(self.handle) };
        if fd >= 0 {
            Ok(fd)
        } else {
            Err(Error::ObtainSocketFd)
        }
    }

    /// Get socket descriptor for IPv6 tunnel connection.
    #[cfg(target_os = "android")]
    pub fn get_socket_v6(&self) -> Result<RawFd> {
        let fd = unsafe { wgNetGetSocketV6(self.handle) };
        if fd >= 0 {
            Ok(fd)
        } else {
            Err(Error::ObtainSocketFd)
        }
    }

    /// Stop the tunnel.
    ///
    /// All connections over the tunnel will be terminated.
    pub fn stop(mut self) {
        self.stop_inner();
    }

    /// Open UDP connection through the tunnel.
    ///
    /// Due to FFI boundary, direct communication is impossible. Instead a bidrectional UDP forwarder listens on
    /// `listen_port`. The clients should connect to it in order to communicate with the exit endpoint over
    /// the tunnel.
    ///
    /// Note that the client traffic should originate from the `client_port` on the loopback interface.
    /// If `exit_endpoint` belongs to IPv6 address family, then the `listen_port` is opened on `::1`, otherwise `127.0.0.1`.
    pub fn open_connection(
        &mut self,
        listen_port: u16,
        client_port: u16,
        exit_endpoint: SocketAddr,
    ) -> Result<TunnelConnection> {
        TunnelConnection::open(self, listen_port, client_port, exit_endpoint)
    }

    fn stop_inner(&mut self) {
        if self.handle >= 0 {
            unsafe { wgNetTurnOff(self.handle) };
            self.handle = -1;
        }
        if !self.boxed_logger_ptr.is_null() {
            // causes crash on ios and android
            // unsafe {
            //     let _ = Box::from_raw(self.boxed_logger_ptr);
            // }
            self.boxed_logger_ptr = std::ptr::null_mut();
        }
    }
}

impl Drop for Tunnel {
    fn drop(&mut self) {
        self.stop_inner()
    }
}

/// UDP connection through the netstack tunnel.
#[derive(Debug)]
pub struct TunnelConnection {
    handle: i32,
}

impl TunnelConnection {
    fn open(
        entry_tunnel: &Tunnel,
        listen_port: u16,
        client_port: u16,
        exit_endpoint: SocketAddr,
    ) -> Result<Self> {
        let exit_endpoint = CString::new(exit_endpoint.to_string())
            .expect("failed to convert exit endpoint to CString");
        let handle = unsafe {
            wgNetOpenConnectionThroughTunnel(
                entry_tunnel.handle,
                listen_port,
                client_port,
                exit_endpoint.as_ptr(),
            )
        };

        if handle >= 0 {
            Ok(Self { handle })
        } else {
            Err(Error::OpenConnection(handle))
        }
    }

    pub fn close(mut self) {
        self.close_inner()
    }

    fn close_inner(&mut self) {
        if self.handle >= 0 {
            unsafe { wgNetCloseConnectionThroughTunnel(self.handle) };
            self.handle = -1;
        }
    }
}

impl Drop for TunnelConnection {
    fn drop(&mut self) {
        self.close_inner();
    }
}

fn to_comma_separated_addrs(ip_addrs: &[IpAddr]) -> String {
    ip_addrs
        .iter()
        .map(|x| x.to_string())
        .collect::<Vec<_>>()
        .join(",")
}

extern "C" {
    fn wgNetTurnOn(
        local_addresses: *const c_char,
        dns_addresses: *const c_char,
        mtu: i32,
        settings: *const c_char,
        logging_callback: Option<LoggingCallback>,
        logging_context: *mut c_void,
    ) -> i32;
    fn wgNetTurnOff(net_tunnel_handle: i32);
    fn wgNetSetConfig(net_tunnel_handle: i32, settings: *const c_char) -> i64;
    #[allow(unused)]
    fn wgNetGetConfig(net_tunnel_handle: i32) -> *const c_char;
    fn wgNetOpenConnectionThroughTunnel(
        entry_tunnel_handle: i32,
        listen_port: u16,
        client_port: u16,
        exit_endpoint: *const c_char,
    ) -> i32;
    fn wgNetCloseConnectionThroughTunnel(handle: i32);
    #[cfg(target_os = "android")]
    fn wgNetGetSocketV4(net_tunnel_handle: i32) -> i32;
    #[cfg(target_os = "android")]
    fn wgNetGetSocketV6(net_tunnel_handle: i32) -> i32;
}
