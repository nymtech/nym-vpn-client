// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#[cfg(target_os = "android")]
use std::os::fd::RawFd;
use std::{
    ffi::{CStr, CString, c_char, c_void},
    fmt,
    net::{IpAddr, SocketAddr},
    str::FromStr,
    sync::Arc,
};

use nym_crypto::asymmetric::x25519;
#[cfg(windows)]
use nym_windows::net::AddressFamily;

use super::{
    Error, LoggingCallback, PeerConfig, PeerEndpointUpdate, Result, uapi::UapiConfigBuilder,
    wireguard_go::TunnelStats,
};
#[cfg(feature = "amnezia")]
use crate::amnezia::AmneziaConfig;

/// Netstack interface configuration.
pub struct InterfaceConfig {
    pub keypair: Arc<x25519::KeyPair>,
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
    fn into_uapi_config(self) -> Vec<u8> {
        let mut config_builder = UapiConfigBuilder::new();
        config_builder.add(
            "private_key",
            self.interface.keypair.private_key().to_bytes().as_ref(),
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
            for peer in self.peers.into_iter() {
                peer.append_to(&mut config_builder);
            }
        }

        config_builder.into_bytes()
    }
}

/// Netstack/WireGuard tunnel
#[derive(Debug)]
pub struct Tunnel {
    tunnel_handle: i32,
}

/// Reads live peer stats of a netstack WireGuard tunnel via the UAPI GET interface.
///
/// Once the owning [`Tunnel`] is stopped the Go side no longer knows the handle and
/// [`TunnelStatsReader::get_stats`] fails with [`Error::GetUapiConfig`].
#[derive(Debug, Clone)]
pub struct TunnelStatsReader {
    tunnel_handle: i32,
}

impl TunnelStatsReader {
    pub fn get_stats(&self) -> Result<TunnelStats> {
        if self.tunnel_handle < 0 {
            return Err(Error::TunnelStopped);
        }
        let raw = unsafe { wgNetGetConfig(self.tunnel_handle) };
        if raw.is_null() {
            return Err(Error::GetUapiConfig);
        }
        let s = unsafe { CStr::from_ptr(raw).to_string_lossy().into_owned() };
        unsafe { wgFreePtr(raw as *mut c_void) };
        tracing::trace!("Netstack TunnelStats: '{s}'");
        Ok(TunnelStats::parse(&s))
    }
}

impl Tunnel {
    pub fn start(config: Config) -> Result<Self> {
        let interface_mtu = config.interface.mtu;
        let local_addrs = CString::new(to_comma_separated_addrs(&config.interface.local_addrs))
            .map_err(|_| Error::ConvertToCString("interface local addrs"))?;
        let dns_addrs = CString::new(to_comma_separated_addrs(&config.interface.dns_addrs))
            .map_err(|_| Error::ConvertToCString("interface dns addrs"))?;
        let settings = CString::new(config.into_uapi_config())
            .map_err(|_| Error::ConvertToCString("uapi config"))?;

        let tunnel_handle = unsafe {
            wgNetTurnOn(
                local_addrs.as_ptr(),
                dns_addrs.as_ptr(),
                i32::from(interface_mtu),
                settings.as_ptr(),
                wg_netstack_logger_callback,
                std::ptr::null_mut(),
            )
        };

        if tunnel_handle >= 0 {
            Ok(Self { tunnel_handle })
        } else {
            Err(Error::StartTunnel(tunnel_handle))
        }
    }

    /// Create a stats reader that can query live peer stats without owning the tunnel.
    pub fn stats_reader(&self) -> TunnelStatsReader {
        TunnelStatsReader {
            tunnel_handle: self.tunnel_handle,
        }
    }

    /// Update the endpoints of peers matched by public key.
    pub fn update_peers(&mut self, peer_updates: &[PeerEndpointUpdate]) -> Result<()> {
        let mut config_builder = UapiConfigBuilder::new();
        for peer_update in peer_updates {
            peer_update.append_to(&mut config_builder);
        }
        let settings = CString::new(config_builder.into_bytes())
            .map_err(|_| Error::ConvertToCString("peer update config"))?;
        let ret_code = unsafe { wgNetSetConfig(self.tunnel_handle, settings.as_ptr()) };

        if ret_code == 0 {
            Ok(())
        } else {
            Err(Error::SetUapiConfig(ret_code))
        }
    }

    /// Get socket descriptor for IPv4 tunnel connection.
    #[cfg(target_os = "android")]
    pub fn get_socket_v4(&self) -> Result<RawFd> {
        let fd = unsafe { wgNetGetSocketV4(self.tunnel_handle) };
        if fd >= 0 {
            Ok(fd)
        } else {
            Err(Error::ObtainSocketFd)
        }
    }

    /// Get socket descriptor for IPv6 tunnel connection.
    #[cfg(target_os = "android")]
    pub fn get_socket_v6(&self) -> Result<RawFd> {
        let fd = unsafe { wgNetGetSocketV6(self.tunnel_handle) };
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

    /// Start UDP proxy through the tunnel to the given endpoint.
    ///
    /// Due to FFI boundary, direct communication is impossible. Instead a bidrectional UDP proxy listens on
    /// `listen_port`. The clients should connect to it in order to communicate with the exit endpoint over
    /// the tunnel.
    ///
    /// Note that the client traffic should originate from the `client_port` on the loopback interface.
    /// If `endpoint` belongs to IPv6 address family, then the `listen_port` is opened on `::1`, otherwise `127.0.0.1`.
    pub fn start_in_tunnel_udp_connection_proxy(
        &mut self,
        listen_port: u16,
        client_port: u16,
        endpoint: SocketAddr,
    ) -> Result<InTunnelUdpConnectionProxy> {
        let endpoint =
            CString::new(endpoint.to_string()).map_err(|_| Error::ConvertToCString("endpoint"))?;
        let mut out_listen_addr: *mut c_char = std::ptr::null_mut();
        let out_listen_addr_ptr: *mut *mut c_char = &mut out_listen_addr;
        let udp_proxy_handle = unsafe {
            wgNetStartUDPConnectionProxy(
                self.tunnel_handle,
                listen_port,
                client_port,
                endpoint.as_ptr(),
                out_listen_addr_ptr,
                wg_netstack_logger_callback,
                std::ptr::null_mut(),
            )
        };

        if udp_proxy_handle >= 0 {
            // SAFETY: libwg is expected to set a non-null value upon successful return.
            let listen_addr_cstr = unsafe { CStr::from_ptr(out_listen_addr) };

            let listen_addr = listen_addr_cstr
                .to_str()
                .map_err(|_| Error::ConvertToString("udp listen address"))
                .map(|s| s.to_owned());

            // SAFETY: free C string allocated in Go using the correct deallocator.
            unsafe { wgFreePtr(out_listen_addr as *mut _) };

            let listen_addr = listen_addr?;
            let listen_addr =
                SocketAddr::from_str(&listen_addr).map_err(|_| Error::ParseListenAddr)?;

            Ok(InTunnelUdpConnectionProxy::new(
                udp_proxy_handle,
                listen_addr,
            ))
        } else {
            Err(Error::StartUdpProxy(udp_proxy_handle))
        }
    }

    /// Start TCP proxy through the tunnel to the given endpoint.
    ///
    /// Due to FFI boundary, direct communication is impossible. Instead a bidirectional TCP proxy listens on a loopback port that can be obtained via [`InTunnelTcpConnectionProxy::listen_addr()`].
    /// The clients should connect to it in order to communicate with the endpoint over the tunnel. Each new connection established to proxy listen address
    /// will establish a new connection to the endpoint over the tunnel.
    ///
    /// If `endpoint` belongs to IPv6 address family, then the `listen_port` is opened on `::1`, otherwise `127.0.0.1`.
    pub fn start_in_tunnel_tcp_connection_proxy(
        &mut self,
        endpoint: SocketAddr,
    ) -> Result<InTunnelTcpConnectionProxy> {
        let endpoint_str = endpoint.to_string();
        let endpoint =
            CString::new(endpoint_str).map_err(|_| Error::ConvertToCString("endpoint"))?;
        let mut out_listen_addr: *mut c_char = std::ptr::null_mut();
        let out_listen_addr_ptr: *mut *mut c_char = &mut out_listen_addr;

        let tcp_proxy_handle = unsafe {
            wgNetStartTCPConnectionProxy(
                self.tunnel_handle,
                endpoint.as_ptr(),
                out_listen_addr_ptr,
                wg_netstack_logger_callback,
                std::ptr::null_mut(),
            )
        };

        if tcp_proxy_handle >= 0 {
            // SAFETY: libwg is expected to set a non-null value upon successful return.
            let listen_addr_cstr = unsafe { CStr::from_ptr(out_listen_addr) };

            let listen_addr = listen_addr_cstr
                .to_str()
                .map_err(|_| Error::ConvertToString("tcp listen address"))
                .map(|s| s.to_owned());

            // SAFETY: free C string allocated in Go using the correct deallocator.
            unsafe { wgFreePtr(out_listen_addr as *mut _) };

            let listen_addr = listen_addr?;
            let listen_addr =
                SocketAddr::from_str(&listen_addr).map_err(|_| Error::ParseListenAddr)?;

            Ok(InTunnelTcpConnectionProxy::new(
                tcp_proxy_handle,
                listen_addr,
            ))
        } else {
            Err(Error::StartTcpProxy(tcp_proxy_handle))
        }
    }

    fn stop_inner(&mut self) {
        if self.tunnel_handle >= 0 {
            unsafe { wgNetTurnOff(self.tunnel_handle) };
            self.tunnel_handle = -1;
        }
    }

    /// Re-attach itself to the new primary interface.
    ///
    /// Typically used on default route change.
    #[cfg(target_os = "ios")]
    pub fn bump_sockets(&mut self) {
        unsafe { wgNetBumpSockets(self.tunnel_handle) }
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

/// UDP connection proxy through the netstack tunnel.
#[derive(Debug)]
pub struct InTunnelUdpConnectionProxy {
    proxy_handle: i32,
    listen_addr: SocketAddr,
}

impl InTunnelUdpConnectionProxy {
    fn new(proxy_handle: i32, listen_addr: SocketAddr) -> Self {
        Self {
            proxy_handle,
            listen_addr,
        }
    }

    /// Returns local endpoint that can be used to proxy data to the remote endpoint.
    pub fn listen_addr(&self) -> SocketAddr {
        self.listen_addr
    }

    pub fn close(mut self) {
        self.close_inner()
    }

    fn close_inner(&mut self) {
        if self.proxy_handle >= 0 {
            unsafe { wgNetStopUDPConnectionProxy(self.proxy_handle) };
            self.proxy_handle = -1;
        }
    }
}

impl Drop for InTunnelUdpConnectionProxy {
    fn drop(&mut self) {
        self.close_inner();
    }
}

/// TCP connection proxy through the netstack tunnel.
#[derive(Debug)]
pub struct InTunnelTcpConnectionProxy {
    proxy_handle: i32,
    listen_addr: SocketAddr,
}

impl InTunnelTcpConnectionProxy {
    fn new(proxy_handle: i32, listen_addr: SocketAddr) -> Self {
        Self {
            proxy_handle,
            listen_addr,
        }
    }

    /// Returns local endpoint that can be used to proxy data to the remote endpoint.
    pub fn listen_addr(&self) -> SocketAddr {
        self.listen_addr
    }

    pub fn close(mut self) {
        self.close_inner()
    }

    fn close_inner(&mut self) {
        if self.proxy_handle >= 0 {
            unsafe { wgNetStopTCPConnectionProxy(self.proxy_handle) };
            self.proxy_handle = -1;
        }
    }
}

impl Drop for InTunnelTcpConnectionProxy {
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

unsafe extern "C" {
    /// Start the netstack tunnel.
    unsafe fn wgNetTurnOn(
        local_addresses: *const c_char,
        dns_addresses: *const c_char,
        mtu: i32,
        settings: *const c_char,
        logging_callback: LoggingCallback,
        logging_context: *mut c_void,
    ) -> i32;

    /// Pass a handle that was created by wgNetTurnOn to stop the wireguard tunnel.
    unsafe fn wgNetTurnOff(net_tunnel_handle: i32);

    /// Sets the config of the WireGuard interface.
    unsafe fn wgNetSetConfig(net_tunnel_handle: i32, settings: *const c_char) -> i64;

    /// Returns the config of the WireGuard interface.
    unsafe fn wgNetGetConfig(net_tunnel_handle: i32) -> *const c_char;

    /// Start UDP connection proxy through the netstack tunnel.
    ///
    /// Returns negative integer on error, otherwise the valid UDP proxy handle.
    unsafe fn wgNetStartUDPConnectionProxy(
        net_tunnel_handle: i32,
        listen_port: u16,
        client_port: u16,
        endpoint: *const c_char,
        out_listen_addr: *mut *mut c_char,
        logging_callback: LoggingCallback,
        logging_context: *mut c_void,
    ) -> i32;

    /// Stop UDP connection proxy.
    unsafe fn wgNetStopUDPConnectionProxy(udp_proxy_handle: i32);

    /// Start TCP connection proxy through the netstack tunnel.
    ///
    /// Returns negative integer on error, otherwise the valid TCP proxy handle.
    unsafe fn wgNetStartTCPConnectionProxy(
        net_tunnel_handle: i32,
        endpoint: *const c_char,
        out_listen_addr: *mut *mut c_char,
        logging_callback: LoggingCallback,
        logging_context: *mut c_void,
    ) -> i32;

    /// Stop TCP connection proxy.
    unsafe fn wgNetStopTCPConnectionProxy(tcp_proxy_handle: i32);

    /// Returns tunnel IPv4 socket.
    #[cfg(target_os = "android")]
    unsafe fn wgNetGetSocketV4(net_tunnel_handle: i32) -> i32;

    /// Returns tunnel IPv6 socket.
    #[cfg(target_os = "android")]
    unsafe fn wgNetGetSocketV6(net_tunnel_handle: i32) -> i32;

    /// Re-attach wireguard-go to the tunnel interface.
    #[cfg(target_os = "ios")]
    unsafe fn wgNetBumpSockets(net_tunnel_handle: i32);

    /// Re-bind tunnel socket to the new interface.
    ///
    /// - `family` - address family
    /// - `interface_index` - index of network interface to which the tunnel socket should be bound to. Pass 0 to bind to blackhole.
    #[cfg(windows)]
    unsafe fn wgNetRebindTunnelSocket(address_family: u16, interface_index: u32);

    /// Frees a pointer allocated by the go runtime - useful to free return value of wgGetConfig
    unsafe fn wgFreePtr(ptr: *mut c_void);
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
        let str = unsafe { CStr::from_ptr(msg).to_string_lossy() };
        let trimmed_str = str.trim_end();
        tracing::debug!("{}", trimmed_str);
    }
}
