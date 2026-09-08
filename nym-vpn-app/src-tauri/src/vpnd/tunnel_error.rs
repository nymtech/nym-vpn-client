use nym_vpn_lib_types as lib;
use serde::Serialize;
use ts_rs::TS;

#[derive(Serialize, Clone, Debug, strum::Display, PartialEq, TS)]
#[ts(export, export_to = "tauri.ts")]
#[serde(rename_all = "kebab-case")]
pub enum TunnelError {
    Internal(String),
    SetFirewallPolicy,
    SetDns,
    SetRouting,
    SameEntryAndExitGw,
    PerformantEntryGwUnavailable,
    PerformantExitGwUnavailable,
    InvalidEntryGwId,
    InvalidExitGwId,
    InvalidEntryGwCountry,
    InvalidExitGwCountry,
    MaxDevicesReached,
    BandwidthExceeded,
    InactiveSubscription,
    DeviceTimeOutOfSync,
    DeviceLoggedOut,
    Ipv6Unavailable,
    TunDevice,
    TunnelProvider,
    InactiveAccount,
    CredentialWastedOnEntryGateway,
    CredentialWastedOnExitGateway,
    NeedFullDiskPermissions,
    SplitTunnel,
    NeedsRelaxedIndependenceCriteria,
    NeedsDeviceLocation,
    CredentialFetchingFailed,
    NoCredentialAvailable,
    ConnectionAttemptsExceeded,
}

impl From<lib::ErrorStateReason> for TunnelError {
    fn from(error: lib::ErrorStateReason) -> Self {
        match error {
            lib::ErrorStateReason::Internal(msg) => TunnelError::Internal(msg),
            lib::ErrorStateReason::SetFirewallPolicy => TunnelError::SetFirewallPolicy,
            lib::ErrorStateReason::SetDns => TunnelError::SetDns,
            lib::ErrorStateReason::SameEntryAndExitGateway => TunnelError::SameEntryAndExitGw,
            lib::ErrorStateReason::PerformantEntryGatewayUnavailable => {
                TunnelError::PerformantEntryGwUnavailable
            }
            lib::ErrorStateReason::PerformantExitGatewayUnavailable => {
                TunnelError::PerformantExitGwUnavailable
            }
            lib::ErrorStateReason::InvalidEntryGatewayIdentity => TunnelError::InvalidEntryGwId,
            lib::ErrorStateReason::InvalidExitGatewayIdentity => TunnelError::InvalidExitGwId,
            lib::ErrorStateReason::InvalidEntryGatewayCountry => TunnelError::InvalidEntryGwCountry,
            lib::ErrorStateReason::InvalidExitGatewayCountry => TunnelError::InvalidExitGwCountry,
            lib::ErrorStateReason::MaxDevicesReached => TunnelError::MaxDevicesReached,
            lib::ErrorStateReason::BandwidthExceeded => TunnelError::BandwidthExceeded,
            lib::ErrorStateReason::InactiveSubscription => TunnelError::InactiveSubscription,
            lib::ErrorStateReason::DeviceTimeOutOfSync => TunnelError::DeviceTimeOutOfSync,
            lib::ErrorStateReason::Ipv6Unavailable => TunnelError::Ipv6Unavailable,
            lib::ErrorStateReason::SetRouting => TunnelError::SetRouting,
            lib::ErrorStateReason::TunDevice => TunnelError::TunDevice,
            lib::ErrorStateReason::TunnelProvider => TunnelError::TunnelProvider,
            lib::ErrorStateReason::InactiveAccount => TunnelError::InactiveAccount,
            lib::ErrorStateReason::DeviceLoggedOut => TunnelError::DeviceLoggedOut,
            lib::ErrorStateReason::CredentialWastedOnEntryGateway => {
                TunnelError::CredentialWastedOnEntryGateway
            }
            lib::ErrorStateReason::CredentialWastedOnExitGateway => {
                TunnelError::CredentialWastedOnExitGateway
            }
            lib::ErrorStateReason::NeedFullDiskPermissions => TunnelError::NeedFullDiskPermissions,
            lib::ErrorStateReason::SplitTunnel => TunnelError::SplitTunnel,
            lib::ErrorStateReason::NeedsRelaxedIndependenceCriteria => {
                TunnelError::NeedsRelaxedIndependenceCriteria
            }
            lib::ErrorStateReason::NeedsDeviceLocation => TunnelError::NeedsDeviceLocation,
            lib::ErrorStateReason::CredentialFetchingFailed => {
                TunnelError::CredentialFetchingFailed
            }
            lib::ErrorStateReason::NoCredentialAvailable => TunnelError::NoCredentialAvailable,
            lib::ErrorStateReason::ConnectionAttemptsExceeded => {
                TunnelError::ConnectionAttemptsExceeded
            }
        }
    }
}
