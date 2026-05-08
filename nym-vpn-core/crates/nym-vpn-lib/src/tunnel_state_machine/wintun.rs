// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use nym_windows::net::{self as wnet, AddressFamily};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use windows::Win32::{
    Foundation::ERROR_NOT_FOUND, NetworkManagement::Ndis::NET_LUID_LH,
    Networking::WinSock::RouterDiscoveryDisabled,
};

/// Wintun adapter configuration error.
#[derive(Debug, thiserror::Error)]
pub enum SetupWintunAdapterError {
    #[error("failed to set wintun adapter ipv4 address")]
    SetIpv4Addr(#[source] nym_windows::net::Error),

    #[error("failed to set wintun adapter ipv6 address")]
    SetIpv6Addr(#[source] nym_windows::net::Error),

    #[error("failed to set wintun adapter ipv4 gateway address")]
    SetIpv4Gateway(#[source] nym_windows::net::Error),

    #[error("failed to set wintun adapter ipv6 gateway address")]
    SetIpv6Gateway(#[source] nym_windows::net::Error),

    #[error("failed to set interface mtu")]
    SetMtu(#[source] windows::core::Error),

    #[error("failed to set ip interface entry")]
    SetIpInterfaceEntry(#[source] windows::core::Error),

    #[error("failed to wait for ip interfaces to attach on network interface")]
    WaitForInterfaces(#[source] std::io::Error),

    #[error("failed to wait for addresses to be usable on an network adapter")]
    WaitForInterfaceAddresses(#[source] nym_windows::net::Error),
}

/// Struct holding wintun adapter IP configuration.
pub struct WintunAdapterConfig {
    /// Interface IPv4 address.
    pub interface_ipv4: Ipv4Addr,

    /// Interface IPv6 address.
    pub interface_ipv6: Option<Ipv6Addr>,

    /// Default IPv4 gateway.
    pub gateway_ipv4: Option<Ipv4Addr>,

    /// Default IPv6 gateway.
    pub gateway_ipv6: Option<Ipv6Addr>,
}

pub type Result<T, E = SetupWintunAdapterError> = std::result::Result<T, E>;

/// Configure wintun adapter
pub fn setup_wintun_adapter(luid: NET_LUID_LH, adapter_config: WintunAdapterConfig) -> Result<()> {
    wnet::add_ip_address_for_interface(luid, IpAddr::V4(adapter_config.interface_ipv4))
        .map_err(SetupWintunAdapterError::SetIpv4Addr)?;

    if let Some(interface_ipv6) = adapter_config.interface_ipv6 {
        wnet::add_ip_address_for_interface(luid, IpAddr::V6(interface_ipv6))
            .map_err(SetupWintunAdapterError::SetIpv6Addr)?;
    }

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

/// Set IPv6 address only on network interface
pub fn add_ipv6_address(luid: NET_LUID_LH, interface_ipv6: Ipv6Addr) -> Result<()> {
    wnet::add_ip_address_for_interface(luid, IpAddr::V6(interface_ipv6))
        .map_err(SetupWintunAdapterError::SetIpv6Addr)
}

/// Sets MTU, metric, and disables unnecessary features for the IP interfaces
/// on the specified network interface (identified by `luid`).
pub fn initialize_interfaces(
    luid: NET_LUID_LH,
    ipv4_mtu: Option<u16>,
    ipv6_mtu: Option<u16>,
) -> Result<()> {
    for (family, mtu) in &[
        (AddressFamily::Ipv4, ipv4_mtu),
        (AddressFamily::Ipv6, ipv6_mtu),
    ] {
        let mut row = match wnet::get_ip_interface_entry(*family, &luid) {
            Ok(row) => row,
            Err(error) if error.code() == ERROR_NOT_FOUND.into() => {
                tracing::warn!("Interface not found for {family}");
                continue;
            }
            Err(error) => return Err(SetupWintunAdapterError::SetMtu(error)),
        };

        if let Some(mtu) = mtu {
            row.NlMtu = u32::from(*mtu);
        }

        // Disable DAD, DHCP, and router discovery
        row.SitePrefixLength = 0;
        row.RouterDiscoveryBehavior = RouterDiscoveryDisabled;
        row.DadTransmits = 0;
        row.ManagedAddressConfigurationSupported = false;
        row.OtherStatefulConfigurationSupported = false;

        // Ensure lowest interface metric
        row.Metric = 1;
        row.UseAutomaticMetric = false;

        wnet::set_ip_interface_entry(&mut row)
            .map_err(SetupWintunAdapterError::SetIpInterfaceEntry)?;
    }

    Ok(())
}

/// Waits until the specified IP interfaces have attached to a given network interface.
pub async fn wait_for_interfaces(
    interface_luid: NET_LUID_LH,
    ipv4: bool,
    ipv6: bool,
) -> Result<()> {
    wnet::wait_for_interfaces(interface_luid, ipv4, ipv6)
        .await
        .map_err(SetupWintunAdapterError::WaitForInterfaces)
}

/// Wait for addresses to be usable on an network adapter.
pub async fn wait_for_addresses(interface_luid: NET_LUID_LH) -> Result<()> {
    wnet::wait_for_addresses(interface_luid)
        .await
        .map_err(SetupWintunAdapterError::WaitForInterfaceAddresses)
}
