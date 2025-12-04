// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::time::Instant;

use nym_vpn_lib_types::{ActionAfterDisconnect, TunnelState};
use tokio::sync::mpsc::UnboundedSender;
use tokio_util::sync::CancellationToken;

/// Channel receiving generic stats events to be used by a statistics aggregator.
pub(crate) type StatisticsReceiver = tokio::sync::mpsc::UnboundedReceiver<StatisticsEvent>;

/// Channel allowing generic statistics events to be reported to a stats event aggregator
#[derive(Clone)]
pub struct StatisticsSender {
    stats_tx: UnboundedSender<StatisticsEvent>,
    cancel_token: CancellationToken,
}

impl StatisticsSender {
    /// Create a new statistics Sender
    pub fn new(
        stats_tx: UnboundedSender<StatisticsEvent>,
        cancel_token: CancellationToken,
    ) -> Self {
        StatisticsSender {
            stats_tx,
            cancel_token,
        }
    }

    /// Report a statistics event using the sender.
    pub fn report(&self, event: StatisticsEvent) {
        if let Err(err) = self.stats_tx.send(event)
            && !self.cancel_token.is_cancelled()
        {
            tracing::error!("Failed to send stats event: {err}");
        }
    }

    pub fn report_connection_request(&self) {
        self.report(StatisticsEvent::new_connect_request());
    }

    pub fn report_disconnection_request(&self) {
        self.report(StatisticsEvent::new_disconnect_request());
    }

    pub fn report_tunnel_state(&self, state: TunnelState) {
        self.report(StatisticsEvent::usage_event_from_state(state.clone()));
        self.report(StatisticsEvent::sending_event_from_state(state));
    }
}

/// Statistics events
#[derive(Debug, Clone)]
pub enum StatisticsEvent {
    Usage(UsageEvent),
    ReportSending(ReportSendingEvent),
}

impl StatisticsEvent {
    fn new_connect_request() -> Self {
        Self::Usage(UsageEvent::ConnectRequest(Instant::now()))
    }
    fn new_disconnect_request() -> Self {
        Self::Usage(UsageEvent::DisconnectRequest(Instant::now()))
    }
    fn new_connecting(
        retry_attempt: u32,
        maybe_exit_id: Option<String>,
        maybe_exit_cc: Option<String>,
        maybe_tunnel_type: Option<String>,
    ) -> Self {
        Self::Usage(UsageEvent::Connecting {
            instant: Instant::now(),
            retry_attempt,
            maybe_exit_id,
            maybe_exit_cc,
            maybe_tunnel_type,
        })
    }

    fn new_connected(exit_id: String, exit_cc: Option<String>, tunnel_type: String) -> Self {
        Self::Usage(UsageEvent::Connected {
            instant: Instant::now(),
            exit_id,
            exit_cc,
            tunnel_type,
        })
    }

    fn new_disconnecting(after_disconnect: ActionAfterDisconnect) -> Self {
        Self::Usage(UsageEvent::Disconnecting {
            instant: Instant::now(),
            after_disconnect,
        })
    }

    fn new_disconnected() -> Self {
        Self::Usage(UsageEvent::Disconnected(Instant::now()))
    }

    fn new_error(error: impl ToString) -> Self {
        Self::Usage(UsageEvent::Error {
            instant: Instant::now(),
            error: error.to_string(),
        })
    }

    fn new_offline(reconnect: bool) -> Self {
        Self::Usage(UsageEvent::Offline {
            instant: Instant::now(),
            reconnect,
        })
    }

    fn usage_event_from_state(state: TunnelState) -> Self {
        match state {
            TunnelState::Disconnected => Self::new_disconnected(),
            TunnelState::Connecting {
                retry_attempt,
                connection_data,
                ..
            } => {
                let maybe_exit_id = connection_data.as_ref().map(|d| d.exit_gateway.id.clone());
                let maybe_exit_cc = connection_data
                    .as_ref()
                    .and_then(|d| d.exit_gateway.country_code.clone());
                let maybe_tunnel_type = connection_data
                    .and_then(|d| d.tunnel)
                    .map(|d| d.tunnel_type().short_name().into());
                Self::new_connecting(
                    retry_attempt,
                    maybe_exit_id,
                    maybe_exit_cc,
                    maybe_tunnel_type,
                )
            }
            TunnelState::Connected { connection_data } => Self::new_connected(
                connection_data.exit_gateway.id,
                connection_data.exit_gateway.country_code,
                connection_data.tunnel.tunnel_type().short_name().into(),
            ),
            TunnelState::Disconnecting { after_disconnect } => {
                Self::new_disconnecting(after_disconnect)
            }
            TunnelState::Error(reason) => Self::new_error(reason),
            TunnelState::Offline { reconnect } => Self::new_offline(reconnect),
        }
    }

    pub(crate) fn sending_event_from_state(state: TunnelState) -> Self {
        match state {
            TunnelState::Disconnected => Self::ReportSending(ReportSendingEvent::Disconnected),
            TunnelState::Connected { .. } => Self::ReportSending(ReportSendingEvent::Connected),
            _ => Self::ReportSending(ReportSendingEvent::Standby),
        }
    }
}

#[derive(Debug, Clone)]
pub enum UsageEvent {
    // User action
    ConnectRequest(Instant),
    DisconnectRequest(Instant),

    // State machine state events
    Connecting {
        instant: Instant,
        retry_attempt: u32,
        maybe_exit_id: Option<String>,
        maybe_exit_cc: Option<String>,
        maybe_tunnel_type: Option<String>,
    },
    Connected {
        instant: Instant,
        exit_id: String,
        exit_cc: Option<String>,
        tunnel_type: String,
    },
    Disconnected(Instant),
    Disconnecting {
        instant: Instant,
        after_disconnect: ActionAfterDisconnect,
    },
    Error {
        instant: Instant,
        error: String,
    },
    Offline {
        instant: Instant,
        reconnect: bool,
    },
}

// Not a stat event per se, but used to instruct the report event to do stuff
#[derive(Debug, Clone, Copy)]
pub enum ReportSendingEvent {
    // Tunnel is connected, we're free to send
    Connected,
    // Tunnel is disconnected, send only if the config flag allows it
    Disconnected,
    // Tunnel is connecting/disconnecting/offline/error, don't try at all
    Standby,

    AllowDirectSending(bool),
}
