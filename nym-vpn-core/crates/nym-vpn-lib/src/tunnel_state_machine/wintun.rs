use nym_windows::net as wnet;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use windows::Win32::NetworkManagement::Ndis::NET_LUID_LH;

/// Wintun adapter configuration error.
#[derive(Debug, thiserror::Error)]
pub enum SetupWintunAdapterError {
    #[error("failed to set wintun adapter ipv4 address: {}", _0)]
    SetIpv4Addr(#[source] nym_windows::net::Error),

    #[error("failed to set wintun adapter ipv6 address: {}", _0)]
    SetIpv6Addr(#[source] nym_windows::net::Error),

    #[error("failed to set wintun adapter ipv4 gateway address: {}", _0)]
    SetIpv4Gateway(#[source] nym_windows::net::Error),

    #[error("failed to set wintun adapter ipv6 gateway address: {}", _0)]
    SetIpv6Gateway(#[source] nym_windows::net::Error),
}

/// Struct holding wintun adapter IP configuration.
pub struct WintunAdapterConfig {
    /// Interface IPv4 address.
    pub interface_ipv4: Ipv4Addr,

    /// Interface IPv6 address.
    pub interface_ipv6: Ipv6Addr,

    /// Default IPv4 gateway.
    pub gateway_ipv4: Option<Ipv4Addr>,

    /// Default IPv6 gateway.
    pub gateway_ipv6: Option<Ipv6Addr>,
}

/// Configure wintun adapter
pub fn setup_wintun_adapter(
    luid: NET_LUID_LH,
    adapter_config: WintunAdapterConfig,
) -> Result<(), SetupWintunAdapterError> {
    wnet::add_ip_address_for_interface(luid, IpAddr::V4(adapter_config.interface_ipv4))
        .map_err(SetupWintunAdapterError::SetIpv4Addr)?;
    wnet::add_ip_address_for_interface(luid, IpAddr::V6(adapter_config.interface_ipv6))
        .map_err(SetupWintunAdapterError::SetIpv6Addr)?;

    if let Some(gateway_ipv4) = adapter_config.gateway_ipv4 {
        wnet::add_default_ipv4_gateway_for_interface(luid, gateway_ipv4)
            .map_err(SetupWintunAdapterError::SetIpv4Gateway)?;
    }

    if let Some(gateway_ipv6) = adapter_config.gateway_ipv6 {
        wnet::add_default_ipv6_gateway_for_interface(luid, gateway_ipv6)
            .map_err(SetupWintunAdapterError::SetIpv6Gateway)?;
    }

    Ok(())
}
