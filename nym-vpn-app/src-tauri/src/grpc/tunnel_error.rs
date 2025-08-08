use nym_vpn_proto::proto as p;
use p::tunnel_state::{Error as ProtoTunnelError, ErrorStateReason};
use serde::Serialize;
use ts_rs::TS;

#[derive(Serialize, Clone, Debug, strum::Display, PartialEq, TS)]
#[ts(export)]
#[serde(rename_all = "kebab-case")]
#[serde(tag = "key", content = "data")]
pub enum TunnelError {
    Internal(Option<String>),
    Dns(Option<String>),
    Api(Option<String>),
    AccountController(Option<String>),
    Firewall(Option<String>),
    Routing(Option<String>),
    SameEntryAndExitGw(Option<String>),
    InvalidEntryGwCountry(Option<String>),
    InvalidExitGwCountry(Option<String>),
    MaxDevicesReached(Option<String>),
    BandwidthExceeded(Option<String>),
    InactiveSubscription(Option<String>),
    DeviceTimeOutOfSync(Option<String>),
    Ipv6Unavailable(Option<String>),
}

impl From<ProtoTunnelError> for TunnelError {
    fn from(error: ProtoTunnelError) -> Self {
        match error.reason() {
            ErrorStateReason::Internal => TunnelError::Internal(error.detail),
            ErrorStateReason::Firewall => TunnelError::Firewall(error.detail),
            ErrorStateReason::Routing => TunnelError::Routing(error.detail),
            ErrorStateReason::Dns => TunnelError::Dns(error.detail),
            ErrorStateReason::AccountControl => TunnelError::AccountController(error.detail),
            ErrorStateReason::SameEntryAndExitGateway => {
                TunnelError::SameEntryAndExitGw(error.detail)
            }
            ErrorStateReason::InvalidEntryGatewayCountry => {
                TunnelError::InvalidEntryGwCountry(error.detail)
            }
            ErrorStateReason::InvalidExitGatewayCountry => {
                TunnelError::InvalidExitGwCountry(error.detail)
            }
            ErrorStateReason::MaxDevicesReached => TunnelError::MaxDevicesReached(error.detail),
            ErrorStateReason::BandwidthExceeded => TunnelError::BandwidthExceeded(error.detail),
            ErrorStateReason::InactiveSubscription => {
                TunnelError::InactiveSubscription(error.detail)
            }
            ErrorStateReason::Api => TunnelError::Api(error.detail),
            ErrorStateReason::DeviceTimeOutOfSync => TunnelError::DeviceTimeOutOfSync(error.detail),
            ErrorStateReason::CreateMixnetStorage => TunnelError::Internal(error.detail),
            ErrorStateReason::Ipv6Unavailable => TunnelError::Ipv6Unavailable(error.detail),
        }
    }
}
