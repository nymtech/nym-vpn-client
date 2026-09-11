// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::net::Ipv6Addr;

use ipnetwork::Ipv6Network;

#[cfg(target_os = "linux")]
pub async fn set_ipv6_addr(device_name: &str, ipv6_addr: Ipv6Addr) -> nym_ifconfig::Result<()> {
    use ipnetwork::IpNetwork;
    use nym_ifconfig::Session;

    let sess = Session::new()?;
    sess.add_address(
        device_name,
        IpNetwork::V6(Ipv6Network::from(ipv6_addr)),
        None,
    )
    .await?;

    Ok(())
}

#[cfg(target_os = "macos")]
pub async fn set_ipv6_addr(device_name: &str, ipv6_addr: Ipv6Addr) -> nym_ifconfig::Result<()> {
    use nym_ifconfig::{AddAddressRequestV6, Ipv6AddrFlags, Ipv6AddrLifetime, Session};

    let mut sess = Session::default();
    sess.add_address(
        device_name,
        AddAddressRequestV6 {
            address: Ipv6Network::from(ipv6_addr),
            destination: None,
            lifetime: Ipv6AddrLifetime::default(),
            flags: Ipv6AddrFlags::IN6_IFF_NODAD,
        },
    )?;

    Ok(())
}
