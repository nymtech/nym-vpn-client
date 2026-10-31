// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Pure plugin state machine: daemon tunnel states in, NetworkManager signals out.

use std::{
    collections::HashMap,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
};

use nym_vpn_lib_types::{ActionAfterDisconnect, TunnelConnectionData, TunnelState};
use zbus::zvariant::{OwnedValue, Value};

use crate::nm::{failure, service_state};

#[derive(Debug, thiserror::Error)]
pub enum GatewayError {
    #[error("entry gateway {0} is IPv6; NetworkManager integration only supports IPv4 gateways")]
    Ipv6(Ipv6Addr),
}

/// The address NetworkManager is told the VPN talks to (`Config["gateway"]`).
///
/// It is the first hop the daemon actually sends packets to: the bridge when one is in use,
/// otherwise the entry WireGuard endpoint, or the mixnet entry gateway.
pub fn external_gateway(data: &TunnelConnectionData) -> Result<Ipv4Addr, GatewayError> {
    let ip = match data {
        TunnelConnectionData::Wireguard(wg) => wg
            .entry_bridge_addr
            .as_ref()
            .map(|bridge| bridge.remote_addr.ip())
            .unwrap_or_else(|| wg.entry.endpoint.ip()),
        TunnelConnectionData::Mixnet(mixnet) => mixnet.entry_ip,
    };
    match ip {
        IpAddr::V4(v4) => Ok(v4),
        IpAddr::V6(v6) => Err(GatewayError::Ipv6(v6)),
    }
}

/// IPv4 address as libnm expects it: the `in_addr` bytes read as a host `u32`.
pub fn nm_ipv4(addr: Ipv4Addr) -> u32 {
    u32::from_ne_bytes(addr.octets())
}

fn owned<'a, T: Into<Value<'a>>>(value: T) -> OwnedValue {
    value
        .into()
        .try_to_owned()
        .expect("primitive values never contain file descriptors")
}

/// The `Config` signal payload: gateway only, no tun device, no IP families.
pub fn config_dict(gateway: Ipv4Addr) -> HashMap<String, OwnedValue> {
    HashMap::from([
        ("gateway".to_string(), owned(nm_ipv4(gateway))),
        ("has-ip4".to_string(), owned(false)),
        ("has-ip6".to_string(), owned(false)),
        ("can-persist".to_string(), owned(false)),
    ])
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Idle,
    Starting,
    Started,
    Stopping,
    Stopped,
}

/// A NetworkManager signal the service must emit, in order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Effect {
    StateChanged(u32),
    Failure(u32),
    Config(Ipv4Addr),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisconnectDecision {
    /// The daemon is already down; just report STOPPED.
    AlreadyStopped,
    /// NetworkManager is tearing the VPN down on its own (network loss, sleep, timeout); report
    /// STOPPED and leave the daemon to its own recovery.
    LeaveDaemon,
    /// The user disconnected the VPN in NetworkManager; tear the tunnel down.
    DisconnectDaemon,
}

/// States in which NetworkManager shows the VPN as stopped. The plugin never tears the daemon
/// down from them: in `Error` the daemon keeps blocking traffic until it is told to connect or
/// disconnect, and in `Offline { reconnect: true }` it reconnects once the network is back.
pub fn is_terminal(state: &TunnelState) -> bool {
    matches!(
        state,
        TunnelState::Disconnected | TunnelState::Error(_) | TunnelState::Offline { .. }
    )
}

pub fn disconnect_decision(daemon: &TunnelState, user_requested: bool) -> DisconnectDecision {
    if is_terminal(daemon) {
        DisconnectDecision::AlreadyStopped
    } else if user_requested {
        DisconnectDecision::DisconnectDaemon
    } else {
        DisconnectDecision::LeaveDaemon
    }
}

pub struct Machine {
    phase: Phase,
}

impl Machine {
    pub fn new() -> Self {
        Self { phase: Phase::Idle }
    }

    pub fn phase(&self) -> Phase {
        self.phase
    }

    pub fn service_state(&self) -> u32 {
        match self.phase {
            Phase::Idle => service_state::INIT,
            Phase::Starting => service_state::STARTING,
            Phase::Started => service_state::STARTED,
            Phase::Stopping => service_state::STOPPING,
            Phase::Stopped => service_state::STOPPED,
        }
    }

    fn enter(&mut self, phase: Phase) -> Effect {
        tracing::debug!("Set new phase: {:?}", self.phase);
        self.phase = phase;
        Effect::StateChanged(self.service_state())
    }

    fn fail(&mut self, reason: u32) -> Vec<Effect> {
        vec![Effect::Failure(reason), self.enter(Phase::Stopped)]
    }

    pub fn begin_connect(&mut self) -> Vec<Effect> {
        match self.phase {
            Phase::Idle | Phase::Stopped => vec![self.enter(Phase::Starting)],
            _ => vec![],
        }
    }

    pub fn on_daemon_state(&mut self, state: &TunnelState) -> Vec<Effect> {
        tracing::info!("Received daemon state: {state:?}, phase: {:?}", self.phase);

        match self.phase {
            Phase::Starting => self.handle_starting_phase(&state),
            Phase::Started => self.handle_started_phase(&state),
            Phase::Stopping => self.handle_stopping_phase(&state),
            Phase::Idle | Phase::Stopped => vec![],
        }
    }

    fn handle_starting_phase(&mut self, state: &TunnelState) -> Vec<Effect> {
        match state {
            TunnelState::Connecting {
                connection_data, ..
            } => {
                if let Some(connection_data) =
                    connection_data.as_ref().and_then(|v| v.tunnel.as_ref())
                {
                    tracing::info!("Phase::Starting: Connecting!");
                    match external_gateway(&connection_data) {
                        Ok(gateway) => {
                            tracing::info!(
                                "Phase::Starting(Connecting): Effect(gateway = {gateway})"
                            );
                            vec![Effect::Config(gateway)]
                        }
                        Err(err) => {
                            tracing::error!("{err}");
                            self.fail(failure::BAD_IP_CONFIG)
                        }
                    }
                } else {
                    vec![]
                }
            }
            TunnelState::Connected { connection_data } => {
                tracing::info!("Phase::Starting: Connected!");
                match external_gateway(&connection_data.tunnel) {
                    Ok(gateway) => {
                        tracing::info!("Phase::Started(Connected): Effect(gateway = {gateway})");
                        vec![Effect::Config(gateway), self.enter(Phase::Started)]
                    }
                    Err(err) => {
                        tracing::error!("{err}");
                        self.fail(failure::BAD_IP_CONFIG)
                    }
                }
            }
            TunnelState::Disconnecting { after_disconnect } => match after_disconnect {
                ActionAfterDisconnect::Reconnect => {
                    vec![]
                }
                ActionAfterDisconnect::Nothing
                | ActionAfterDisconnect::Error
                | ActionAfterDisconnect::Offline => {
                    vec![self.enter(Phase::Stopping)]
                }
            },
            TunnelState::Disconnected | TunnelState::Error(_) | TunnelState::Offline { .. } => {
                self.fail(failure::CONNECT_FAILED)
            }
        }
    }

    fn handle_started_phase(&mut self, state: &TunnelState) -> Vec<Effect> {
        match state {
            TunnelState::Connected { connection_data } => {
                tracing::info!("Phase::Started: Connected!");
                match external_gateway(&connection_data.tunnel) {
                    Ok(gateway) => {
                        tracing::info!("Phase::Started: Effect(gateway = {gateway})");
                        vec![Effect::Config(gateway)]
                    }
                    Err(err) => {
                        tracing::error!("{err}");
                        vec![]
                    }
                }
            }
            TunnelState::Disconnecting { after_disconnect } => match after_disconnect {
                ActionAfterDisconnect::Reconnect => {
                    vec![self.enter(Phase::Starting)]
                }
                ActionAfterDisconnect::Nothing
                | ActionAfterDisconnect::Error
                | ActionAfterDisconnect::Offline => {
                    vec![self.enter(Phase::Stopping)]
                }
            },
            TunnelState::Connecting { .. } => vec![self.enter(Phase::Starting)],
            TunnelState::Disconnected | TunnelState::Error(_) | TunnelState::Offline { .. } => {
                vec![self.enter(Phase::Stopped)]
            }
        }
    }

    fn handle_stopping_phase(&mut self, state: &TunnelState) -> Vec<Effect> {
        if is_terminal(state) {
            vec![self.enter(Phase::Stopped)]
        } else {
            vec![]
        }
    }

    pub fn begin_disconnect(&mut self) -> Vec<Effect> {
        match self.phase {
            Phase::Starting | Phase::Started => vec![self.enter(Phase::Stopping)],
            _ => vec![],
        }
    }

    pub fn finish_stop(&mut self) -> Vec<Effect> {
        match self.phase {
            Phase::Starting | Phase::Started | Phase::Stopping => vec![self.enter(Phase::Stopped)],
            _ => vec![],
        }
    }

    pub fn on_stream_closed(&mut self) -> Vec<Effect> {
        self.finish_stop()
    }
}

#[cfg(test)]
pub(crate) mod fixtures {
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};

    use nym_vpn_lib_types::{
        ActionAfterDisconnect, BridgeAddress, ConnectionData, ErrorStateReason,
        EstablishConnectionState, GatewayLightInfo, MixnetConnectionData, NymAddress,
        SelectorFallbackState, TunnelConnectionData, TunnelState, TunnelType,
        WireguardConnectionData, WireguardNode,
    };

    pub(crate) fn gw(id: &str) -> GatewayLightInfo {
        GatewayLightInfo::new(id.to_string(), Some("CH".to_string()))
    }

    pub(crate) fn node(endpoint: SocketAddr) -> WireguardNode {
        WireguardNode {
            endpoint,
            public_key: "pk".to_string(),
            private_ipv4: Ipv4Addr::new(10, 0, 0, 2),
            private_ipv6: None,
        }
    }

    pub(crate) fn connected_with(tunnel: TunnelConnectionData) -> TunnelState {
        TunnelState::Connected {
            connection_data: ConnectionData {
                entry_gateway: gw("entry"),
                exit_gateway: gw("exit"),
                connected_at: time::OffsetDateTime::now_utc(),
                tunnel,
            },
        }
    }

    pub(crate) fn connected(entry_endpoint: SocketAddr, bridge: Option<SocketAddr>) -> TunnelState {
        connected_with(TunnelConnectionData::Wireguard(WireguardConnectionData {
            entry_bridge_addr: bridge.map(|remote_addr| BridgeAddress {
                listen_addr: "127.0.0.1:1".parse().unwrap(),
                remote_addr,
            }),
            entry: node(entry_endpoint),
            exit: node("10.1.1.1:51820".parse().unwrap()),
        }))
    }

    pub(crate) fn connected_mixnet(entry_ip: IpAddr) -> TunnelState {
        connected_with(TunnelConnectionData::Mixnet(MixnetConnectionData {
            nym_address: NymAddress::new("addr".into(), "entry".into()),
            exit_ipr: NymAddress::new("ipr".into(), "exit".into()),
            entry_ip,
            exit_ip: "9.9.9.10".parse().unwrap(),
            ipv4: Ipv4Addr::new(10, 0, 0, 2),
            ipv6: None,
        }))
    }

    pub(crate) fn connecting() -> TunnelState {
        TunnelState::Connecting {
            retry_attempt: 0,
            state: EstablishConnectionState::RefreshingGateways,
            selector_fallback_state: SelectorFallbackState {
                entry_fallback: false,
                exit_fallback: false,
            },
            tunnel_type: TunnelType::Wireguard,
            connection_data: None,
        }
    }

    pub(crate) fn error() -> TunnelState {
        TunnelState::Error(ErrorStateReason::Internal("boom".into()))
    }

    pub(crate) fn offline(reconnect: bool) -> TunnelState {
        TunnelState::Offline { reconnect }
    }

    pub(crate) fn disconnecting(after: ActionAfterDisconnect) -> TunnelState {
        TunnelState::Disconnecting {
            after_disconnect: after,
        }
    }

    pub(crate) const V4: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(1, 2, 3, 4)), 51820);
    pub(crate) const GW: Ipv4Addr = Ipv4Addr::new(1, 2, 3, 4);
}
