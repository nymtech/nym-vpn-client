// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

mod connection_monitor;
mod connection_probe;
mod icmp_probe;
#[cfg(test)]
mod mock_probe;
mod tcp_probe;

use std::net::{Ipv4Addr, Ipv6Addr};

pub use connection_monitor::{
    ConnectionEvent, ConnectionMonitor, ConnectionStatusEvent, Error, Phase, TimingConfig,
};
pub use connection_probe::{BoxedProbeError, ConnectionProbe, ProbeError};
pub use icmp_probe::{IcmpProbe, IcmpProbeConfig, IcmpProbeError};
pub use tcp_probe::{TcpProbe, TcpProbeConfig, TcpProbeError};

/// Default host used for probing via IPv4
/// The same hosts are used by ICMP and TCP probes.
const DEFAULT_IPV4_PROBE_IP: Ipv4Addr = Ipv4Addr::new(1, 1, 1, 1);

/// Default host used for probing via IPv6
/// The same hosts are used by ICMP and TCP probes.
const DEFAULT_IPV6_PROBE_IP: Ipv6Addr = Ipv6Addr::new(0x2606, 0x4700, 0x4700, 0, 0, 0, 0, 0x1111);

/// Default port for probing the above hosts over TCP
const DEFAULT_TCP_PROBE_PORT: u16 = 80;
