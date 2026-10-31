// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Pure plugin state machine: daemon tunnel states in, NetworkManager signals out.

use std::net::IpAddr;

use nym_vpn_lib_types::{ActionAfterDisconnect, TunnelConnectionData, TunnelState};

use crate::nm::{failure, service_state};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Idle,
    Starting,
    Started,
    Stopping,
    Stopped,
}

impl Phase {
    pub fn service_state(&self) -> u32 {
        match self {
            Self::Idle => service_state::INIT,
            Self::Starting => service_state::STARTING,
            Self::Started => service_state::STARTED,
            Self::Stopping => service_state::STOPPING,
            Self::Stopped => service_state::STOPPED,
        }
    }
}

/// A NetworkManager signal the service must emit, in order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Effect {
    StateChanged(u32),
    Failure(u32),
    Config(IpAddr),
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

    fn enter(&mut self, phase: Phase) -> Effect {
        self.phase = phase;
        Effect::StateChanged(phase.service_state())
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

    #[tracing::instrument(skip(self))]
    pub fn on_daemon_state(&mut self, state: &TunnelState) -> Vec<Effect> {
        tracing::info!("Received daemon state: {state:?}, phase: {:?}", self.phase);

        match self.phase {
            Phase::Starting => self.handle_starting_phase(state),
            Phase::Started => self.handle_started_phase(state),
            Phase::Stopping => self.handle_stopping_phase(state),
            Phase::Idle | Phase::Stopped => vec![],
        }
    }

    #[tracing::instrument(skip(self))]
    fn handle_starting_phase(&mut self, state: &TunnelState) -> Vec<Effect> {
        match state {
            TunnelState::Connecting { .. } => {
                vec![]
            }
            TunnelState::Connected { connection_data } => {
                let gateway = external_gateway(&connection_data.tunnel);
                vec![Effect::Config(gateway), self.enter(Phase::Started)]
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

    #[tracing::instrument(skip(self))]
    fn handle_started_phase(&mut self, state: &TunnelState) -> Vec<Effect> {
        match state {
            TunnelState::Connected { connection_data } => {
                vec![Effect::Config(external_gateway(&connection_data.tunnel))]
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
            TunnelState::Connecting { .. } => vec![],
            TunnelState::Disconnected | TunnelState::Error(_) | TunnelState::Offline { .. } => {
                vec![self.enter(Phase::Stopped)]
            }
        }
    }

    #[tracing::instrument(skip(self))]
    fn handle_stopping_phase(&mut self, state: &TunnelState) -> Vec<Effect> {
        match state {
            TunnelState::Disconnected | TunnelState::Error(_) | TunnelState::Offline { .. } => {
                vec![self.enter(Phase::Stopped)]
            }
            TunnelState::Disconnecting { .. }
            | TunnelState::Connecting { .. }
            | TunnelState::Connected { .. } => vec![],
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

/// The address NetworkManager is told the VPN talks to (`Config["gateway"]`).
///
/// It is the first hop the daemon actually sends packets to: the bridge when one is in use,
/// otherwise the entry WireGuard endpoint, or the mixnet entry gateway.
fn external_gateway(data: &TunnelConnectionData) -> IpAddr {
    match data {
        TunnelConnectionData::Wireguard(wg) => wg
            .entry_bridge_addr
            .as_ref()
            .map(|bridge| bridge.remote_addr.ip())
            .unwrap_or_else(|| wg.entry.endpoint.ip()),
        TunnelConnectionData::Mixnet(mixnet) => mixnet.entry_ip,
    }
}
