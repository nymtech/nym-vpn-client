// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(target_os = "android")]
use std::os::fd::RawFd;
use std::{
    ffi::{c_char, c_void, CStr, CString},
    fmt,
    net::{IpAddr, SocketAddr},
};

#[cfg(windows)]
use nym_windows::net::AddressFamily;

use super::{
    uapi::UapiConfigBuilder, Error, LoggingCallback, PeerConfig, PeerEndpointUpdate, PrivateKey,
    Result,
};
#[cfg(feature = "amnezia")]
use crate::amnezia::AmneziaConfig;

/// Netstack interface configuration.
pub struct InterfaceConfig {
    pub private_key: PrivateKey,
    pub local_addrs: Vec<IpAddr>,
    pub dns_addrs: Vec<IpAddr>,
    pub mtu: u16,
    /// Mark used for mark-based routing.
    #[cfg(target_os = "linux")]
    pub fwmark: Option<u32>,
    #[cfg(feature = "amnezia")]
    pub azwg_config: Option<AmneziaConfig>,
}

impl fmt::Debug for InterfaceConfig {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut d = f.debug_struct("InterfaceConfig");
        d.field("private_key", &"(hidden)")
            .field("local_addrs", &self.local_addrs)
            .field("dns_addrs", &self.dns_addrs)
            .field("mtu", &self.mtu);
        #[cfg(target_os = "linux")]
        d.field("fwmark", &self.fwmark);
        #[cfg(feature = "amnezia")]
        d.field("azwg_config", &self.azwg_config);
        d.finish()
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

/// Netstack/WireGuard tunnel
#[derive(Debug)]
pub struct Tunnel {
    handle: i32,
}

impl Tunnel {
    pub fn start(config: Config) -> Result<Self> {
        let local_addrs = CString::new(to_comma_separated_addrs(&config.interface.local_addrs))
            .map_err(|_| Error::IpAddrToCstr)?;
        let dns_addrs = CString::new(to_comma_separated_addrs(&config.interface.dns_addrs))
            .map_err(|_| Error::IpAddrToCstr)?;
        let settings =
            CString::new(config.as_uapi_config()).map_err(|_| Error::ConfigContainsNulByte)?;

        let handle = unsafe {
            wgNetTurnOn(
                local_addrs.as_ptr(),
                dns_addrs.as_ptr(),
                i32::from(config.interface.mtu),
                settings.as_ptr(),
                wg_netstack_logger_callback,
                std::ptr::null_mut(),
            )
        };

        if handle >= 0 {
            Ok(Self { handle })
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
    }

    /// Re-attach itself to the new primary interface.
    ///
    /// Typically used on default route change.
    #[cfg(target_os = "ios")]
    pub fn bump_sockets(&mut self) {
        unsafe { wgNetBumpSockets(self.handle) }
    }

    /// Re-bind tunnel socket to the new network interface.
    /// Pass 0 for the interface to bind to blackhole.
    #[cfg(windows)]
    pub fn rebind_tunnel_socket(&mut self, address_family: AddressFamily, interface_index: u32) {
        unsafe { wgNetRebindTunnelSocket(address_family.to_af_family(), interface_index) }
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
        let exit_endpoint =
            CString::new(exit_endpoint.to_string()).map_err(|_| Error::SocketAddrToCstr)?;
        let handle = unsafe {
            wgNetOpenConnectionThroughTunnel(
                entry_tunnel.handle,
                listen_port,
                client_port,
                exit_endpoint.as_ptr(),
                wg_netstack_logger_callback,
                std::ptr::null_mut(),
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
    /// Start the netstack tunnel.
    fn wgNetTurnOn(
        local_addresses: *const c_char,
        dns_addresses: *const c_char,
        mtu: i32,
        settings: *const c_char,
        logging_callback: LoggingCallback,
        logging_context: *mut c_void,
    ) -> i32;

    /// Pass a handle that was created by wgNetTurnOn to stop the wireguard tunnel.
    fn wgNetTurnOff(net_tunnel_handle: i32);

    /// Sets the config of the WireGuard interface.
    fn wgNetSetConfig(net_tunnel_handle: i32, settings: *const c_char) -> i64;

    /// Returns the config of the WireGuard interface.
    #[allow(unused)]
    fn wgNetGetConfig(net_tunnel_handle: i32) -> *const c_char;

    /// Open connection through the tunnel.
    fn wgNetOpenConnectionThroughTunnel(
        entry_tunnel_handle: i32,
        listen_port: u16,
        client_port: u16,
        exit_endpoint: *const c_char,
        logging_callback: LoggingCallback,
        logging_context: *mut c_void,
    ) -> i32;

    /// Close connection through the tunnel.
    fn wgNetCloseConnectionThroughTunnel(handle: i32);

    /// Returns tunnel IPv4 socket.
    #[cfg(target_os = "android")]
    fn wgNetGetSocketV4(net_tunnel_handle: i32) -> i32;

    /// Returns tunnel IPv6 socket.
    #[cfg(target_os = "android")]
    fn wgNetGetSocketV6(net_tunnel_handle: i32) -> i32;

    /// Re-attach wireguard-go to the tunnel interface.
    #[cfg(target_os = "ios")]
    fn wgNetBumpSockets(handle: i32);

    /// Re-bind tunnel socket to the new interface.
    ///
    /// - `family` - address family
    /// - `interface_index` - index of network interface to which the tunnel socket should be bound to. Pass 0 to bind to blackhole.
    #[cfg(windows)]
    fn wgNetRebindTunnelSocket(address_family: u16, interface_index: u32);
}

/// Callback used by libwg to pass netstack logs.
///
/// # Safety
/// Do not call this method directly.
#[doc(hidden)]
pub unsafe extern "system" fn wg_netstack_logger_callback(
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
