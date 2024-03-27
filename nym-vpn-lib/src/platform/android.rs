// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: Apache-2.0

use super::*;
use ipnetwork::IpNetwork;
use jnix::jni::{
    objects::JObject,
    sys::{jboolean, jint, JNI_FALSE},
    JNIEnv,
};
use jnix::{IntoJava, JnixEnv};
use nix::sys::{
    select::{pselect, FdSet},
    time::{TimeSpec, TimeValLike},
};
use rand::{thread_rng, Rng};
use std::io;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, UdpSocket};
use std::os::fd::RawFd;
use std::sync::{Arc, Mutex, Once};
use std::time::{Duration, Instant};
use talpid_tunnel::tun_provider::{TunConfig, TunProvider};
use talpid_types::android::AndroidContext;
use talpid_types::ErrorExt;

static LOAD_CLASSES: Once = Once::new();

lazy_static! {
    static ref CONTEXT: Mutex<Option<AndroidContext>> = Mutex::new(None);
}

pub const CLASSES: &[&str] = &[
    "java/lang/Boolean",
    "java/net/InetAddress",
    "java/net/InetSocketAddress",
    "java/util/ArrayList",
    "net/nymtech/vpn/net/Endpoint",
    "net/nymtech/vpn/net/TransportProtocol",
    "net/nymtech/vpn/net/TunnelEndpoint",
    "net/nymtech/vpn/tun_provider/InetNetwork",
    "net/nymtech/vpn/tun_provider/TunConfig",
    "net/nymtech/vpn/tunnel/ActionAfterDisconnect",
    "net/nymtech/vpn/tunnel/ErrorState",
    "net/nymtech/vpn/tunnel/ErrorStateCause$AuthFailed",
    "net/nymtech/vpn/tunnel/ErrorStateCause$Ipv6Unavailable",
    "net/nymtech/vpn/tunnel/ErrorStateCause$SetFirewallPolicyError",
    "net/nymtech/vpn/tunnel/ErrorStateCause$SetDnsError",
    "net/nymtech/vpn/tunnel/ErrorStateCause$StartTunnelError",
    "net/nymtech/vpn/tunnel/ErrorStateCause$TunnelParameterError",
    "net/nymtech/vpn/tunnel/ErrorStateCause$IsOffline",
    "net/nymtech/vpn/tunnel/ErrorStateCause$InvalidDnsServers",
    "net/nymtech/vpn/tunnel/ErrorStateCause$VpnPermissionDenied",
    "net/nymtech/vpn/tunnel/ParameterGenerationError",
    "net/nymtech/vpn/ConnectivityListener",
    "net/nymtech/vpn/CreateTunResult$Success",
    "net/nymtech/vpn/CreateTunResult$InvalidDnsServers",
    "net/nymtech/vpn/CreateTunResult$PermissionDenied",
    "net/nymtech/vpn/CreateTunResult$TunnelDeviceError",
    "net/nymtech/vpn/NymVpnService",
];

pub(crate) struct TunnelConfiguration {
    pub(crate) tun_provider: Arc<Mutex<TunProvider>>,
    pub(crate) gateway_fd: Option<RawFd>,
}

fn init_jni_logger() {
    use android_logger::{Config, FilterBuilder};

    android_logger::init_once(
        Config::default()
            .with_max_level(LevelFilter::Trace)
            .with_tag("libnymvpn")
            .with_filter(
                FilterBuilder::new()
                    .parse("debug,tungstenite=warn,mio=warn,tokio_tungstenite=warn")
                    .build(),
            ),
    );
    log::debug!("Logger initialized");
}

pub(crate) fn get_context() -> Option<AndroidContext> {
    CONTEXT.lock().unwrap().clone()
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_nymtech_vpn_NymVpnService_initVPN(
    env: JNIEnv<'_>,
    _this: JObject<'_>,
    vpn_service: JObject<'_>,
) {
    if get_context().is_some() {
        warn!("Context was already initialised, not doing anything");
        return;
    }

    init_jni_logger();

    let env = JnixEnv::from(env);
    let jvm = if let Ok(data) = env.get_java_vm() {
        Arc::new(data)
    } else {
        warn!("Java VM is not available. Aborting");
        return;
    };
    let vpn_service = if let Ok(vpn) = env.new_global_ref(vpn_service) {
        vpn
    } else {
        warn!("VPN object is not available. Aborting");
        return;
    };
    let context = AndroidContext { jvm, vpn_service };

    LOAD_CLASSES.call_once(|| env.preload_classes(CLASSES.iter().cloned()));

    *CONTEXT.lock().unwrap() = Some(context);
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_nymtech_vpn_NymVpnService_defaultTunConfig<'env>(
    env: JNIEnv<'env>,
    _this: JObject<'_>,
) -> JObject<'env> {
    let env = JnixEnv::from(env);
    TunConfig::default().into_java(&env).forget()
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern "system" fn Java_net_nymtech_vpn_NymVpnService_waitForTunnelUp(
    _: JNIEnv<'_>,
    _this: JObject<'_>,
    tunFd: jint,
    isIpv6Enabled: jboolean,
) {
    let tun_fd = tunFd as RawFd;
    let is_ipv6_enabled = isIpv6Enabled != JNI_FALSE;

    if let Err(error) = wait_for_tunnel_up(tun_fd, is_ipv6_enabled) {
        log::error!(
            "{}",
            error.display_chain_with_msg("Failed to wait for tunnel device to be usable")
        );
    }
}

#[derive(Debug, err_derive::Error)]
#[error(no_from)]
enum SendRandomDataError {
    #[error(display = "failed to bind an UDP socket")]
    BindUdpSocket(#[error(source)] io::Error),

    #[error(display = "failed to send random data through UDP socket")]
    SendToUdpSocket(#[error(source)] io::Error),
}

#[derive(Debug, err_derive::Error)]
enum Error {
    #[error(display = "failed to verify the tunnel device")]
    VerifyTunDevice(#[error(source)] SendRandomDataError),

    #[error(display = "failed to select() on tunnel device")]
    Select(#[error(source)] nix::Error),

    #[error(display = "timed out while waiting for tunnel device to receive data")]
    TunnelDeviceTimeout,
    #[error(display = "timed out while waiting for tunnel device to receive data")]
    ParseExplorerUrlFailed,
}

fn wait_for_tunnel_up(tun_fd: RawFd, is_ipv6_enabled: bool) -> Result<(), Error> {
    let mut fd_set = FdSet::new();
    fd_set.insert(tun_fd);
    let timeout = TimeSpec::microseconds(300);
    const TIMEOUT: Duration = Duration::from_secs(60);
    let start = Instant::now();
    while start.elapsed() < TIMEOUT {
        // if tunnel device is ready to be read from, traffic is being routed through it
        if pselect(None, Some(&mut fd_set), None, None, Some(&timeout), None)? > 0 {
            return Ok(());
        }
        // have to add tun_fd back into the bitset
        fd_set.insert(tun_fd);
        try_sending_random_udp(is_ipv6_enabled)?;
    }

    Err(Error::TunnelDeviceTimeout)
}

fn try_sending_random_udp(is_ipv6_enabled: bool) -> Result<(), SendRandomDataError> {
    let mut tried_ipv6 = false;
    const TIMEOUT: Duration = Duration::from_millis(300);
    let start = Instant::now();

    while start.elapsed() < TIMEOUT {
        // pick any random route to select between Ipv4 and Ipv6
        // TODO: if we are to allow LAN on Android by changing the routes that are stuffed in
        // TunConfig, then this should be revisited to be fair between IPv4 and IPv6
        let should_generate_ipv4 = !is_ipv6_enabled || thread_rng().gen();

        let rand_port = thread_rng().gen();
        let (local_addr, rand_dest_addr) = if should_generate_ipv4 || tried_ipv6 {
            let mut ipv4_bytes = [0u8; 4];
            thread_rng().fill(&mut ipv4_bytes);
            (
                SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), 0),
                SocketAddr::new(IpAddr::from(ipv4_bytes), rand_port),
            )
        } else {
            let mut ipv6_bytes = [0u8; 16];
            tried_ipv6 = true;
            thread_rng().fill(&mut ipv6_bytes);
            (
                SocketAddr::new(Ipv6Addr::UNSPECIFIED.into(), 0),
                SocketAddr::new(IpAddr::from(ipv6_bytes), rand_port),
            )
        };

        // TODO: once https://github.com/rust-lang/rust/issues/27709 is resolved, please use
        // `is_global()` to check if a new address should be attempted.
        if !is_public_ip(rand_dest_addr.ip()) {
            continue;
        }

        let socket = UdpSocket::bind(local_addr).map_err(SendRandomDataError::BindUdpSocket)?;

        let mut buf = vec![0u8; thread_rng().gen_range(17, 214)];
        // fill buff with random data
        thread_rng().fill(buf.as_mut_slice());
        match socket.send_to(&buf, rand_dest_addr) {
            Ok(_) => return Ok(()),
            Err(err) => {
                if tried_ipv6 {
                    continue;
                }
                match err.raw_os_error() {
                    // Error code 101 - specified network is unreachable
                    // Error code 22 - specified address is not usable
                    Some(101) | Some(22) => {
                        // if we failed whilst trying to send to IPv6, we should not try
                        // IPv6 again.
                        continue;
                    }
                    _ => return Err(SendRandomDataError::SendToUdpSocket(err)),
                }
            }
        };
    }
    Ok(())
}

fn is_public_ip(addr: IpAddr) -> bool {
    match addr {
        IpAddr::V4(ipv4) => {
            // 0.x.x.x is not a publicly routable address
            if ipv4.octets()[0] == 0u8 {
                return false;
            }
        }
        IpAddr::V6(ipv6) => {
            if ipv6.segments()[0] == 0u16 {
                return false;
            }
        }
    }
    // A non-exhaustive list of non-public subnets
    let publicly_unroutable_subnets: Vec<IpNetwork> = vec![
        // IPv4 local networks
        "10.0.0.0/8".parse().unwrap(),
        "172.16.0.0/12".parse().unwrap(),
        "192.168.0.0/16".parse().unwrap(),
        // IPv4 non-forwardable network
        "169.254.0.0/16".parse().unwrap(),
        "192.0.0.0/8".parse().unwrap(),
        // Documentation networks
        "192.0.2.0/24".parse().unwrap(),
        "198.51.100.0/24".parse().unwrap(),
        "203.0.113.0/24".parse().unwrap(),
        // IPv6 publicly unroutable networks
        "fc00::/7".parse().unwrap(),
        "fe80::/10".parse().unwrap(),
    ];

    !publicly_unroutable_subnets
        .iter()
        .any(|net| net.contains(addr))
}
