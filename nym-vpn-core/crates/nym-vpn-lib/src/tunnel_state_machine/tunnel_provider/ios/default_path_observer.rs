// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: Apache-2.0

use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

/// The type alias for the receiving end of path observer.
pub type DefaultPathReceiver = UnboundedReceiver<OSDefaultPath>;

/// Observer type that wraps network path changes into a channel.
#[derive(Debug)]
pub struct DefaultPathObserver {
    tx: UnboundedSender<OSDefaultPath>,
}

impl DefaultPathObserver {
    pub fn new(tx: UnboundedSender<OSDefaultPath>) -> Self {
        Self { tx }
    }
}

impl OSDefaultPathObserver for DefaultPathObserver {
    fn on_default_path_change(&self, new_path: OSDefaultPath) {
        if self.tx.send(new_path).is_err() {
            tracing::warn!("Failed to send default path change.");
        }
    }
}

#[derive(uniffi::Enum, Debug)]
pub enum OSPathStatus {
    /// The path cannot be evaluated.
    Invalid,

    /// The path is ready to be used for network connections.
    Satisfied,

    /// The path for network connections is not available, either due to lack of network
    /// connectivity or being prohibited by system policy.
    Unsatisfied,

    /// The path is not currently satisfied, but may become satisfied upon a connection attempt.
    /// This can be due to a service, such as a VPN or a cellular data connection not being activated.
    Satisfiable,

    /// Unknown path status was received.
    /// The raw variant code is contained in associated value.
    Unknown(i64),
}

/// Represents a default network route used by the system.
#[derive(uniffi::Record, Debug)]
pub struct OSDefaultPath {
    /// Indicates whether the process is able to make connection through the given path.
    pub status: OSPathStatus,

    /// Set to true for interfaces that are considered expensive, such as when using cellular data plan.
    pub is_expensive: bool,

    /// Set to true when using a constrained interface, such as when using low-data mode.
    pub is_constrained: bool,
}

/// Types observing network changes.
#[uniffi::export(with_foreign)]
pub trait OSDefaultPathObserver: Send + Sync + std::fmt::Debug {
    fn on_default_path_change(&self, new_path: OSDefaultPath);
}
