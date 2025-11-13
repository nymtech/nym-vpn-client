use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

pub const DHCP_CLIENT_PORT_V4: u16 = 68;
pub const DHCP_SERVER_PORT_V4: u16 = 67;
pub const DHCP_CLIENT_PORT_V6: u16 = 546;
pub const DHCP_SERVER_PORT_V6: u16 = 547;
pub const DNS_SERVER_PORT: u16 = 53;

pub const INADDR_ALL_V4: IpAddr = IpAddr::V4(Ipv4Addr::new(255, 255, 255, 255));
pub const INADDR_ANY_V4: IpAddr = IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0));
pub const LINK_LOCAL_V4: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

pub const LINK_LOCAL_V6: IpAddr =
    IpAddr::V6(Ipv6Addr::new(0xFE80, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0));
pub const LINK_LOCAL_DHCP_MULTICAST_V6: IpAddr =
    IpAddr::V6(Ipv6Addr::new(0xFF02, 0x0, 0x0, 0x0, 0x0, 0x0, 0x1, 0x2));
pub const SITE_LOCAL_DHCP_MULTICAST_V6: IpAddr =
    IpAddr::V6(Ipv6Addr::new(0xFF05, 0x0, 0x0, 0x0, 0x0, 0x0, 0x1, 0x3));
