// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::fmt;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use si_scale::helpers::bibytes2;
#[cfg(feature = "typescript-bindings")]
use ts_rs::TS;

use crate::{AccountControllerState, ErrorStateReason, service::VpnServiceConfig};

#[cfg(feature = "nym-type-conversions")]
use nym_statistics_common::clients::packet_statistics::{
    MixnetBandwidthStatisticsEvent, PacketRates,
};

use super::tunnel_state::TunnelState;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Enum))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
#[cfg_attr(feature = "cbor", derive(minicbor::Encode, minicbor::Decode))]
pub enum TunnelEvent {
    #[cfg_attr(feature = "cbor", n(0))]
    NewState(#[cfg_attr(feature = "cbor", n(0))] TunnelState),
    #[cfg_attr(feature = "cbor", n(1))]
    MixnetState(#[cfg_attr(feature = "cbor", n(0))] MixnetEvent),
    #[cfg_attr(feature = "cbor", n(2))]
    ConfigChanged(#[cfg_attr(feature = "cbor", n(0))] Box<VpnServiceConfig>),
    #[cfg_attr(feature = "cbor", n(3))]
    AccountState(#[cfg_attr(feature = "cbor", n(0))] AccountControllerState),
    #[cfg_attr(feature = "cbor", n(4))]
    DiagnosticsSuggested(#[cfg_attr(feature = "cbor", n(0))] DiagnosticsSuggestionReason),
    #[cfg_attr(feature = "cbor", n(5))]
    ConflictDetected(#[cfg_attr(feature = "cbor", n(0))] ConflictDetected),
}

impl fmt::Display for TunnelEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NewState(new_state) => new_state.fmt(f),
            Self::MixnetState(event) => event.fmt(f),
            Self::ConfigChanged(config) => config.fmt(f),
            Self::AccountState(account_state) => account_state.fmt(f),
            Self::DiagnosticsSuggested(reason) => {
                write!(f, "Diagnostics suggested: {reason}")
            }
            Self::ConflictDetected(conflict) => {
                write!(f, "Conflict detected: {conflict}")
            }
        }
    }
}

/// A conflicting application was found on the system that may interfere with
/// NymVPN's own network filtering. Does not change `TunnelState` - the
/// tunnel connects and operates normally regardless.
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Enum))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
#[cfg_attr(feature = "cbor", derive(minicbor::Encode, minicbor::Decode))]
#[cfg_attr(feature = "cbor", cbor(index_only))]
pub enum ConflictDetected {
    #[cfg_attr(feature = "cbor", n(0))]
    InterceptedDns,
    #[cfg_attr(feature = "cbor", n(1))]
    CompetingVpn,
    #[cfg_attr(feature = "cbor", n(2))]
    CompetingFirewall,
}

impl fmt::Display for ConflictDetected {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InterceptedDns => write!(
                f,
                "Something on this system appears to be intercepting DNS queries, which may interfere with connectivity"
            ),
            Self::CompetingVpn => write!(
                f,
                "Another VPN client's tunnel appears to be active alongside NymVPN's, which may interfere with connectivity"
            ),
            Self::CompetingFirewall => write!(
                f,
                "Another application's firewall rules appear to be positioned to intercept NymVPN's own network traffic, which may interfere with connectivity"
            ),
        }
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Enum))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
#[cfg_attr(feature = "cbor", derive(minicbor::Encode, minicbor::Decode))]
pub enum DiagnosticsSuggestionReason {
    /// Stuck retrying to connect without ever reaching `Connected`.
    #[cfg_attr(feature = "cbor", n(0))]
    RepeatedConnectionRetries {
        #[cfg_attr(feature = "cbor", n(0))]
        attempts: u32,
    },

    /// Landed in an error state whose cause is ambiguous or network/reachability shaped,
    /// as opposed to account/billing/permission states that already carry their own
    /// obvious remediation.
    #[cfg_attr(feature = "cbor", n(1))]
    AmbiguousError(#[cfg_attr(feature = "cbor", n(0))] ErrorStateReason),
}

impl fmt::Display for DiagnosticsSuggestionReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RepeatedConnectionRetries { attempts } => {
                write!(f, "repeated connection retries ({attempts})")
            }
            Self::AmbiguousError(reason) => write!(f, "ambiguous error ({reason})"),
        }
    }
}

#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Enum))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
#[cfg_attr(feature = "cbor", derive(minicbor::Encode, minicbor::Decode))]
pub enum MixnetEvent {
    #[cfg_attr(feature = "cbor", n(0))]
    Bandwidth(#[cfg_attr(feature = "cbor", n(0))] BandwidthEvent),
    #[cfg_attr(feature = "cbor", n(1))]
    Connection(#[cfg_attr(feature = "cbor", n(0))] ConnectionEvent),
    #[cfg_attr(feature = "cbor", n(2))]
    ConnectionStatistics(#[cfg_attr(feature = "cbor", n(0))] ConnectionStatisticsEvent),
}

impl fmt::Display for MixnetEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bandwidth(event) => write!(f, "{event}"),
            Self::Connection(event) => write!(f, "{event}"),
            Self::ConnectionStatistics(event) => write!(f, "{event}"),
        }
    }
}

#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Enum))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
#[cfg_attr(feature = "cbor", derive(minicbor::Encode, minicbor::Decode))]
pub enum BandwidthEvent {
    #[cfg_attr(feature = "cbor", n(0))]
    NoBandwidth,
    #[cfg_attr(feature = "cbor", n(1))]
    RemainingBandwidth(#[cfg_attr(feature = "cbor", n(0))] i64),
}

impl fmt::Display for BandwidthEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoBandwidth => f.write_str("No bandwidth"),
            Self::RemainingBandwidth(value) => {
                write!(
                    f,
                    "Remaining bandwidth: {}",
                    si_scale::helpers::bibytes2(*value)
                )
            }
        }
    }
}

#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Enum))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
#[cfg_attr(feature = "cbor", derive(minicbor::Encode, minicbor::Decode))]
#[cfg_attr(feature = "cbor", cbor(index_only))]
pub enum ConnectionEvent {
    #[cfg_attr(feature = "cbor", n(0))]
    EntryGatewayDown,
    #[cfg_attr(feature = "cbor", n(1))]
    ExitGatewayDownIpv4,
    #[cfg_attr(feature = "cbor", n(2))]
    ExitGatewayDownIpv6,
    #[cfg_attr(feature = "cbor", n(3))]
    ExitGatewayRoutingErrorIpv4,
    #[cfg_attr(feature = "cbor", n(4))]
    ExitGatewayRoutingErrorIpv6,
    #[cfg_attr(feature = "cbor", n(5))]
    ConnectedIpv4,
    #[cfg_attr(feature = "cbor", n(6))]
    ConnectedIpv6,
}

impl fmt::Display for ConnectionEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::ConnectedIpv4 => "Connected with IPv4",
            Self::ConnectedIpv6 => "Connected with IPv6",
            Self::EntryGatewayDown => {
                "Entry gateway appears down - it's not routing our mixnet traffic"
            }
            Self::ExitGatewayDownIpv4 => {
                "Exit gateway (or ipr) appears down - it's not responding to IPv4 traffic"
            }
            Self::ExitGatewayDownIpv6 => {
                "Exit gateway (or ipr) appears down - it's not responding to IPv6 traffic"
            }
            Self::ExitGatewayRoutingErrorIpv4 => {
                "Exit gateway (or ipr) appears to be having issues routing and forwarding our external IPv4 traffic"
            }
            Self::ExitGatewayRoutingErrorIpv6 => {
                "Exit gateway (or ipr) appears to be having issues routing and forwarding our external IPv6 traffic"
            }
        };

        f.write_str(s)
    }
}

#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Record))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
#[cfg_attr(feature = "cbor", derive(minicbor::Encode, minicbor::Decode))]
pub struct ConnectionStatisticsEvent {
    #[cfg_attr(feature = "cbor", n(0))]
    pub rates: SphinxPacketRates,
}

impl fmt::Display for ConnectionStatisticsEvent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.rates)
    }
}

#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "uniffi-bindings", derive(uniffi::Record))]
#[cfg_attr(
    feature = "typescript-bindings",
    derive(TS),
    ts(export),
    ts(export_to = "bindings.ts")
)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "typescript-bindings", serde(rename_all = "camelCase"))]
#[cfg_attr(feature = "cbor", derive(minicbor::Encode, minicbor::Decode))]
pub struct SphinxPacketRates {
    #[cfg_attr(feature = "cbor", n(0))]
    pub real_packets_sent: f64,
    #[cfg_attr(feature = "cbor", n(1))]
    pub real_packets_sent_size: f64,
    #[cfg_attr(feature = "cbor", n(2))]
    pub cover_packets_sent: f64,
    #[cfg_attr(feature = "cbor", n(3))]
    pub cover_packets_sent_size: f64,

    #[cfg_attr(feature = "cbor", n(4))]
    pub real_packets_received: f64,
    #[cfg_attr(feature = "cbor", n(5))]
    pub real_packets_received_size: f64,
    #[cfg_attr(feature = "cbor", n(6))]
    pub cover_packets_received: f64,
    #[cfg_attr(feature = "cbor", n(7))]
    pub cover_packets_received_size: f64,

    #[cfg_attr(feature = "cbor", n(8))]
    pub total_acks_received: f64,
    #[cfg_attr(feature = "cbor", n(9))]
    pub total_acks_received_size: f64,
    #[cfg_attr(feature = "cbor", n(10))]
    pub real_acks_received: f64,
    #[cfg_attr(feature = "cbor", n(11))]
    pub real_acks_received_size: f64,
    #[cfg_attr(feature = "cbor", n(12))]
    pub cover_acks_received: f64,
    #[cfg_attr(feature = "cbor", n(13))]
    pub cover_acks_received_size: f64,

    #[cfg_attr(feature = "cbor", n(14))]
    pub real_packets_queued: f64,
    #[cfg_attr(feature = "cbor", n(15))]
    pub retransmissions_queued: f64,
    #[cfg_attr(feature = "cbor", n(16))]
    pub reply_surbs_queued: f64,
    #[cfg_attr(feature = "cbor", n(17))]
    pub additional_reply_surbs_queued: f64,
}

impl fmt::Display for SphinxPacketRates {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.summary())
    }
}

impl SphinxPacketRates {
    pub fn summary(&self) -> String {
        format!(
            "down: {}/s, up: {}/s (cover down: {}/s, cover up: {}/s)",
            bibytes2(self.real_packets_received_size),
            bibytes2(self.real_packets_sent_size),
            bibytes2(self.cover_packets_received_size),
            bibytes2(self.cover_packets_sent_size),
        )
    }

    pub fn real_received(&self) -> String {
        bibytes2(self.real_packets_received_size)
    }

    pub fn real_sent(&self) -> String {
        bibytes2(self.real_packets_sent_size)
    }

    pub fn cover_received(&self) -> String {
        bibytes2(self.cover_packets_received_size)
    }

    pub fn cover_sent(&self) -> String {
        bibytes2(self.cover_packets_sent_size)
    }
}

#[cfg(feature = "nym-type-conversions")]
impl From<&MixnetBandwidthStatisticsEvent> for ConnectionStatisticsEvent {
    fn from(value: &MixnetBandwidthStatisticsEvent) -> Self {
        Self {
            rates: SphinxPacketRates::from(value.rates.clone()),
        }
    }
}

#[cfg(feature = "nym-type-conversions")]
impl From<PacketRates> for SphinxPacketRates {
    fn from(value: PacketRates) -> Self {
        Self {
            real_packets_sent: value.real_packets_sent,
            real_packets_sent_size: value.real_packets_sent_size,
            cover_packets_sent: value.cover_packets_sent,
            cover_packets_sent_size: value.cover_packets_sent_size,
            real_packets_received: value.real_packets_received,
            real_packets_received_size: value.real_packets_received_size,
            cover_packets_received: value.cover_packets_received,
            cover_packets_received_size: value.cover_packets_received_size,
            total_acks_received: value.total_acks_received,
            total_acks_received_size: value.total_acks_received_size,
            real_acks_received: value.real_acks_received,
            real_acks_received_size: value.real_acks_received_size,
            cover_acks_received: value.cover_acks_received,
            cover_acks_received_size: value.cover_acks_received_size,
            real_packets_queued: value.real_packets_queued,
            retransmissions_queued: value.retransmissions_queued,
            reply_surbs_queued: value.reply_surbs_queued,
            additional_reply_surbs_queued: value.additional_reply_surbs_queued,
        }
    }
}
