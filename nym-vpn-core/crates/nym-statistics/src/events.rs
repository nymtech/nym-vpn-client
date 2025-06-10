// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::time::Instant;

use tokio::sync::mpsc::UnboundedSender;
use tokio_util::sync::CancellationToken;

/// Channel receiving generic stats events to be used by a statistics aggregator.
pub type StatisticsReceiver = tokio::sync::mpsc::UnboundedReceiver<StatisticsEvent>;

/// Channel allowing generic statistics events to be reported to a stats event aggregator
#[derive(Clone)]
pub struct StatisticsSender {
    stats_tx: Option<UnboundedSender<StatisticsEvent>>,
    cancel_token: CancellationToken,
}

impl StatisticsSender {
    /// Create a new statistics Sender
    pub fn new(
        stats_tx: Option<UnboundedSender<StatisticsEvent>>,
        cancel_token: CancellationToken,
    ) -> Self {
        StatisticsSender {
            stats_tx,
            cancel_token,
        }
    }

    /// Report a statistics event using the sender.
    pub fn report(&self, event: StatisticsEvent) {
        if let Some(tx) = &self.stats_tx {
            if let Err(err) = tx.send(event) {
                if !self.cancel_token.is_cancelled() {
                    tracing::error!("Failed to send stats event: {err}");
                }
            }
        }
    }
}

/// Statistics events
#[derive(Debug, Clone)]
pub enum StatisticsEvent {
    Usage(UsageEvent),
    Controller(ControllerEvent),
}

impl StatisticsEvent {
    pub fn new_connecting(enable_two_hop: bool) -> Self {
        Self::Usage(UsageEvent::Connecting {
            instant: Instant::now(),
            enable_two_hop,
        })
    }

    pub fn new_connected() -> Self {
        Self::Usage(UsageEvent::Connected(Instant::now()))
    }

    pub fn new_disconnecting() -> Self {
        Self::Usage(UsageEvent::Disconnecting(Instant::now()))
    }

    pub fn new_disconnected() -> Self {
        Self::Usage(UsageEvent::Disconnected(Instant::now()))
    }

    pub fn new_error(error: impl ToString) -> Self {
        Self::Usage(UsageEvent::Error {
            instant: Instant::now(),
            error: error.to_string(),
        })
    }

    pub fn reset_seed() -> Self {
        Self::Controller(ControllerEvent::ResetSeed)
    }

    pub fn remove_seed() -> Self {
        Self::Controller(ControllerEvent::RemoveSeed)
    }
}

#[derive(Debug, Clone)]
pub enum UsageEvent {
    Connecting {
        instant: Instant,
        enable_two_hop: bool,
    },
    Connected(Instant),
    Disconnected(Instant),
    Error {
        instant: Instant,
        error: String,
    },
    Disconnecting(Instant),
}

// Not a stat event per se, but used to instruct the controller to do stuff
#[derive(Debug, Clone)]
pub enum ControllerEvent {
    ResetSeed,
    RemoveSeed,
}
