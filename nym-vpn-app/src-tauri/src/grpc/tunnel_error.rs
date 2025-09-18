use nym_vpn_proto::proto as p;
use p::tunnel_state::{Error as ProtoTunnelError, ErrorStateReason};
use serde::Serialize;
use ts_rs::TS;

#[derive(Serialize, Clone, Debug, strum::Display, PartialEq, TS)]
#[ts(export)]
#[serde(rename_all = "kebab-case")]
#[serde(tag = "key", content = "message")]
pub enum TunnelError {
    Internal(Option<String>),
    SetFirewallPolicy(Option<String>),
    SetDns(Option<String>),
    SetRouting(Option<String>),
    SameEntryAndExitGw(Option<String>),
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
