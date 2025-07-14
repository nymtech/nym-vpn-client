// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::time::{Duration, Instant};

use nym_statistics_common::report::vpn_client::UsageReport;

use crate::{events::UsageEvent, storage::StatsStorage};

pub(crate) struct UsageHandler {
    _storage: StatsStorage,
    connection_time: Option<Duration>,
    connecting: Option<Instant>,
    two_hop: bool,
    pub(crate) is_connected: bool,
}

impl UsageHandler {
    pub(crate) fn new(storage: StatsStorage) -> Self {
        UsageHandler {
            _storage: storage,
            connection_time: None,
            connecting: None,
            two_hop: false,
            is_connected: false,
        }
    }

    pub(crate) fn get_report(&mut self) -> UsageReport {
        UsageReport {
            connection_time_ms: self
                .connection_time
                .take()
                .map(|d| d.as_millis().try_into().unwrap_or(i32::MAX)), //SW We should never hit the or clause, but it's better than to panic (it's ~24 days)
            two_hop: self.two_hop,
        }
    }

    pub(crate) fn handle_event(&mut self, event: UsageEvent) {
        match event {
            UsageEvent::Connecting {
                instant,
                enable_two_hop,
            } => {
                self.connecting = Some(instant);
                self.connection_time = None;
                self.two_hop = enable_two_hop;
                self.is_connected = false;
            }
            UsageEvent::Connected(instant) => {
                if let Some(connecting_time) = self.connecting {
                    self.connection_time = Some(instant.duration_since(connecting_time));
                }
                self.is_connected = true;
            }
            UsageEvent::Disconnecting(_) => {
                // If this branch doesn't trigger, it means we were connected and `connect` was called again, so we must ignore that, otherwise we will overwrite stuff
                if self.connecting.is_none() || self.is_connected {
                    self.connecting = None;
                    self.connection_time = None;
                    self.is_connected = false;
                }
            }
            _ => {
                self.connecting = None;
                self.connection_time = None;
                self.is_connected = false;
            }
        }
    }
}
