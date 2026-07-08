// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Module implementing ICMP connection probe

use std::{
    net::{IpAddr, SocketAddr},
    sync::atomic::{AtomicU16, Ordering},
    time::Duration,
};

use surge_ping::{
    Client as SurgeClient, Config as SurgeConfig, ICMP, PingIdentifier, PingSequence, SurgeError,
};

use super::{
    BoxedProbeError, ConnectionProbe, DEFAULT_IPV4_PROBE_IP, DEFAULT_IPV6_PROBE_IP, ProbeError,
};

/// Default ICMP identifier
const DEFAULT_ICMP_IDENT: u16 = 217;

/// ICMP probe configuration
#[derive(Debug, Clone)]
pub struct IcmpProbeConfig {
    /// ICMP identifier
    pub ident: u16,

    /// Probe IP address
    pub probe_ip: IpAddr,

    /// Bind socket to local address
    pub local_address: Option<IpAddr>,

    /// Bind socket to interface
    #[cfg(any(
        target_os = "linux",
        target_os = "android",
        target_os = "ios",
        target_os = "macos"
    ))]
    pub interface: Option<String>,
}

impl IcmpProbeConfig {
    /// Returns default configuration for probing over IPv4
    pub fn default_v4() -> Self {
        Self {
            ident: DEFAULT_ICMP_IDENT,
            probe_ip: IpAddr::V4(DEFAULT_IPV4_PROBE_IP),
            local_address: None,
            #[cfg(any(
                target_os = "linux",
                target_os = "android",
                target_os = "ios",
                target_os = "macos"
            ))]
            interface: None,
        }
    }

    /// Returns default configuration for probing over IPv6
    pub fn default_v6() -> Self {
        Self {
            ident: DEFAULT_ICMP_IDENT,
            probe_ip: IpAddr::V6(DEFAULT_IPV6_PROBE_IP),
            local_address: None,
            #[cfg(any(
                target_os = "linux",
                target_os = "android",
                target_os = "ios",
                target_os = "macos"
            ))]
            interface: None,
        }
    }

    /// Create new configuration with the given probe IP address
    pub fn new(probe_ip: impl Into<IpAddr>) -> Self {
        Self {
            ident: DEFAULT_ICMP_IDENT,
            probe_ip: probe_ip.into(),
            local_address: None,
            #[cfg(any(
                target_os = "linux",
                target_os = "android",
                target_os = "ios",
                target_os = "macos"
            ))]
            interface: None,
        }
    }

    /// Bind socket to local address
    pub fn with_local_address(mut self, local_addr: IpAddr) -> Self {
        self.local_address = Some(local_addr);
        self
    }

    /// Bind socket to interface
    #[cfg(any(
        target_os = "linux",
        target_os = "android",
        target_os = "ios",
        target_os = "macos"
    ))]
    pub fn with_interface(mut self, interface: String) -> Self {
        self.interface = Some(interface);
        self
    }
}

/// Public error type for the ICMP probe.
#[derive(Debug)]
pub struct IcmpProbeError {
    inner: IcmpProbeInnerError,
}

impl IcmpProbeError {
    pub fn inner(&self) -> &(dyn std::error::Error + 'static) {
        &self.inner
    }
}

impl std::fmt::Display for IcmpProbeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(f)
    }
}

impl std::error::Error for IcmpProbeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.inner.source()
    }
}

impl From<IcmpProbeInnerError> for IcmpProbeError {
    fn from(value: IcmpProbeInnerError) -> Self {
        Self { inner: value }
    }
}

/// Private error type for the ICMP probe.
#[derive(Debug, thiserror::Error)]
enum IcmpProbeInnerError {
    #[error("failed to send icmp packet")]
    Send(#[source] SurgeError),

    #[error("failed to create icmp client")]
    CreateIcmpClient(#[source] std::io::Error),

    #[cfg(any(target_os = "ios", target_os = "macos"))]
    #[error("failed to get interface index for {0}")]
    GetInterfaceIndex(String, #[source] nix::errno::Errno),

    #[cfg(any(target_os = "ios", target_os = "macos"))]
    #[error("received invalid interface index for {0}")]
    InvalidInterfaceIndex(String),
}

impl ProbeError for IcmpProbeError {
    fn is_timeout(&self) -> bool {
        matches!(
            self.inner,
            IcmpProbeInnerError::Send(SurgeError::Timeout { .. })
        )
    }

    fn is_send_failure(&self) -> bool {
        matches!(self.inner, IcmpProbeInnerError::Send(_)) && !self.is_timeout()
    }
}

impl From<IcmpProbeError> for BoxedProbeError {
    fn from(value: IcmpProbeError) -> Self {
        BoxedProbeError(Box::new(value))
    }
}

// The probe that sends ICMP packets directly to the destination
pub struct IcmpProbe {
    ident: u16,
    seq: AtomicU16,
    probe_ip: IpAddr,
    client: SurgeClient,
}

impl IcmpProbe {
    pub fn new(config: IcmpProbeConfig) -> Result<Self, IcmpProbeError> {
        let icmp_version = if config.probe_ip.is_ipv4() {
            ICMP::V4
        } else {
            ICMP::V6
        };
        let mut builder = SurgeConfig::builder().kind(icmp_version);
        if let Some(addr) = config.local_address {
            builder = builder.bind(SocketAddr::new(addr, 0));
        }

        #[cfg(any(target_os = "android", target_os = "linux"))]
        if let Some(interface) = config.interface.as_deref() {
            builder = builder.interface(interface);
        }

        #[cfg(any(target_os = "ios", target_os = "macos"))]
        if let Some(interface) = config.interface.as_deref() {
            // Convert interface name to index
            let index = nix::net::if_::if_nametoindex(interface)
                .map_err(|err| IcmpProbeInnerError::GetInterfaceIndex(interface.to_owned(), err))
                .and_then(|index| {
                    std::num::NonZeroU32::new(index).ok_or(
                        IcmpProbeInnerError::InvalidInterfaceIndex(interface.to_owned()),
                    )
                })?;
            builder = builder.interface_index(index);
        }

        let client_config = builder.build();
        let client =
            SurgeClient::new(&client_config).map_err(IcmpProbeInnerError::CreateIcmpClient)?;

        Ok(Self {
            ident: config.ident,
            seq: AtomicU16::new(0),
            probe_ip: config.probe_ip,
            client,
        })
    }
}

#[async_trait::async_trait]
impl ConnectionProbe for IcmpProbe {
    async fn send(&self, timeout: Duration) -> Result<(), BoxedProbeError> {
        let seq = self.seq.fetch_add(1, Ordering::SeqCst);
        let ident = PingIdentifier(self.ident);

        let mut pinger = self.client.pinger(self.probe_ip, ident).await;
        pinger.timeout(timeout);
        let (_packet, _time) = pinger
            .ping(PingSequence(seq), &[])
            .await
            .map_err(|err| IcmpProbeError::from(IcmpProbeInnerError::Send(err)))?;

        Ok(())
    }
}

// Running these tests on Linux requires CAP_NET_RAW
#[cfg(test)]
mod tests {
    use std::net::{Ipv4Addr, Ipv6Addr};

    use super::*;

    #[tokio::test]
    async fn test_send_loopback_icmp_v4() {
        let probe = create_loopback_probe_with_addr(Ipv4Addr::LOCALHOST);
        probe
            .send(Duration::from_secs(1))
            .await
            .expect("failed to send probe");
    }

    #[tokio::test]
    async fn test_send_loopback_icmp_v6() {
        let probe = create_loopback_probe_with_addr(Ipv6Addr::LOCALHOST);
        probe
            .send(Duration::from_secs(1))
            .await
            .expect("failed to send probe");
    }

    fn create_loopback_probe_with_addr(probe_ip: impl Into<IpAddr>) -> IcmpProbe {
        let probe_ip: IpAddr = probe_ip.into();
        let mut config = IcmpProbeConfig::new(probe_ip);
        #[cfg(any(target_os = "linux", target_os = "android"))]
        {
            config = config.with_interface("lo".to_owned());
        }
        #[cfg(any(target_os = "ios", target_os = "macos"))]
        {
            config = config.with_interface("lo0".to_owned());
        }
        #[cfg(not(any(
            target_os = "linux",
            target_os = "android",
            target_os = "ios",
            target_os = "macos"
        )))]
        {
            config = config.with_local_address(probe_ip);
        }

        IcmpProbe::new(config).expect("failed to create icmp probe")
    }
}
