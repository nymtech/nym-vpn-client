// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{error::Error, fmt};
use time::{Duration, OffsetDateTime};

const MAX_ACCEPTABLE_SKEW_SECONDS: i64 = 60;
const SKEW_SECONDS_CONSIDERED_SAME: i64 = 2;

pub type AccountSignerError = Box<dyn Error + Send + Sync>;

/// Account credentials used to authenticate requests to the VPN API.
pub trait AccountSigner: Send + Sync {
    /// Cosmos account identifier of the account.
    fn id(&self) -> String;

    /// Base58-encoded secp256k1 public key of the account.
    fn pub_key(&self) -> &str;

    /// Signs the message with the account's secp256k1 private key.
    fn sign_raw(&self, message: &[u8]) -> Result<Vec<u8>, AccountSignerError>;
}

/// Defines the mode of operation of the associated account.
#[derive(Debug, Copy, Clone, strum_macros::Display, PartialEq, Eq)]
#[strum(serialize_all = "snake_case")]
pub enum VpnAccountMode {
    /// Account works in the API mode, i.e. the subscription is managed
    /// by the VPN API which provides required ticketbooks
    Api,

    /// Account works in the decentralised mode, i.e. there is no associated subscription
    /// and the account uses its own funds for obtaining required ticketbooks
    // add an alias for our US friends
    Decentralised,

    /// Account works in the API mode, but the mnemonic is derived from the Privy
    /// wallet private key.
    Privy,
}

impl VpnAccountMode {
    pub fn is_api(&self) -> bool {
        matches!(self, Self::Api)
    }

    pub fn is_decentralised(&self) -> bool {
        matches!(self, Self::Decentralised)
    }

    pub fn is_privy(&self) -> bool {
        matches!(self, Self::Privy)
    }
}

#[derive(Clone, Copy, Debug)]
pub struct VpnApiTime {
    // The local time on the client.
    pub local_time: OffsetDateTime,

    // The estimated time on the remote server. Based on RTT, it's not guaranteed to be accurate.
    pub estimated_remote_time: OffsetDateTime,
}

impl VpnApiTime {
    pub fn from_estimated_remote_time(
        local_time: OffsetDateTime,
        estimated_remote_time: OffsetDateTime,
    ) -> Self {
        Self {
            local_time,
            estimated_remote_time,
        }
    }

    pub fn from_remote_timestamp(
        local_time_before_request: OffsetDateTime,
        remote_timestamp: OffsetDateTime,
        local_time_after_request: OffsetDateTime,
    ) -> Self {
        let rtt = local_time_after_request - local_time_before_request;
        let estimated_remote_time = remote_timestamp + (rtt / 2);
        Self {
            local_time: local_time_after_request,
            estimated_remote_time,
        }
    }

    // Local time minus remote time. Meaning if the value is positive, the local time is ahead
    // of the remote time.
    pub fn local_time_ahead_skew(&self) -> Duration {
        self.local_time - self.estimated_remote_time
    }

    pub fn is_almost_same(&self) -> bool {
        self.local_time_ahead_skew().abs().whole_seconds() < SKEW_SECONDS_CONSIDERED_SAME
    }

    pub fn is_acceptable_synced(&self) -> bool {
        self.local_time_ahead_skew().abs().whole_seconds() < MAX_ACCEPTABLE_SKEW_SECONDS
    }

    pub fn is_synced(&self) -> VpnApiTimeSynced {
        if self.is_almost_same() {
            VpnApiTimeSynced::AlmostSame
        } else if self.is_acceptable_synced() {
            VpnApiTimeSynced::AcceptableSynced
        } else {
            VpnApiTimeSynced::NotSynced
        }
    }

    pub fn estimate_remote_now(&self) -> OffsetDateTime {
        tracing::debug!(
            "Estimating remote now using (local time ahead) skew: {}",
            self.local_time_ahead_skew()
        );
        let local_time_now = OffsetDateTime::now_utc();
        local_time_now - self.local_time_ahead_skew()
    }

    pub fn estimate_remote_now_unix(&self) -> u128 {
        self.estimate_remote_now().unix_timestamp() as u128
    }
}

impl fmt::Display for VpnApiTime {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Local time: {}, Remote time: {}, Skew: {}",
            self.local_time,
            self.estimated_remote_time,
            self.local_time_ahead_skew(),
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VpnApiTimeSynced {
    AlmostSame,
    AcceptableSynced,
    NotSynced,
}

impl VpnApiTimeSynced {
    pub fn is_synced(&self) -> bool {
        matches!(
            self,
            VpnApiTimeSynced::AlmostSame | VpnApiTimeSynced::AcceptableSynced
        )
    }

    pub fn is_not_synced(&self) -> bool {
        !self.is_synced()
    }
}
