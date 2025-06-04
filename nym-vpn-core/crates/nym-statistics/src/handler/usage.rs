// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::time::{Duration, Instant};

use nym_vpn_store::VpnStorage;

use crate::{events::UsageEvent, report::UsageReport, storage::StatsStorage};

pub(crate) struct UsageHandler<S>
where
    S: VpnStorage,
{
    _storage: StatsStorage<S>,
    //SW Potentially, this could hold a Watch<TunnelState> directly, to spare some event handling? No, because then that handler needs to operate independently and we don't want that
    connection_time: Option<Duration>,
    connecting: Option<Instant>,
    two_hop: bool,
}

impl<S> UsageHandler<S>
where
    S: VpnStorage,
{
    pub(crate) fn new(storage: StatsStorage<S>) -> Self {
        UsageHandler {
            _storage: storage,
            connection_time: None,
            connecting: None,
            two_hop: false,
        }
    }

    pub(crate) fn get_report(&mut self) -> UsageReport {
        UsageReport {
            connection_time_ms: self.connection_time.take().map(|d| d.as_millis()),
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
            }
            UsageEvent::Connected(instant) => {
                if let Some(connecting_time) = self.connecting {
                    self.connection_time = Some(instant.duration_since(connecting_time));
                }
            }
            _ => {
                self.connecting = None;
                self.connection_time = None;
            }
        }
    }
}
