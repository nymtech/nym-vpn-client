// Copyright 2016-2024 Mullvad VPN AB. All Rights Reserved.
// Copyright 2024 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    collections::{BTreeMap, HashSet},
    io,
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    num::NonZeroI32,
    sync::LazyLock,
};

use futures::{StreamExt, TryStreamExt};
use ipnetwork::IpNetwork;
use libc::RT_TABLE_COMPAT;
use nym_common::trace_err_chain;
use rtnetlink::{
    Handle, RouteMessageBuilder,
    constants::{RTMGRP_IPV4_ROUTE, RTMGRP_IPV6_ROUTE, RTMGRP_LINK, RTMGRP_NOTIFY},
    packet_core::{
        NLM_F_ACK, NLM_F_CREATE, NLM_F_DUMP, NLM_F_REPLACE, NLM_F_REQUEST, NetlinkMessage,
        NetlinkPayload,
    },
    packet_route::{
        AddressFamily, RouteNetlinkMessage,
        link::{LinkAttribute, LinkLayerType, LinkMessage},
        route::{
            RouteAddress, RouteAttribute, RouteFlags, RouteHeader, RouteMessage, RouteMetric,
            RouteProtocol, RouteScope, RouteType, RouteVia,
        },
        rule::{RuleAction, RuleAttribute, RuleFlags, RuleHeader, RuleMessage},
    },
    sys::{AsyncSocket, SocketAddr},
};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

use crate::{
    NetNode, Node, RequiredRoute, Route,
    imp::{CallbackMessage, RouteManagerCommand},
};

static SUPPRESS_RULE_V4: LazyLock<RuleMessage> = LazyLock::new(|| {
    let mut rule = RuleMessage::default();
    rule.header = RuleHeader {
        family: AddressFamily::Inet,
        action: RuleAction::ToTable,
        ..RuleHeader::default()
    };
    rule.attributes = vec![
        RuleAttribute::SuppressPrefixLen(0),
        RuleAttribute::Table(RouteHeader::RT_TABLE_MAIN as u32),
    ];
    rule
});
static SUPPRESS_RULE_V6: LazyLock<RuleMessage> = LazyLock::new(|| {
    let mut v6_rule = SUPPRESS_RULE_V4.clone();
    v6_rule.header.family = AddressFamily::Inet6;
    v6_rule
});

fn all_rules(fwmark: u32, table: u32) -> [RuleMessage; 4] {
    [
        no_fwmark_rule_v4(fwmark, table),
        no_fwmark_rule_v6(fwmark, table),
        SUPPRESS_RULE_V4.clone(),
        SUPPRESS_RULE_V6.clone(),
    ]
}

fn no_fwmark_rule_v4(fwmark: u32, table: u32) -> RuleMessage {
    let mut rule = RuleMessage::default();
    rule.header = RuleHeader {
        family: AddressFamily::Inet,
        action: RuleAction::ToTable,
        flags: RuleFlags::Invert,
        ..RuleHeader::default()
    };
    rule.attributes = vec![RuleAttribute::FwMark(fwmark), RuleAttribute::Table(table)];
    rule
}

fn no_fwmark_rule_v6(fwmark: u32, table: u32) -> RuleMessage {
    let mut v6_rule = no_fwmark_rule_v4(fwmark, table);
    v6_rule.header.family = AddressFamily::Inet6;
    v6_rule
}

pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can happen in the Linux routing integration
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("failed to open a netlink connection")]
    Connect(#[source] io::Error),

    #[error("failed to bind netlink socket")]
    Bind(#[source] io::Error),

    #[error("netlink error")]
    Netlink(#[source] rtnetlink::Error),

    #[error("route without a valid node")]
    InvalidRoute,

    #[error("failed to convert route address to IP. Received: {0}")]
    ConvertRouteAddrToIp(String),

    #[error("failed to convert route via to IP. Received: {0}")]
    ConvertRouteViaToIp(String),

    #[error("invalid network prefix")]
    InvalidNetworkPrefix(#[source] ipnetwork::IpNetworkError),

    #[error("unknown device index: {0}")]
    UnknownDeviceIndex(u32),

    #[error("failed to get a route for the given IP address")]
    GetRoute(#[source] rtnetlink::Error),

    #[error("no netlink response for route query")]
    NoRoute,

    #[error("route node was malformed")]
    InvalidRouteNode,

    #[error("no link found")]
    LinkNotFound,

    /// Unable to create routing table for tagged connections and packets.
    #[error("cannot find a free routing table ID")]
    NoFreeRoutingTableId,

    #[error("shutting down route manager")]
    Shutdown,
}

pub struct RouteManagerImpl {
    handle: Handle,
    messages: futures::channel::mpsc::UnboundedReceiver<(
        NetlinkMessage<RouteNetlinkMessage>,
        SocketAddr,
    )>,
    iface_map: BTreeMap<u32, NetworkInterface>,
    listeners: Vec<UnboundedSender<CallbackMessage>>,

    // currently added routes
    added_routes: HashSet<Route>,

    /// Tunnel specific routing table, traffic not marked will be routed via this routing table.
    table_id: u32,
    /// Firewall mark identifies traffic which shouldn't be routed via the tunnel routing table. It
    /// is used to construct a routing rule.
    fwmark: u32,
}

impl RouteManagerImpl {
    pub async fn new(table_id: u32, fwmark: u32) -> Result<Self> {
        let (mut connection, handle, messages) =
            rtnetlink::new_connection().map_err(Error::Connect)?;

        let mgroup_flags = RTMGRP_IPV4_ROUTE | RTMGRP_IPV6_ROUTE | RTMGRP_LINK | RTMGRP_NOTIFY;
        let addr = SocketAddr::new(0, mgroup_flags);
        connection
            .socket_mut()
            .socket_mut()
            .bind(&addr)
            .map_err(Error::Bind)?;

        tokio::spawn(connection);

        let iface_map = Self::initialize_link_map(&handle).await?;

        let mut monitor = Self {
            handle,
            messages,
            iface_map,
            listeners: vec![],
            added_routes: HashSet::new(),
            table_id,
            fwmark,
        };

        monitor.clear_routing_rules().await?;

        Ok(monitor)
    }

    async fn create_routing_rules(&mut self, enable_ipv6: bool) -> Result<()> {
        self.clear_routing_rules().await?;

        for rule in all_rules(self.fwmark, self.table_id)
            .into_iter()
            .filter(|rule| rule.header.family == AddressFamily::Inet || enable_ipv6)
        {
            let mut req = NetlinkMessage::from(RouteNetlinkMessage::NewRule(rule));
            req.header.flags = NLM_F_REQUEST | NLM_F_ACK | NLM_F_CREATE | NLM_F_REPLACE;

            let mut response = self.handle.request(req).map_err(Error::Netlink)?;

            while let Some(message) = response.next().await {
                if let NetlinkPayload::Error(error) = message.payload {
                    return Err(Error::Netlink(rtnetlink::Error::NetlinkError(error)));
                }
            }
        }
        Ok(())
    }

    async fn clear_routing_rules(&mut self) -> Result<()> {
        let rules = self.get_rules().await?;
        for rule in all_rules(self.fwmark, self.table_id) {
            let mut matching_rule = None;

            // `RTM_DELRULE` is way too picky about which rules are considered the same.
            // So iterate over all rules and ignore irrelevant attributes.
            for found_rule in &rules {
                // Match header
                if found_rule.header.family != rule.header.family {
                    continue;
                }
                if found_rule.header.action != rule.header.action {
                    continue;
                }

                let found_rule_flags = found_rule.header.flags;
                let rule_flags = rule.header.flags;
                if (found_rule_flags & rule_flags) != rule_flags {
                    continue;
                }

                // Match NLAs
                let mut contains_nlas = true;
                for nla in &rule.attributes {
                    if !found_rule.attributes.contains(nla) {
                        contains_nlas = false;
                        break;
                    }
                }
                if contains_nlas {
                    tracing::trace!("Existing routing rule matched: {:?}", found_rule);
                    matching_rule = Some(found_rule);
                    break;
                }
            }

            if let Some(rule) = matching_rule {
                self.delete_rule_if_exists((*rule).clone()).await?;
            }
        }
        Ok(())
    }

    async fn get_rules(&mut self) -> Result<Vec<RuleMessage>> {
        let mut req = NetlinkMessage::from(RouteNetlinkMessage::GetRule(RuleMessage::default()));
        req.header.flags = NLM_F_REQUEST | NLM_F_ACK | NLM_F_DUMP;

        let mut response = self.handle.request(req).map_err(Error::Netlink)?;

        let mut rules = vec![];

        while let Some(message) = response.next().await {
            match message.payload {
                NetlinkPayload::InnerMessage(RouteNetlinkMessage::NewRule(rule)) => {
                    rules.push(rule);
                }
                NetlinkPayload::Error(error) => {
                    return Err(Error::Netlink(rtnetlink::Error::NetlinkError(error)));
                }
                _ => (),
            }
        }
        Ok(rules)
    }

    async fn delete_rule_if_exists(&mut self, rule: RuleMessage) -> Result<()> {
        let mut req = NetlinkMessage::from(RouteNetlinkMessage::DelRule(rule));
        req.header.flags = NLM_F_REQUEST | NLM_F_ACK;

        let mut response = self.handle.request(req).map_err(Error::Netlink)?;

        while let Some(message) = response.next().await {
            if let NetlinkPayload::Error(error) = message.payload
                && error.to_io().kind() != io::ErrorKind::NotFound
            {
                return Err(Error::Netlink(rtnetlink::Error::NetlinkError(error)));
            }
        }
        Ok(())
    }

    async fn add_required_routes(&mut self, required_routes: HashSet<RequiredRoute>) -> Result<()> {
        let mut required_normal_routes = HashSet::new();

        for route in required_routes {
            match route.node {
                NetNode::RealNode(node) => {
                    let table = if route.main_table {
                        RouteHeader::RT_TABLE_MAIN.into()
                    } else {
                        self.table_id
                    };
                    let mut new_route = Route::new(node, route.prefix).table(table);
                    new_route.mtu = route.mtu.map(u32::from);
                    required_normal_routes.insert(new_route);
                }
            }
        }

        for normal_route in required_normal_routes.into_iter() {
            self.add_route(normal_route).await?;
        }

        Ok(())
    }

    async fn initialize_link_map(
        handle: &rtnetlink::Handle,
    ) -> Result<BTreeMap<u32, NetworkInterface>> {
        let mut link_map = BTreeMap::new();
        let mut link_request = handle.link().get().execute();
        while let Some(link) = link_request.try_next().await.map_err(Error::Netlink)? {
            if let Some((idx, device)) = Self::map_interface(link) {
                link_map.insert(idx, device);
            }
        }

        Ok(link_map)
    }

    fn find_iface_idx(&self, iface_name: &str) -> Option<u32> {
        self.iface_map
            .iter()
            .find(|(_idx, iface)| iface.name.as_str() == iface_name)
            .map(|(idx, _name)| *idx)
    }

    fn process_deleted_route(&mut self, route: &Route) -> Result<()> {
        self.added_routes.remove(route);
        Ok(())
    }

    async fn cleanup_routes(&mut self) {
        for route in self.added_routes.drain().collect::<Vec<_>>().iter() {
            if let Err(e) = self.delete_route_if_exists(route).await {
                tracing::error!("Failed to remove route: {}: {}", route, e);
            }
        }
    }

    pub(crate) async fn run(
        mut self,
        mut manage_rx: UnboundedReceiver<RouteManagerCommand>,
    ) -> Result<()> {
        loop {
            tokio::select! {
                command = manage_rx.recv() => {
                    self.process_command(command).await?;
                },
                Some((route_change, _socket)) = self.messages.next() => {
                    if let Err(error) = self.process_netlink_message(route_change) {
                        trace_err_chain!(error, "Failed to process netlink message");
                    }
                }
            };
        }
    }

    async fn process_command(&mut self, command: Option<RouteManagerCommand>) -> Result<()> {
        match command {
            None | Some(RouteManagerCommand::Shutdown(_)) => {
                tracing::trace!("Shutting down route manager");
                self.destructor().await;
                tracing::trace!("Route manager done");
                if let Some(RouteManagerCommand::Shutdown(shutdown_signal)) = command {
                    let _ = shutdown_signal.send(());
                }
                return Err(Error::Shutdown);
            }
            Some(RouteManagerCommand::AddRoutes(routes, result_tx)) => {
                tracing::debug!("Adding routes: {:?}", routes);
                let _ = result_tx.send(self.add_required_routes(routes.clone()).await);
            }
            Some(RouteManagerCommand::CreateRoutingRules(enable_ipv6, result_tx)) => {
                let _ = result_tx.send(self.create_routing_rules(enable_ipv6).await);
            }
            Some(RouteManagerCommand::ClearRoutingRules(result_tx)) => {
                let _ = result_tx.send(self.clear_routing_rules().await);
            }
            Some(RouteManagerCommand::NewChangeListener(result_tx)) => {
                let _ = result_tx.send(self.listen());
            }
            Some(RouteManagerCommand::GetDestinationRoute(destination, mark, result_tx)) => {
                let _ = result_tx.send(self.get_destination_route(destination, mark).await);
            }
            Some(RouteManagerCommand::GetMtuForRoute(ip, result_tx)) => {
                let _ = result_tx.send(self.get_mtu_for_route(ip).await);
            }
            Some(RouteManagerCommand::ClearRoutes) => {
                tracing::debug!("Clearing routes");
                self.cleanup_routes().await;
            }
        }
        Ok(())
    }

    fn process_netlink_message(&mut self, msg: NetlinkMessage<RouteNetlinkMessage>) -> Result<()> {
        match msg.payload {
            NetlinkPayload::InnerMessage(RouteNetlinkMessage::NewLink(new_link)) => {
                if let Some((idx, name)) = Self::map_interface(new_link) {
                    self.iface_map.insert(idx, name);
                }
            }
            NetlinkPayload::InnerMessage(RouteNetlinkMessage::DelLink(old_link)) => {
                if let Some((idx, _)) = Self::map_interface(old_link) {
                    self.iface_map.remove(&idx);
                }
            }
            NetlinkPayload::InnerMessage(RouteNetlinkMessage::NewRoute(new_route)) => {
                if let Some(addition) = self.parse_route_message(new_route)? {
                    self.notify_change_listeners(CallbackMessage::NewRoute(addition));
                }
            }
            NetlinkPayload::InnerMessage(RouteNetlinkMessage::DelRoute(old_route)) => {
                if let Some(deletion) = self.parse_route_message(old_route)? {
                    self.process_deleted_route(&deletion)?;
                    self.notify_change_listeners(CallbackMessage::DelRoute(deletion));
                }
            }
            _ => (),
        };
        Ok(())
    }

    fn notify_change_listeners(&mut self, message: CallbackMessage) {
        self.listeners
            .retain(|listener| listener.send(message.clone()).is_ok());
    }

    // Tries to coax a Route out of a RouteMessage
    fn parse_route_message(&self, msg: RouteMessage) -> Result<Option<Route>> {
        let af_spec = msg.header.address_family;
        let destination_length = msg.header.destination_prefix_length;
        let is_ipv4 = match af_spec {
            AddressFamily::Inet => true,
            AddressFamily::Inet6 => false,
            af_spec => {
                tracing::error!("Unexpected routing protocol: {:?}", af_spec);
                return Ok(None);
            }
        };

        // By default, the prefix is unspecified.
        let mut prefix = IpNetwork::new(
            if is_ipv4 {
                Ipv4Addr::UNSPECIFIED.into()
            } else {
                Ipv6Addr::UNSPECIFIED.into()
            },
            destination_length,
        )
        .map_err(Error::InvalidNetworkPrefix)?;

        let mut node_addr = None;
        let mut device = None;
        let mut metric = None;
        let mut gateway: Option<IpAddr> = None;
        let mut table_id = u32::from(msg.header.table);
        let mut route_mtu = None;

        for nla in msg.attributes.iter() {
            match nla {
                RouteAttribute::Oif(device_idx) => {
                    match self.iface_map.get(device_idx) {
                        Some(route_device) => {
                            if !route_device.is_loopback() {
                                device = Some(route_device);
                            } else {
                                gateway = if is_ipv4 {
                                    Some(Ipv4Addr::LOCALHOST.into())
                                } else {
                                    Some(Ipv6Addr::LOCALHOST.into())
                                };
                            }
                        }
                        None => {
                            return Err(Error::UnknownDeviceIndex(*device_idx));
                        }
                    };
                }

                RouteAttribute::Via(addr) => {
                    node_addr = route_via_to_ip(addr.clone()).map(Some)?;
                }

                RouteAttribute::Destination(addr) => {
                    prefix = route_address_to_ip(addr.clone()).and_then(|ip| {
                        ipnetwork::IpNetwork::new(ip, destination_length)
                            .map_err(Error::InvalidNetworkPrefix)
                    })?;
                }

                // gateway NLAs indicate that this is actually a default route
                RouteAttribute::Gateway(gateway_ip) => {
                    gateway = route_address_to_ip(gateway_ip.clone()).map(Some)?;
                }

                RouteAttribute::Priority(priority) => {
                    metric = Some(*priority);
                }

                RouteAttribute::Table(id) => {
                    table_id = *id;
                }

                RouteAttribute::Metrics(metrics) => {
                    for metric in metrics {
                        if let RouteMetric::Mtu(mtu) = metric {
                            route_mtu = Some(*mtu);
                        }
                    }
                }
                _ => continue,
            }
        }

        if device.is_none() && node_addr.is_none() && gateway.is_none() {
            return Err(Error::InvalidRoute);
        }

        let node = Node {
            ip: node_addr.or(gateway),
            device: device.map(|dev| dev.name.clone()),
        };

        Ok(Some(Route {
            node,
            prefix,
            metric,
            table_id,
            mtu: route_mtu,
        }))
    }

    fn map_interface(msg: LinkMessage) -> Option<(u32, NetworkInterface)> {
        let index = msg.header.index;
        let link_layer_type = msg.header.link_layer_type;
        for nla in msg.attributes {
            if let LinkAttribute::IfName(name) = nla {
                return Some((
                    index,
                    NetworkInterface {
                        name,
                        link_layer_type,
                    },
                ));
            }
        }

        None
    }

    async fn delete_route_if_exists(&self, route: &Route) -> Result<()> {
        if let Err(error) = self.delete_route(route).await {
            if let Error::Netlink(rtnetlink::Error::NetlinkError(msg)) = &error
                && msg.code == NonZeroI32::new(-libc::ESRCH)
            {
                return Ok(());
            }
            Err(error)
        } else {
            Ok(())
        }
    }

    async fn delete_route(&self, route: &Route) -> Result<()> {
        let compat_table = compat_table_id(route.table_id);
        let scope = match route.prefix {
            IpNetwork::V4(v4_prefix) => {
                if v4_prefix.prefix() > 0 && v4_prefix.prefix() < 32 {
                    RouteScope::Link
                } else {
                    RouteScope::Universe
                }
            }
            IpNetwork::V6(v6_prefix) => {
                if v6_prefix.prefix() > 0 && v6_prefix.prefix() < 128 {
                    RouteScope::Link
                } else {
                    RouteScope::Universe
                }
            }
        };

        let mut route_message = RouteMessage::default();
        route_message.header = RouteHeader {
            address_family: if route.prefix.is_ipv4() {
                AddressFamily::Inet
            } else {
                AddressFamily::Inet6
            },
            source_prefix_length: 0,
            destination_prefix_length: route.prefix.prefix(),
            tos: 0u8,
            table: compat_table,
            protocol: RouteProtocol::Unspec,
            scope,
            kind: RouteType::Unspec,
            flags: RouteFlags::empty(),
        };
        route_message.attributes = vec![RouteAttribute::Destination(ip_to_route_address(
            route.prefix.ip(),
        ))];

        if compat_table == RT_TABLE_COMPAT {
            route_message
                .attributes
                .push(RouteAttribute::Table(route.table_id));
        }

        if let Some(interface_name) = route.node.get_device()
            && let Some(iface_idx) = self.find_iface_idx(interface_name)
        {
            route_message
                .attributes
                .push(RouteAttribute::Oif(iface_idx));
        }

        if let Some(gateway) = route.node.get_address() {
            let gateway_attr = if route.node.get_device().is_some() {
                RouteAttribute::Gateway(ip_to_route_address(gateway))
            } else {
                RouteAttribute::Via(ip_to_route_via(gateway))
            };
            route_message.attributes.push(gateway_attr);
        }

        if let Some(metric) = route.metric {
            route_message
                .attributes
                .push(RouteAttribute::Priority(metric));
        }

        self.handle
            .route()
            .del(route_message)
            .execute()
            .await
            .map_err(Error::Netlink)
    }

    async fn add_route_direct(&mut self, route: Route) -> Result<()> {
        let mut add_message = match &route.prefix {
            IpNetwork::V4(v4_prefix) => {
                let mut message_builder = RouteMessageBuilder::<Ipv4Addr>::new()
                    .destination_prefix(v4_prefix.ip(), v4_prefix.prefix());

                if v4_prefix.prefix() > 0 && v4_prefix.prefix() < 32 {
                    message_builder = message_builder.scope(RouteScope::Link);
                }

                if let Some(IpAddr::V4(node_address)) = route.node.get_address() {
                    message_builder = message_builder.gateway(node_address);
                }

                if let Some(interface_name) = route.node.get_device()
                    && let Some(iface_idx) = self.find_iface_idx(interface_name)
                {
                    message_builder = message_builder.output_interface(iface_idx);
                }

                message_builder.build()
            }
            IpNetwork::V6(v6_prefix) => {
                let mut message_builder = RouteMessageBuilder::<Ipv6Addr>::new()
                    .destination_prefix(v6_prefix.ip(), v6_prefix.prefix());

                if v6_prefix.prefix() > 0 && v6_prefix.prefix() < 128 {
                    message_builder = message_builder.scope(RouteScope::Link);
                }

                if let Some(IpAddr::V6(node_address)) = route.node.get_address() {
                    message_builder = message_builder.gateway(node_address);
                }

                if let Some(interface_name) = route.node.get_device()
                    && let Some(iface_idx) = self.find_iface_idx(interface_name)
                {
                    message_builder = message_builder.output_interface(iface_idx);
                }

                message_builder.build()
            }
        };

        let compat_table = compat_table_id(route.table_id);
        add_message.header.table = compat_table;
        if compat_table == RT_TABLE_COMPAT {
            add_message
                .attributes
                .push(RouteAttribute::Table(route.table_id));
        }

        // TODO: Request support for route priority in RouteAddIpv{4,6}Request
        if let Some(metric) = route.metric {
            add_message
                .attributes
                .push(RouteAttribute::Priority(metric));
        }

        // Set route MTU
        if let Some(mtu) = route.mtu {
            add_message
                .attributes
                .push(RouteAttribute::Metrics(vec![RouteMetric::Mtu(mtu)]));
        }

        self.handle
            .route()
            .add(add_message)
            .replace()
            .execute()
            .await
            .map_err(Error::Netlink)
    }

    async fn add_route(&mut self, route: Route) -> Result<()> {
        self.add_route_direct(route.clone()).await?;
        self.added_routes.insert(route);
        Ok(())
    }

    fn listen(&mut self) -> UnboundedReceiver<CallbackMessage> {
        let (tx, rx) = mpsc::unbounded_channel();
        self.listeners.push(tx);
        rx
    }

    async fn destructor(&mut self) {
        self.cleanup_routes().await;

        if let Err(error) = self.clear_routing_rules().await {
            trace_err_chain!(error, "Failed to remove routing rules");
        }
    }

    async fn get_mtu_for_route(&self, ip: IpAddr) -> Result<u16> {
        // RECURSION_LIMIT controls how many times we recurse to find the device name by looking up
        // an IP with `get_destination_route`.
        // TODO: Check route MTU first
        const RECURSION_LIMIT: usize = 10;
        const STANDARD_MTU: u16 = 1500;
        let mut attempted_ip = ip;
        for _ in 0..RECURSION_LIMIT {
            let route = self
                .get_destination_route(attempted_ip, Some(self.fwmark))
                .await?;
            match route {
                Some(route) => {
                    let node = route.get_node();
                    match (node.get_device(), node.get_address()) {
                        (Some(device), _) => {
                            let mtu = self.get_device_mtu(device.to_string()).await?;
                            if mtu != STANDARD_MTU {
                                tracing::info!(
                                    "Found MTU: {} on device {} which is different from the standard {}",
                                    mtu,
                                    device,
                                    STANDARD_MTU
                                );
                            }
                            return Ok(mtu);
                        }
                        (None, Some(address)) => attempted_ip = address,
                        (None, None) => {
                            tracing::error!(
                                "Route contains an invalid node which lacks both a device and an address"
                            );
                            return Err(Error::InvalidRouteNode);
                        }
                    }
                }
                None => {
                    tracing::error!(
                        "No route detected when assigning the mtu to the Wireguard tunnel"
                    );
                    return Err(Error::NoRoute);
                }
            }
        }
        tracing::error!(
            "Retried {} times looking for the correct device and could not find it",
            RECURSION_LIMIT
        );
        Err(Error::NoRoute)
    }

    async fn get_device_mtu(&self, device: String) -> Result<u16> {
        let mut links = self.handle.link().get().execute();
        let target_device = LinkAttribute::IfName(device);
        while let Some(msg) = links.try_next().await.map_err(|_| Error::LinkNotFound)? {
            let found = msg.attributes.contains(&target_device);
            if found
                && let Some(LinkAttribute::Mtu(mtu)) = msg
                    .attributes
                    .into_iter()
                    .find(|e| matches!(e, LinkAttribute::Mtu(_)))
            {
                return Ok(
                    u16::try_from(mtu).expect("MTU returned by device does not fit into a u16")
                );
            }
        }
        Err(Error::LinkNotFound)
    }

    async fn get_destination_route(
        &self,
        destination: IpAddr,
        fwmark: Option<u32>,
    ) -> Result<Option<Route>> {
        let num_octets = match destination {
            IpAddr::V4(address) => address.octets().len(),
            IpAddr::V6(address) => address.octets().len(),
        };
        let destination_prefix_len = 8u8 * (num_octets as u8);
        let mut message = match destination {
            IpAddr::V4(addr) => RouteMessageBuilder::<Ipv4Addr>::new()
                .destination_prefix(addr, destination_prefix_len)
                .build(),
            IpAddr::V6(addr) => RouteMessageBuilder::<Ipv6Addr>::new()
                .destination_prefix(addr, destination_prefix_len)
                .build(),
        };

        if let Some(mark) = fwmark {
            message.attributes.push(RouteAttribute::Mark(mark));
        }
        message.header.flags = RouteFlags::FibMatch;

        let mut stream = self.handle.route().get(message).execute();
        match stream.try_next().await {
            Ok(Some(route_msg)) => self.parse_route_message(route_msg),
            Ok(None) => Err(Error::NoRoute),
            Err(rtnetlink::Error::NetlinkError(nl_err))
                if nl_err.code == NonZeroI32::new(-libc::ENETUNREACH) =>
            {
                Ok(None)
            }
            Err(err) => Err(Error::GetRoute(err)),
        }
    }
}

fn ip_to_route_address(addr: IpAddr) -> RouteAddress {
    match addr {
        IpAddr::V4(addr) => RouteAddress::Inet(addr),
        IpAddr::V6(addr) => RouteAddress::Inet6(addr),
    }
}

fn route_address_to_ip(route_addr: RouteAddress) -> Result<IpAddr> {
    match route_addr {
        RouteAddress::Inet(ipv4) => Ok(IpAddr::V4(ipv4)),
        RouteAddress::Inet6(ipv6) => Ok(IpAddr::V6(ipv6)),
        other => Err(Error::ConvertRouteAddrToIp(format!("{other:?}"))),
    }
}

fn ip_to_route_via(addr: IpAddr) -> RouteVia {
    match addr {
        IpAddr::V4(addr) => RouteVia::Inet(addr),
        IpAddr::V6(addr) => RouteVia::Inet6(addr),
    }
}

fn route_via_to_ip(via: RouteVia) -> Result<IpAddr> {
    match via {
        RouteVia::Inet(ipv4) => Ok(IpAddr::V4(ipv4)),
        RouteVia::Inet6(ipv6) => Ok(IpAddr::V6(ipv6)),
        other => Err(Error::ConvertRouteViaToIp(format!("{other:?}"))),
    }
}

fn compat_table_id(id: u32) -> u8 {
    // RT_TABLE_COMPAT must be combined with nla Table(id)
    if id > 255 { RT_TABLE_COMPAT } else { id as u8 }
}

#[derive(Debug)]
struct NetworkInterface {
    name: String,
    link_layer_type: LinkLayerType,
}

impl NetworkInterface {
    fn is_loopback(&self) -> bool {
        self.link_layer_type == LinkLayerType::Loopback
    }

    /// Best-effort classification of virtual/tunnel interfaces. Interface
    /// naming isn't guaranteed (a user or tool can name a device anything),
    /// but these prefixes cover the overwhelming majority of VPN clients on
    /// Linux (WireGuard, OpenVPN/TAP, PPP-based clients, NetworkManager's
    /// generic tun devices).
    fn is_tunnel_like(&self) -> bool {
        const TUNNEL_PREFIXES: &[&str] = &["wg", "tun", "tap", "ppp"];
        TUNNEL_PREFIXES
            .iter()
            .any(|prefix| self.name.starts_with(prefix))
    }
}

/// Get every interface currently holding a default-route-shaped entry in the
/// main routing table, for the given address family. See
/// [`crate::DefaultRouteInterfaces`].
///
/// Opens its own short-lived netlink connection rather than reusing a
/// running [`RouteManagerImpl`], since this is meant to be a cheap,
/// standalone, read-only query.
pub(crate) async fn get_default_route_interfaces(
    family: crate::AddressFamily,
) -> std::result::Result<crate::DefaultRouteInterfaces, super::Error> {
    let (connection, mut handle, _) = rtnetlink::new_connection().map_err(Error::Connect)?;
    tokio::spawn(connection);

    let iface_map = RouteManagerImpl::initialize_link_map(&handle).await?;

    let rt_family = match family {
        crate::AddressFamily::Ipv4 => AddressFamily::Inet,
        crate::AddressFamily::Ipv6 => AddressFamily::Inet6,
    };

    let mut route_message = RouteMessage::default();
    route_message.header.address_family = rt_family;

    let mut req = NetlinkMessage::from(RouteNetlinkMessage::GetRoute(route_message));
    req.header.flags = NLM_F_REQUEST | NLM_F_DUMP;

    let mut response = handle.request(req).map_err(Error::Netlink)?;

    let mut result = crate::DefaultRouteInterfaces::default();

    while let Some(message) = response.next().await {
        match message.payload {
            NetlinkPayload::InnerMessage(RouteNetlinkMessage::NewRoute(route)) => {
                if !is_default_route_prefix(&route) || !route_in_main_table(&route) {
                    continue;
                }

                let mut oif = None;
                let mut has_gateway = false;
                for attribute in &route.attributes {
                    match attribute {
                        RouteAttribute::Oif(idx) => oif = Some(*idx),
                        RouteAttribute::Gateway(_) | RouteAttribute::Via(_) => has_gateway = true,
                        _ => {}
                    }
                }

                let (Some(oif), true) = (oif, has_gateway) else {
                    continue;
                };

                let is_virtual = iface_map
                    .get(&oif)
                    .map(NetworkInterface::is_tunnel_like)
                    .unwrap_or(false);

                if is_virtual {
                    result.virtual_.insert(oif);
                } else {
                    result.physical.insert(oif);
                }
            }
            NetlinkPayload::Error(error) => {
                return Err(Error::Netlink(rtnetlink::Error::NetlinkError(error)).into());
            }
            _ => {}
        }
    }

    Ok(result)
}

/// Whether `route` is a default route, or one of the two `/1` halves some
/// VPN clients install instead of replacing `0.0.0.0/0` directly.
fn is_default_route_prefix(route: &RouteMessage) -> bool {
    match route.header.destination_prefix_length {
        0 => true,
        1 if route.header.address_family == AddressFamily::Inet => {
            route.attributes.iter().any(|attribute| match attribute {
                RouteAttribute::Destination(addr) => matches!(
                    route_address_to_ip(addr.clone()),
                    Ok(IpAddr::V4(addr))
                        if addr == Ipv4Addr::new(0, 0, 0, 0) || addr == Ipv4Addr::new(128, 0, 0, 0)
                ),
                _ => false,
            })
        }
        _ => false,
    }
}

/// Whether `route` belongs to the main routing table (as opposed to e.g. a
/// separate policy-routing table NymVPN's own split-tunneling uses).
fn route_in_main_table(route: &RouteMessage) -> bool {
    let mut table_id = u32::from(route.header.table);
    for attribute in &route.attributes {
        if let RouteAttribute::Table(id) = attribute {
            table_id = *id;
        }
    }
    table_id == u32::from(RouteHeader::RT_TABLE_MAIN)
}

#[cfg(test)]
mod test {
    use super::*;

    /// Tests if dropping inside a tokio runtime panics
    #[test]
    fn test_drop_in_executor() {
        let runtime = tokio::runtime::Runtime::new().expect("Failed to initialize runtime");
        runtime.block_on(async {
            let manager = RouteManagerImpl::new(0, 0)
                .await
                .expect("Failed to initialize route manager");
            std::mem::drop(manager);
        });
    }

    /// Tests if dropping outside a runtime panics
    #[test]
    fn test_drop() {
        let runtime = tokio::runtime::Runtime::new().expect("Failed to initialize runtime");
        let manager = runtime.block_on(async {
            RouteManagerImpl::new(1000, 1000)
                .await
                .expect("Failed to initialize route manager")
        });
        std::mem::drop(manager);
    }
}
