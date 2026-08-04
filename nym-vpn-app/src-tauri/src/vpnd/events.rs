use nym_vpn_lib_types as lib;
use serde::Serialize;
use tracing::instrument;
use ts_rs::TS;

use super::tunnel_error::TunnelError;

#[derive(Serialize, Clone, Debug, PartialEq, TS)]
#[ts(export, export_to = "tauri.ts")]
#[serde(rename_all = "kebab-case")]
pub enum DiagnosticsSuggestedReason {
    RepeatedConnectionRetries { attempts: u32 },
    AmbiguousError(TunnelError),
}

impl DiagnosticsSuggestedReason {
    #[instrument(skip(reason))]
    pub fn from_lib(reason: lib::DiagnosticsSuggestionReason) -> Self {
        match reason {
            lib::DiagnosticsSuggestionReason::RepeatedConnectionRetries { attempts } => {
                Self::RepeatedConnectionRetries { attempts }
            }
            lib::DiagnosticsSuggestionReason::AmbiguousError(reason) => {
                Self::AmbiguousError(TunnelError::from(reason))
            }
        }
    }
}

#[derive(Serialize, Clone, Debug, PartialEq, TS, strum::AsRefStr)]
#[ts(export, export_to = "tauri.ts")]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "snake_case")]
pub enum MixnetEvent {
    EntryGwDown,
    ExitGwDownIpv4,
    ExitGwDownIpv6,
    ExitGwRoutingErrorIpv4,
    ExitGwRoutingErrorIpv6,
    ConnectedIpv4,
    ConnectedIpv6,
    NoBandwidth,
    RemainingBandwidth(i64),
    SphinxPacketMetrics, // TODO include metrics
}

impl MixnetEvent {
    #[instrument(skip(event))]
    pub fn from_lib(event: lib::MixnetEvent) -> Self {
        match event {
            lib::MixnetEvent::Bandwidth(b) => MixnetEvent::from(b),
            lib::MixnetEvent::Connection(event) => MixnetEvent::from(event),
            lib::MixnetEvent::ConnectionStatistics(_) => MixnetEvent::SphinxPacketMetrics,
        }
    }
}

impl From<lib::BandwidthEvent> for MixnetEvent {
    fn from(event: lib::BandwidthEvent) -> Self {
        match event {
            lib::BandwidthEvent::NoBandwidth => Self::NoBandwidth,
            lib::BandwidthEvent::RemainingBandwidth(b) => Self::RemainingBandwidth(b),
        }
    }
}

impl From<lib::ConnectionEvent> for MixnetEvent {
    fn from(event: lib::ConnectionEvent) -> Self {
        match event {
            lib::ConnectionEvent::EntryGatewayDown => Self::EntryGwDown,
            lib::ConnectionEvent::ExitGatewayDownIpv4 => Self::ExitGwDownIpv4,
            lib::ConnectionEvent::ExitGatewayDownIpv6 => Self::ExitGwDownIpv6,
            lib::ConnectionEvent::ExitGatewayRoutingErrorIpv4 => Self::ExitGwRoutingErrorIpv4,
            lib::ConnectionEvent::ExitGatewayRoutingErrorIpv6 => Self::ExitGwRoutingErrorIpv6,
            lib::ConnectionEvent::ConnectedIpv4 => Self::ConnectedIpv4,
            lib::ConnectionEvent::ConnectedIpv6 => Self::ConnectedIpv6,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diagnostics_suggested_reason_from_lib_repeated_connection_retries() {
        let reason = DiagnosticsSuggestedReason::from_lib(
            lib::DiagnosticsSuggestionReason::RepeatedConnectionRetries { attempts: 3 },
        );

        assert_eq!(
            reason,
            DiagnosticsSuggestedReason::RepeatedConnectionRetries { attempts: 3 }
        );
    }

    #[test]
    fn diagnostics_suggested_reason_from_lib_ambiguous_error() {
        let reason = DiagnosticsSuggestedReason::from_lib(
            lib::DiagnosticsSuggestionReason::AmbiguousError(lib::ErrorStateReason::SetDns),
        );

        assert_eq!(
            reason,
            DiagnosticsSuggestedReason::AmbiguousError(TunnelError::SetDns)
        );
    }
}
