use nym_vpn_lib_types as lib;
use nym_vpn_proto::proto as p;
use p::tunnel_state::{Error as ProtoTunnelError, ErrorStateReason};
use serde::Serialize;
use ts_rs::TS;

// TODO refactor the variants, only Internal needs to have a String
#[derive(Serialize, Clone, Debug, strum::Display, PartialEq, TS)]
#[ts(export, export_to = "tauri.ts")]
#[serde(rename_all = "kebab-case")]
#[serde(tag = "key", content = "message")]
pub enum TunnelError {
    Internal(Option<String>),
    SetFirewallPolicy(Option<String>),
    SetDns(Option<String>),
    SetRouting(Option<String>),
    SameEntryAndExitGw(Option<String>),
    PerformantEntryGwUnavailable(Option<String>),
    PerformantExitGwUnavailable(Option<String>),
    InvalidEntryGwId(Option<String>),
    InvalidExitGwId(Option<String>),
    InvalidEntryGwCountry(Option<String>),
    InvalidExitGwCountry(Option<String>),
    MaxDevicesReached(Option<String>),
    BandwidthExceeded(Option<String>),
    InactiveSubscription(Option<String>),
    DeviceTimeOutOfSync(Option<String>),
    Ipv6Unavailable(Option<String>),
    TunDevice(Option<String>),
    TunnelProvider(Option<String>),
    InactiveAccount(Option<String>),
    DeviceLoggedOut(Option<String>),
    CredentialWastedOnEntryGateway(Option<String>),
    CredentialWastedOnExitGateway(Option<String>),
}

impl From<ProtoTunnelError> for TunnelError {
    fn from(error: ProtoTunnelError) -> Self {
        match error.reason() {
            ErrorStateReason::Internal => TunnelError::Internal(error.message),
            ErrorStateReason::SetFirewallPolicy => TunnelError::SetFirewallPolicy(error.message),
            ErrorStateReason::SetDns => TunnelError::SetDns(error.message),
            ErrorStateReason::SameEntryAndExitGateway => {
                TunnelError::SameEntryAndExitGw(error.message)
            }
            ErrorStateReason::PerformantEntryGatewayUnavailable => {
                TunnelError::PerformantEntryGwUnavailable(error.message)
            }
            ErrorStateReason::PerformantExitGatewayUnavailable => {
                TunnelError::PerformantExitGwUnavailable(error.message)
            }
            ErrorStateReason::InvalidEntryGatewayIdentity => {
                TunnelError::InvalidEntryGwId(error.message)
            }
            ErrorStateReason::InvalidExitGatewayIdentity => {
                TunnelError::InvalidExitGwId(error.message)
            }
            ErrorStateReason::InvalidEntryGatewayCountry => {
                TunnelError::InvalidEntryGwCountry(error.message)
            }
            ErrorStateReason::InvalidExitGatewayCountry => {
                TunnelError::InvalidExitGwCountry(error.message)
            }
            ErrorStateReason::MaxDevicesReached => TunnelError::MaxDevicesReached(error.message),
            ErrorStateReason::BandwidthExceeded => TunnelError::BandwidthExceeded(error.message),
            ErrorStateReason::InactiveSubscription => {
                TunnelError::InactiveSubscription(error.message)
            }
            ErrorStateReason::DeviceTimeOutOfSync => {
                TunnelError::DeviceTimeOutOfSync(error.message)
            }
            ErrorStateReason::Ipv6Unavailable => TunnelError::Ipv6Unavailable(error.message),
            ErrorStateReason::SetRouting => TunnelError::SetRouting(error.message),
            ErrorStateReason::TunDevice => TunnelError::TunDevice(error.message),
            ErrorStateReason::TunnelProvider => TunnelError::TunnelProvider(error.message),
            ErrorStateReason::InactiveAccount => TunnelError::InactiveAccount(error.message),
            ErrorStateReason::DeviceLoggedOut => TunnelError::DeviceLoggedOut(error.message),
            ErrorStateReason::CredentialWastedOnEntryGateway => {
                TunnelError::CredentialWastedOnEntryGateway(error.message)
            }
            ErrorStateReason::CredentialWastedOnExitGateway => {
                TunnelError::CredentialWastedOnExitGateway(error.message)
            }
        }
    }
}

impl From<lib::ErrorStateReason> for TunnelError {
    fn from(error: lib::ErrorStateReason) -> Self {
        match error {
            lib::ErrorStateReason::Internal(msg) => TunnelError::Internal(Some(msg)),
            lib::ErrorStateReason::SetFirewallPolicy => TunnelError::SetFirewallPolicy(None),
            lib::ErrorStateReason::SetDns => TunnelError::SetDns(None),
            lib::ErrorStateReason::SameEntryAndExitGateway => TunnelError::SameEntryAndExitGw(None),
            lib::ErrorStateReason::PerformantEntryGatewayUnavailable => {
                TunnelError::PerformantEntryGwUnavailable(None)
            }
            lib::ErrorStateReason::PerformantExitGatewayUnavailable => {
                TunnelError::PerformantExitGwUnavailable(None)
            }
            lib::ErrorStateReason::InvalidEntryGatewayIdentity => {
                TunnelError::InvalidEntryGwId(None)
            }
            lib::ErrorStateReason::InvalidExitGatewayIdentity => TunnelError::InvalidExitGwId(None),
            lib::ErrorStateReason::InvalidEntryGatewayCountry => {
                TunnelError::InvalidEntryGwCountry(None)
            }
            lib::ErrorStateReason::InvalidExitGatewayCountry => {
                TunnelError::InvalidExitGwCountry(None)
            }
            lib::ErrorStateReason::MaxDevicesReached => TunnelError::MaxDevicesReached(None),
            lib::ErrorStateReason::BandwidthExceeded => TunnelError::BandwidthExceeded(None),
            lib::ErrorStateReason::InactiveSubscription => TunnelError::InactiveSubscription(None),
            lib::ErrorStateReason::DeviceTimeOutOfSync => TunnelError::DeviceTimeOutOfSync(None),
            lib::ErrorStateReason::Ipv6Unavailable => TunnelError::Ipv6Unavailable(None),
            lib::ErrorStateReason::SetRouting => TunnelError::SetRouting(None),
            lib::ErrorStateReason::TunDevice => TunnelError::TunDevice(None),
            lib::ErrorStateReason::TunnelProvider => TunnelError::TunnelProvider(None),
            lib::ErrorStateReason::InactiveAccount => TunnelError::InactiveAccount(None),
            lib::ErrorStateReason::DeviceLoggedOut => TunnelError::DeviceLoggedOut(None),
            lib::ErrorStateReason::CredentialWastedOnEntryGateway => {
                TunnelError::CredentialWastedOnEntryGateway(None)
            }
            lib::ErrorStateReason::CredentialWastedOnExitGateway => {
                TunnelError::CredentialWastedOnExitGateway(None)
            }
        }
    }
}
