// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_vpn_lib_types::{
    AccountControllerState, BandwidthEvent, ConflictDetected, ConnectionEvent,
    ConnectionStatisticsEvent, DiagnosticsSuggestionReason, ErrorStateReason, MixnetEvent,
    SphinxPacketRates, TunnelEvent, TunnelState,
};

use crate::{conversions::ConversionError, proto};

impl TryFrom<proto::TunnelEvent> for TunnelEvent {
    type Error = ConversionError;

    fn try_from(value: proto::TunnelEvent) -> Result<Self, Self::Error> {
        let event = value
            .event
            .ok_or(ConversionError::NoValueSet("TunnelEvent.event"))?;

        Ok(match event {
            proto::tunnel_event::Event::TunnelState(tunnel_state) => {
                TunnelEvent::NewState(TunnelState::try_from(tunnel_state)?)
            }
            proto::tunnel_event::Event::MixnetEvent(mixnet_event) => {
                TunnelEvent::MixnetState(MixnetEvent::try_from(mixnet_event)?)
            }
            proto::tunnel_event::Event::ConfigChangedEvent(config_changed_event) => {
                let new_config = config_changed_event
                    .new_config
                    .ok_or(ConversionError::NoValueSet("ConfigChangedEvent.new_config"))?;
                TunnelEvent::ConfigChanged(Box::new(new_config.try_into()?))
            }
            proto::tunnel_event::Event::AccountState(account_state) => {
                TunnelEvent::AccountState(AccountControllerState::try_from(account_state)?)
            }
            proto::tunnel_event::Event::DiagnosticsSuggestedEvent(event) => {
                TunnelEvent::DiagnosticsSuggested(DiagnosticsSuggestionReason::try_from(event)?)
            }
            proto::tunnel_event::Event::ConflictDetectedEvent(event) => {
                TunnelEvent::ConflictDetected(ConflictDetected::try_from(event)?)
            }
        })
    }
}

impl From<TunnelEvent> for proto::TunnelEvent {
    fn from(value: TunnelEvent) -> Self {
        let event = match value {
            TunnelEvent::NewState(tunnel_state) => {
                proto::tunnel_event::Event::TunnelState(proto::TunnelState::from(tunnel_state))
            }
            TunnelEvent::MixnetState(mixnet_event) => {
                proto::tunnel_event::Event::MixnetEvent(proto::MixnetEvent::from(mixnet_event))
            }
            TunnelEvent::ConfigChanged(new_config) => {
                proto::tunnel_event::Event::ConfigChangedEvent(proto::ConfigChangedEvent {
                    new_config: Some(proto::VpnServiceConfig::from(*new_config)),
                })
            }
            TunnelEvent::AccountState(account_state) => proto::tunnel_event::Event::AccountState(
                proto::AccountControllerState::from(account_state),
            ),
            TunnelEvent::DiagnosticsSuggested(reason) => {
                proto::tunnel_event::Event::DiagnosticsSuggestedEvent(
                    proto::DiagnosticsSuggestedEvent::from(reason),
                )
            }
            TunnelEvent::ConflictDetected(conflict) => {
                proto::tunnel_event::Event::ConflictDetectedEvent(
                    proto::ConflictDetectedEvent::from(conflict),
                )
            }
        };
        Self { event: Some(event) }
    }
}

impl TryFrom<proto::ConflictDetectedEvent> for ConflictDetected {
    type Error = ConversionError;

    fn try_from(value: proto::ConflictDetectedEvent) -> Result<Self, Self::Error> {
        let conflict = proto::conflict_detected_event::Conflict::try_from(value.conflict)
            .map_err(|e| ConversionError::Decode("ConflictDetectedEvent.conflict", e))?;

        Ok(match conflict {
            proto::conflict_detected_event::Conflict::InterceptedDns => Self::InterceptedDns,
            proto::conflict_detected_event::Conflict::CompetingVpn => Self::CompetingVpn,
        })
    }
}

impl From<ConflictDetected> for proto::ConflictDetectedEvent {
    fn from(value: ConflictDetected) -> Self {
        let conflict = match value {
            ConflictDetected::InterceptedDns => {
                proto::conflict_detected_event::Conflict::InterceptedDns
            }
            ConflictDetected::CompetingVpn => {
                proto::conflict_detected_event::Conflict::CompetingVpn
            }
        };
        Self {
            conflict: conflict as i32,
        }
    }
}

impl TryFrom<proto::DiagnosticsSuggestedEvent> for DiagnosticsSuggestionReason {
    type Error = ConversionError;

    fn try_from(value: proto::DiagnosticsSuggestedEvent) -> Result<Self, Self::Error> {
        let reason = value.reason.ok_or(ConversionError::NoValueSet(
            "DiagnosticsSuggestedEvent.reason",
        ))?;

        Ok(match reason {
            proto::diagnostics_suggested_event::Reason::RepeatedConnectionRetries(retries) => {
                Self::RepeatedConnectionRetries {
                    attempts: retries.attempts,
                }
            }
            proto::diagnostics_suggested_event::Reason::AmbiguousError(error) => {
                Self::AmbiguousError(ErrorStateReason::try_from(error)?)
            }
        })
    }
}

impl From<DiagnosticsSuggestionReason> for proto::DiagnosticsSuggestedEvent {
    fn from(value: DiagnosticsSuggestionReason) -> Self {
        let reason = match value {
            DiagnosticsSuggestionReason::RepeatedConnectionRetries { attempts } => {
                proto::diagnostics_suggested_event::Reason::RepeatedConnectionRetries(
                    proto::diagnostics_suggested_event::RepeatedConnectionRetries { attempts },
                )
            }
            DiagnosticsSuggestionReason::AmbiguousError(reason) => {
                proto::diagnostics_suggested_event::Reason::AmbiguousError(
                    proto::tunnel_state::Error::from(reason),
                )
            }
        };
        Self {
            reason: Some(reason),
        }
    }
}

impl TryFrom<proto::MixnetEvent> for MixnetEvent {
    type Error = ConversionError;

    fn try_from(value: proto::MixnetEvent) -> Result<Self, Self::Error> {
        let event = value
            .event
            .ok_or(ConversionError::NoValueSet("MixnetEvent.event"))?;

        Ok(match event {
            proto::mixnet_event::Event::BandwidthEvent(bandwidth_event) => {
                Self::Bandwidth(BandwidthEvent::try_from(bandwidth_event)?)
            }
            proto::mixnet_event::Event::ConnectionEvent(connection_event) => {
                let proto_connection_event =
                    proto::mixnet_event::ConnectionEvent::try_from(connection_event)
                        .map_err(|e| ConversionError::Decode("ConnectionEvent", e))?;
                Self::Connection(ConnectionEvent::from(proto_connection_event))
            }
            proto::mixnet_event::Event::ConnectionStatisticsEvent(connection_statistics_event) => {
                Self::ConnectionStatistics(ConnectionStatisticsEvent::try_from(
                    connection_statistics_event,
                )?)
            }
        })
    }
}

impl From<MixnetEvent> for proto::MixnetEvent {
    fn from(value: MixnetEvent) -> Self {
        let event = match value {
            MixnetEvent::Bandwidth(e) => proto::mixnet_event::Event::BandwidthEvent(
                proto::mixnet_event::BandwidthEvent::from(e),
            ),
            MixnetEvent::Connection(e) => proto::mixnet_event::Event::ConnectionEvent(
                proto::mixnet_event::ConnectionEvent::from(e) as i32,
            ),
            MixnetEvent::ConnectionStatistics(e) => {
                proto::mixnet_event::Event::ConnectionStatisticsEvent(
                    proto::mixnet_event::ConnectionStatisticsEvent::from(e),
                )
            }
        };

        Self { event: Some(event) }
    }
}

impl From<proto::mixnet_event::ConnectionEvent> for ConnectionEvent {
    fn from(value: proto::mixnet_event::ConnectionEvent) -> Self {
        match value {
            proto::mixnet_event::ConnectionEvent::EntryGatewayDown => Self::EntryGatewayDown,
            proto::mixnet_event::ConnectionEvent::ExitGatewayDownIpv4 => Self::ExitGatewayDownIpv4,
            proto::mixnet_event::ConnectionEvent::ExitGatewayDownIpv6 => Self::ExitGatewayDownIpv6,
            proto::mixnet_event::ConnectionEvent::ExitGatewayRoutingErrorIpv4 => {
                Self::ExitGatewayRoutingErrorIpv4
            }
            proto::mixnet_event::ConnectionEvent::ExitGatewayRoutingErrorIpv6 => {
                Self::ExitGatewayRoutingErrorIpv6
            }
            proto::mixnet_event::ConnectionEvent::ConnectedIpv4 => Self::ConnectedIpv4,
            proto::mixnet_event::ConnectionEvent::ConnectedIpv6 => Self::ConnectedIpv6,
        }
    }
}

impl From<ConnectionEvent> for proto::mixnet_event::ConnectionEvent {
    fn from(value: ConnectionEvent) -> Self {
        match value {
            ConnectionEvent::EntryGatewayDown => Self::EntryGatewayDown,
            ConnectionEvent::ExitGatewayDownIpv4 => Self::ExitGatewayDownIpv4,
            ConnectionEvent::ExitGatewayDownIpv6 => Self::ExitGatewayDownIpv6,
            ConnectionEvent::ExitGatewayRoutingErrorIpv4 => Self::ExitGatewayRoutingErrorIpv4,
            ConnectionEvent::ExitGatewayRoutingErrorIpv6 => Self::ExitGatewayRoutingErrorIpv6,
            ConnectionEvent::ConnectedIpv4 => Self::ConnectedIpv4,
            ConnectionEvent::ConnectedIpv6 => Self::ConnectedIpv6,
        }
    }
}

impl TryFrom<proto::mixnet_event::BandwidthEvent> for BandwidthEvent {
    type Error = ConversionError;

    fn try_from(value: proto::mixnet_event::BandwidthEvent) -> Result<Self, Self::Error> {
        let event = value
            .event
            .ok_or(ConversionError::NoValueSet("BandwidthEvent.event"))?;

        Ok(match event {
            proto::mixnet_event::bandwidth_event::Event::NoBandwidth(
                proto::mixnet_event::bandwidth_event::NoBandwidth {},
            ) => Self::NoBandwidth,
            proto::mixnet_event::bandwidth_event::Event::RemainingBandwidth(
                proto::mixnet_event::bandwidth_event::RemainingBandwidth { value },
            ) => Self::RemainingBandwidth(value),
        })
    }
}

impl From<BandwidthEvent> for proto::mixnet_event::BandwidthEvent {
    fn from(value: BandwidthEvent) -> Self {
        let event = match value {
            BandwidthEvent::NoBandwidth => {
                proto::mixnet_event::bandwidth_event::Event::NoBandwidth(
                    proto::mixnet_event::bandwidth_event::NoBandwidth {},
                )
            }
            BandwidthEvent::RemainingBandwidth(value) => {
                proto::mixnet_event::bandwidth_event::Event::RemainingBandwidth(
                    proto::mixnet_event::bandwidth_event::RemainingBandwidth { value },
                )
            }
        };
        Self { event: Some(event) }
    }
}

impl TryFrom<proto::mixnet_event::ConnectionStatisticsEvent> for ConnectionStatisticsEvent {
    type Error = ConversionError;

    fn try_from(
        value: proto::mixnet_event::ConnectionStatisticsEvent,
    ) -> Result<Self, Self::Error> {
        let rates = value.rates.ok_or(ConversionError::NoValueSet(
            "ConnectionStatisticsEvent.rates",
        ))?;
        Ok(Self {
            rates: SphinxPacketRates::from(rates),
        })
    }
}

impl From<ConnectionStatisticsEvent> for proto::mixnet_event::ConnectionStatisticsEvent {
    fn from(value: ConnectionStatisticsEvent) -> Self {
        Self {
            rates: Some(proto::mixnet_event::SphinxPacketRates::from(value.rates)),
        }
    }
}

impl From<proto::mixnet_event::SphinxPacketRates> for SphinxPacketRates {
    fn from(value: proto::mixnet_event::SphinxPacketRates) -> Self {
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

impl From<SphinxPacketRates> for proto::mixnet_event::SphinxPacketRates {
    fn from(value: SphinxPacketRates) -> Self {
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
