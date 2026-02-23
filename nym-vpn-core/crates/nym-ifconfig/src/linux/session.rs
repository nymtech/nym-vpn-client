// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use futures::{StreamExt, TryStreamExt};
use ipnetwork::IpNetwork;
use rtnetlink::{
    AddressMessageBuilder, LinkMessageBuilder, LinkUnspec,
    packet_route::{address::AddressAttribute, link::LinkAttribute},
};
use tokio_util::sync::{CancellationToken, DropGuard};

// Re-exports
pub use rtnetlink::packet_route::link::LinkFlags;

use crate::{Error, ErrorKind, Result};

/// Network interface configuration session
#[derive(Debug)]
pub struct Session {
    rt_handle: rtnetlink::Handle,
    _drop_guard: DropGuard,
}

impl Session {
    pub fn new() -> Result<Self> {
        let shutdown_token = CancellationToken::new();
        let child_token = shutdown_token.child_token();
        let drop_guard = shutdown_token.drop_guard();

        let (connection, rt_handle, mut rx) = rtnetlink::new_connection()
            .map_err(|err| Error::new(ErrorKind::Netlink, Box::new(err)))?;
        tokio::spawn(connection);
        tokio::spawn(async move {
            let _ = child_token
                .run_until_cancelled(async {
                    loop {
                        while let Some(_event) = rx.next().await {
                            // Consume events to avoid unbounded channel from overflowing
                        }
                    }
                })
                .await;
        });

        Ok(Self {
            rt_handle,
            _drop_guard: drop_guard,
        })
    }

    pub async fn add_address(
        &self,
        interface: &str,
        addr: IpNetwork,
        destination: Option<IpAddr>,
    ) -> Result<()> {
        let mut links = self
            .rt_handle
            .link()
            .get()
            .match_name(interface.to_owned())
            .execute();

        let link = links
            .try_next()
            .await?
            .ok_or(Error::without_source(ErrorKind::InterfaceNotFound))?;

        let mut msg = self
            .rt_handle
            .address()
            .add(link.header.index, addr.ip(), addr.prefix());

        // Same as: ip address add 10.8.0.1 peer 10.8.0.2/32 dev tun0
        if let Some(destination) = destination {
            for attr in msg.message_mut().attributes.iter_mut() {
                if let AddressAttribute::Address(_) = attr {
                    *attr = AddressAttribute::Address(destination);
                }
            }
        }

        msg.execute().await?;

        Ok(())
    }

    pub async fn remove_address(
        &self,
        interface: impl Into<String>,
        addr: IpNetwork,
    ) -> Result<()> {
        let mut links = self
            .rt_handle
            .link()
            .get()
            .match_name(interface.into())
            .execute();

        let link = links
            .try_next()
            .await?
            .ok_or(Error::without_source(ErrorKind::InterfaceNotFound))?;

        let message = match addr {
            IpNetwork::V4(addr) => AddressMessageBuilder::<Ipv4Addr>::new()
                .index(link.header.index)
                .address(addr.ip(), addr.prefix())
                .build(),
            IpNetwork::V6(addr) => AddressMessageBuilder::<Ipv6Addr>::new()
                .index(link.header.index)
                .address(addr.ip(), addr.prefix())
                .build(),
        };

        self.rt_handle.address().del(message).execute().await?;

        Ok(())
    }

    /// Get IP addresses assigned on interface.
    pub async fn addresses(
        &self,
        interface: impl Into<String>,
    ) -> Result<Vec<InterfaceIpAddrEntry>> {
        let mut links = self
            .rt_handle
            .link()
            .get()
            .match_name(interface.into())
            .execute();

        let link = links
            .try_next()
            .await?
            .ok_or(Error::without_source(ErrorKind::InterfaceNotFound))?;

        let mut stream = self
            .rt_handle
            .address()
            .get()
            .set_link_index_filter(link.header.index)
            .execute();

        let mut ip_entries = vec![];
        while let Some(msg) = stream.try_next().await? {
            let mut ip_entry = InterfaceIpAddrEntry {
                address: IpAddr::V4(Ipv4Addr::UNSPECIFIED),
                destination: None,
            };

            let mut addr_ip: Option<IpAddr> = None;
            let mut local_ip: Option<IpAddr> = None;

            for attr in msg.attributes {
                match attr {
                    AddressAttribute::Address(addr) => {
                        addr_ip = Some(addr);
                    }
                    AddressAttribute::Anycast(addr) => {
                        ip_entry.address = IpAddr::from(addr);
                    }
                    AddressAttribute::Local(addr) => {
                        local_ip = Some(addr);
                    }
                    AddressAttribute::Flags(_flags) => {
                        // tbd
                    }
                    _ => {}
                }
            }

            match (addr_ip, local_ip) {
                (Some(addr), None) | (None, Some(addr)) => {
                    ip_entry.address = addr;
                }
                (Some(gateway), Some(local)) => {
                    ip_entry.address = local;
                    if local != gateway {
                        ip_entry.destination = Some(gateway);
                    }
                }
                (None, None) => {}
            }

            ip_entries.push(ip_entry);
        }

        Ok(ip_entries)
    }

    pub async fn mtu(&mut self, interface: impl Into<String>) -> Result<u32> {
        let mut links = self
            .rt_handle
            .link()
            .get()
            .match_name(interface.into())
            .execute();

        let link = links
            .try_next()
            .await?
            .ok_or(Error::without_source(ErrorKind::InterfaceNotFound))?;

        link.attributes
            .iter()
            .find_map(|attr| {
                if let LinkAttribute::Mtu(mtu) = attr {
                    Some(*mtu)
                } else {
                    None
                }
            })
            .ok_or(Error::without_source(ErrorKind::MtuNotFound))
    }

    pub async fn set_mtu(&mut self, interface: impl Into<String>, mtu: u32) -> Result<()> {
        let msg = LinkMessageBuilder::<LinkUnspec>::default()
            .name(interface.into())
            .mtu(mtu)
            .build();

        self.rt_handle.link().set(msg).execute().await?;

        Ok(())
    }

    pub async fn interface_flags(&mut self, interface: impl Into<String>) -> Result<LinkFlags> {
        let mut links = self
            .rt_handle
            .link()
            .get()
            .match_name(interface.into())
            .execute();

        let link = links
            .try_next()
            .await?
            .ok_or(Error::without_source(ErrorKind::InterfaceNotFound))?;

        Ok(link.header.flags)
    }

    pub async fn up(&mut self, interface: impl Into<String>) -> Result<()> {
        let msg = LinkMessageBuilder::<LinkUnspec>::default()
            .name(interface.into())
            .up()
            .build();

        self.rt_handle.link().set(msg).execute().await?;

        Ok(())
    }

    pub async fn down(&mut self, interface: &str) -> Result<()> {
        let msg = LinkMessageBuilder::<LinkUnspec>::default()
            .name(interface.to_owned())
            .down()
            .build();

        self.rt_handle.link().set(msg).execute().await?;

        Ok(())
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct InterfaceIpAddrEntry {
    pub address: IpAddr,
    pub destination: Option<IpAddr>,
}

#[cfg(test)]
mod tests {
    use std::net::IpAddr;

    use super::{super::tun::Tun, *};

    #[tokio::test]
    #[serial_test::serial]
    async fn test_add_p2p_ipv4_address() {
        let tun = Tun::new().expect("failed to create tun");
        let interface = tun.name().unwrap();

        let ipv4_addr: IpNetwork = "10.2.0.10/32".parse().unwrap();

        let sess = Session::new().unwrap();
        sess.add_address(&interface, ipv4_addr, None)
            .await
            .expect("failed to add address");
        assert!(
            sess.addresses(&interface)
                .await
                .unwrap()
                .into_iter()
                .any(|ip| ip.address == ipv4_addr.ip())
        );

        sess.remove_address(&interface, ipv4_addr)
            .await
            .expect("failed to remove address");
        assert!(
            sess.addresses(&interface)
                .await
                .expect("failed to obtain addresses")
                .is_empty()
        );

        ifconfig(&interface);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_add_p2p_ipv4_destination() {
        let tun = Tun::new().expect("failed to create tun");
        let interface = tun.name().expect("failed to obtain interface name");

        let ipv4_addr: IpNetwork = "10.2.0.10/32".parse().unwrap();
        let ipv4_destination: Ipv4Addr = "10.2.0.1".parse().unwrap();

        let sess = Session::new().expect("failed to create session");
        sess.add_address(&interface, ipv4_addr, Some(IpAddr::from(ipv4_destination)))
            .await
            .unwrap();
        assert!(
            sess.addresses(&interface)
                .await
                .unwrap()
                .into_iter()
                .any(|ip| ip.address == ipv4_addr.ip())
        );

        ifconfig(&interface);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_set_p2p_ipv6_address() {
        let tun = Tun::new().expect("failed to create tun");
        let interface = tun.name().expect("failed to obtain interface name");

        let sess = Session::new().expect("failed to create session");
        let ipv6_addr: IpNetwork = "0f9e:6e75:16c0:29a3:a5dc:d5db:2a73:2d85/64"
            .parse()
            .unwrap();

        sess.add_address(&interface, ipv6_addr, None)
            .await
            .expect("failed to add address");
        assert!(
            sess.addresses(&interface)
                .await
                .unwrap()
                .into_iter()
                .any(|ip| ip.address == ipv6_addr.ip())
        );

        sess.remove_address(&interface, ipv6_addr)
            .await
            .expect("failed to remove session");
        assert!(
            sess.addresses(&interface)
                .await
                .expect("failed to get addresses")
                .is_empty()
        );

        ifconfig(&interface);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_set_p2p_mtu() {
        use rand::Rng;

        let tun = Tun::new().expect("failed to create tun");
        let interface = tun.name().expect("failed to obtain interface name");
        let mut sess = Session::new().expect("failed to create session");

        let mtu = rand::thread_rng().gen_range(1280..=1480);
        sess.set_mtu(&interface, mtu)
            .await
            .expect("failed to set mtu");

        let current_mtu = sess.mtu(&interface).await.expect("failed to get mtu");
        assert_eq!(mtu, current_mtu);

        ifconfig(&interface);
    }

    #[tokio::test]
    #[serial_test::serial]
    async fn test_p2p_interface_up_down() {
        let tun = Tun::new().expect("failed to create tun");
        let interface = tun.name().expect("failed to obtain interface name");
        let mut sess = Session::new().expect("failed to create session");

        assert!(
            !sess
                .interface_flags(&interface)
                .await
                .expect("failed to get interface flags")
                .contains(LinkFlags::Up)
        );

        sess.up(&interface)
            .await
            .expect("failed to bring up interface");

        assert!(
            sess.interface_flags(&interface)
                .await
                .expect("failed to get interface flags")
                .contains(LinkFlags::Up)
        );

        sess.down(&interface)
            .await
            .expect("failed to bring down interface");

        assert!(
            !sess
                .interface_flags(&interface)
                .await
                .expect("failed to get interface flags")
                .contains(LinkFlags::Up)
        );

        ifconfig(&interface);
    }

    fn ifconfig(name: &str) {
        use std::io::Write;

        let output = std::process::Command::new("ifconfig")
            .arg(name)
            .output()
            .unwrap();

        std::io::stdout().write_all(&output.stdout).unwrap();
        std::io::stderr().write_all(&output.stderr).unwrap();
    }
}
