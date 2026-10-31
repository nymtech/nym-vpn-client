// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Pure plugin state machine: daemon tunnel states in, NetworkManager signals out.

use std::{
    collections::HashMap,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
};

use nym_vpn_lib_types::{ActionAfterDisconnect, ConnectionData, TunnelConnectionData, TunnelState};
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
pub fn external_gateway(data: &ConnectionData) -> Result<Ipv4Addr, GatewayError> {
    let ip = match &data.tunnel {
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
            Phase::Starting => match state {
                TunnelState::Connected { connection_data } => {
                    tracing::info!("Phase::Starting: Connected!");
                    match external_gateway(connection_data) {
                        Ok(gateway) => {
                            tracing::info!("Phase::Started: Effect(gateway = {gateway})");
                            vec![Effect::Config(gateway), self.enter(Phase::Started)]
                        }
                        Err(err) => {
                            tracing::error!("{err}");
                            self.fail(failure::BAD_IP_CONFIG)
                        }
                    }
                }
                state if is_terminal(state) => self.fail(failure::CONNECT_FAILED),
                _ => vec![],
            },
            Phase::Started => match state {
                TunnelState::Connected { connection_data } => {
                    tracing::info!("Phase::Started: Connected!");
                    match external_gateway(connection_data) {
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
                // Going back to STARTING tells NetworkManager the VPN is reconnecting; it shows the
                // connection as activating until the next Config and STARTED.
                TunnelState::Disconnecting {
                    after_disconnect: ActionAfterDisconnect::Reconnect,
                }
                | TunnelState::Connecting { .. } => vec![self.enter(Phase::Starting)],
                state if is_terminal(state) => vec![self.enter(Phase::Stopped)],
                _ => vec![],
            },
            Phase::Stopping => {
                if is_terminal(state) {
                    vec![self.enter(Phase::Stopped)]
                } else {
                    vec![]
                }
            }
            Phase::Idle | Phase::Stopped => vec![],
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

#[cfg(test)]
mod tests {
    use std::net::Ipv4Addr;

    use nym_vpn_lib_types::{ActionAfterDisconnect, TunnelState};

    use super::{fixtures::*, *};
    use crate::nm::{failure, service_state};

    fn started_machine() -> Machine {
        let mut m = Machine::new();
        m.begin_connect();
        m.on_daemon_state(&connected(V4, None));
        assert_eq!(m.phase(), Phase::Started);
        m
    }

    #[test]
    fn nm_ipv4_is_network_byte_order() {
        assert_eq!(
            nm_ipv4(Ipv4Addr::new(10, 0, 0, 1)),
            u32::from_ne_bytes([10, 0, 0, 1])
        );
    }

    #[test]
    fn gateway_is_entry_endpoint_for_wireguard() {
        let TunnelState::Connected { connection_data } = connected(V4, None) else {
            unreachable!()
        };
        assert_eq!(external_gateway(&connection_data).unwrap(), GW);
    }

    #[test]
    fn gateway_prefers_bridge_remote() {
        let TunnelState::Connected { connection_data } =
            connected(V4, Some("5.6.7.8:4443".parse().unwrap()))
        else {
            unreachable!()
        };
        assert_eq!(
            external_gateway(&connection_data).unwrap(),
            Ipv4Addr::new(5, 6, 7, 8)
        );
    }

    #[test]
    fn gateway_is_entry_ip_for_mixnet() {
        let TunnelState::Connected { connection_data } =
            connected_mixnet("9.9.9.9".parse().unwrap())
        else {
            unreachable!()
        };
        assert_eq!(
            external_gateway(&connection_data).unwrap(),
            Ipv4Addr::new(9, 9, 9, 9)
        );
    }

    #[test]
    fn gateway_ipv6_is_error() {
        let TunnelState::Connected { connection_data } =
            connected("[2001:db8::1]:51820".parse().unwrap(), None)
        else {
            unreachable!()
        };
        assert!(matches!(
            external_gateway(&connection_data),
            Err(GatewayError::Ipv6(_))
        ));
    }

    #[test]
    fn config_dict_has_only_the_four_keys() {
        let d = config_dict(GW);
        assert_eq!(d.len(), 4);
        assert_eq!(u32::try_from(d["gateway"].clone()).unwrap(), nm_ipv4(GW));
        assert!(!bool::try_from(d["has-ip4"].clone()).unwrap());
        assert!(!bool::try_from(d["has-ip6"].clone()).unwrap());
        assert!(!bool::try_from(d["can-persist"].clone()).unwrap());
        assert!(!d.contains_key("tundev"));
    }

    #[test]
    fn begin_connect_emits_starting() {
        let mut m = Machine::new();
        assert_eq!(m.service_state(), service_state::INIT);
        assert_eq!(
            m.begin_connect(),
            vec![Effect::StateChanged(service_state::STARTING)]
        );
        assert_eq!(m.phase(), Phase::Starting);
    }

    #[test]
    fn connected_while_starting_emits_config_then_started() {
        let mut m = Machine::new();
        m.begin_connect();
        assert_eq!(
            m.on_daemon_state(&connected(V4, None)),
            vec![
                Effect::Config(GW),
                Effect::StateChanged(service_state::STARTED)
            ]
        );
        assert_eq!(m.phase(), Phase::Started);
        assert_eq!(m.service_state(), service_state::STARTED);
    }

    #[test]
    fn connected_with_ipv6_gateway_fails_bad_ip_config() {
        let mut m = Machine::new();
        m.begin_connect();
        assert_eq!(
            m.on_daemon_state(&connected("[2001:db8::1]:51820".parse().unwrap(), None)),
            vec![
                Effect::Failure(failure::BAD_IP_CONFIG),
                Effect::StateChanged(service_state::STOPPED)
            ]
        );
        assert_eq!(m.phase(), Phase::Stopped);
    }

    #[test]
    fn error_while_starting_fails_connect() {
        let mut m = Machine::new();
        m.begin_connect();
        assert_eq!(
            m.on_daemon_state(&error()),
            vec![
                Effect::Failure(failure::CONNECT_FAILED),
                Effect::StateChanged(service_state::STOPPED)
            ]
        );
        let mut m = Machine::new();
        m.begin_connect();
        assert_eq!(
            m.on_daemon_state(&TunnelState::Disconnected),
            vec![
                Effect::Failure(failure::CONNECT_FAILED),
                Effect::StateChanged(service_state::STOPPED)
            ]
        );
    }

    #[test]
    fn offline_no_reconnect_while_starting_fails_connect() {
        let mut m = Machine::new();
        m.begin_connect();
        assert_eq!(
            m.on_daemon_state(&offline(false)),
            vec![
                Effect::Failure(failure::CONNECT_FAILED),
                Effect::StateChanged(service_state::STOPPED)
            ]
        );
    }

    #[test]
    fn reconnect_cycle_while_starting_is_ignored() {
        let mut m = Machine::new();
        m.begin_connect();
        assert_eq!(m.on_daemon_state(&connecting()), vec![]);
        assert_eq!(
            m.on_daemon_state(&disconnecting(ActionAfterDisconnect::Reconnect)),
            vec![]
        );
        assert_eq!(m.phase(), Phase::Starting);
    }

    #[test]
    fn offline_recovery_while_starting_fails_connect() {
        let mut m = Machine::new();
        m.begin_connect();
        assert_eq!(
            m.on_daemon_state(&offline(true)),
            vec![
                Effect::Failure(failure::CONNECT_FAILED),
                Effect::StateChanged(service_state::STOPPED)
            ]
        );
    }

    #[test]
    fn reconnect_while_started_goes_back_to_starting() {
        let mut m = started_machine();
        assert_eq!(
            m.on_daemon_state(&disconnecting(ActionAfterDisconnect::Reconnect)),
            vec![Effect::StateChanged(service_state::STARTING)]
        );
        assert_eq!(m.on_daemon_state(&connecting()), vec![]);
        assert_eq!(
            m.on_daemon_state(&connected(V4, None)),
            vec![
                Effect::Config(GW),
                Effect::StateChanged(service_state::STARTED)
            ]
        );
        assert_eq!(m.phase(), Phase::Started);
    }

    #[test]
    fn connecting_while_started_goes_back_to_starting() {
        let mut m = started_machine();
        assert_eq!(
            m.on_daemon_state(&connecting()),
            vec![Effect::StateChanged(service_state::STARTING)]
        );
        assert_eq!(m.phase(), Phase::Starting);
    }

    #[test]
    fn disconnect_without_reconnect_while_started_is_ignored() {
        let mut m = started_machine();
        assert_eq!(
            m.on_daemon_state(&disconnecting(ActionAfterDisconnect::Nothing)),
            vec![]
        );
        assert_eq!(m.phase(), Phase::Started);
    }

    #[test]
    fn reconnect_through_another_gateway_resends_config() {
        let mut m = started_machine();
        let other = Ipv4Addr::new(5, 6, 7, 8);
        assert_eq!(
            m.on_daemon_state(&connected("5.6.7.8:51820".parse().unwrap(), None)),
            vec![Effect::Config(other)]
        );
        assert_eq!(
            m.on_daemon_state(&connected("5.6.7.8:51820".parse().unwrap(), None)),
            vec![]
        );
        assert_eq!(m.phase(), Phase::Started);
    }

    #[test]
    fn terminal_while_started_emits_stopped() {
        for terminal in [
            TunnelState::Disconnected,
            error(),
            offline(false),
            offline(true),
        ] {
            let mut m = started_machine();
            assert_eq!(
                m.on_daemon_state(&terminal),
                vec![Effect::StateChanged(service_state::STOPPED)],
                "{terminal}"
            );
            assert_eq!(m.phase(), Phase::Stopped);
        }
    }

    #[test]
    fn begin_disconnect_emits_stopping_then_finish_stop_emits_stopped() {
        let mut m = started_machine();
        assert_eq!(
            m.begin_disconnect(),
            vec![Effect::StateChanged(service_state::STOPPING)]
        );
        assert_eq!(m.phase(), Phase::Stopping);
        assert_eq!(m.on_daemon_state(&connecting()), vec![]);
        assert_eq!(
            m.finish_stop(),
            vec![Effect::StateChanged(service_state::STOPPED)]
        );
        assert_eq!(m.phase(), Phase::Stopped);
        assert_eq!(m.finish_stop(), vec![]);
    }

    #[test]
    fn terminal_while_stopping_emits_stopped() {
        for terminal in [
            TunnelState::Disconnected,
            error(),
            offline(false),
            offline(true),
        ] {
            let mut m = started_machine();
            m.begin_disconnect();
            assert_eq!(
                m.on_daemon_state(&terminal),
                vec![Effect::StateChanged(service_state::STOPPED)],
                "{terminal}"
            );
        }
    }

    #[test]
    fn stream_closed_while_started_emits_stopped_and_idle_is_noop() {
        let mut m = started_machine();
        assert_eq!(
            m.on_stream_closed(),
            vec![Effect::StateChanged(service_state::STOPPED)]
        );
        assert_eq!(Machine::new().on_stream_closed(), vec![]);
        assert_eq!(m.on_stream_closed(), vec![]);
    }

    #[test]
    fn begin_connect_is_allowed_again_after_stopped() {
        let mut m = started_machine();
        m.on_daemon_state(&TunnelState::Disconnected);
        assert_eq!(
            m.begin_connect(),
            vec![Effect::StateChanged(service_state::STARTING)]
        );
    }

    #[test]
    fn disconnect_decision_table() {
        for terminal in [
            TunnelState::Disconnected,
            error(),
            offline(false),
            offline(true),
        ] {
            for user_requested in [true, false] {
                assert_eq!(
                    disconnect_decision(&terminal, user_requested),
                    DisconnectDecision::AlreadyStopped
                );
            }
        }
        for up in [connected(V4, None), connecting()] {
            assert_eq!(
                disconnect_decision(&up, true),
                DisconnectDecision::DisconnectDaemon,
                "{up}"
            );
            assert_eq!(
                disconnect_decision(&up, false),
                DisconnectDecision::LeaveDaemon,
                "{up}"
            );
        }
    }
}
