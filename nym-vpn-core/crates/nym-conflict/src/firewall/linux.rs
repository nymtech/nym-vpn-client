// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Detects a competing nftables NAT rule that redirects NymVPN's bootstrap
//! DNS traffic (DNS-over-TLS/HTTPS to hardcoded Cloudflare/Quad9 servers -
//! see `nym_http_api_client::dns`'s `default_nameserver_group`) before
//! NymVPN's own firewall chain ever evaluates it.

use std::collections::{HashMap, HashSet};

use netlink_packet_core::{
    NLM_F_DUMP, NLM_F_REQUEST, NetlinkHeader, NetlinkMessage, NetlinkPayload, Nla,
};
use netlink_packet_netfilter::{
    NetfilterHeader, NetfilterMessage, NetfilterMessageInner, NetfilterProtoFamily,
    nftables::{
        ChainAttribute, ChainMessage, Cmp, DataAttribute, ExpressionAttribute, Expressions, Hook,
        HookNumber, Immediate, InetHookNumber, ListAttribute, NfTablesMessage, Payload, Register,
        RuleAttribute, RuleMessage, Verdict, VerdictAttribute,
    },
};
use netlink_sys::{Socket, SocketAddr, protocols::NETLINK_NETFILTER};

use crate::Conflict;

const OWN_TABLE_NAME: &str = "nym";
const BOOTSTRAP_DNS_PORTS: [u64; 2] = [443, 853];

// Raw attributes for expressions not native in netlink-packet-netfilter v0.4.0
const NFTA_RANGE_SREG: u16 = 1;
const NFTA_RANGE_FROM_DATA: u16 = 3;
const NFTA_RANGE_TO_DATA: u16 = 4;
const NFTA_DATA_VALUE: u16 = 1;

const NFTA_NAT_TYPE: u16 = 1;
const NFT_NAT_DNAT: u32 = 1;

const NFTA_TARGET_NAME: u16 = 1;

pub(crate) async fn detect() -> Vec<Conflict> {
    if competing_redirect_exists().await.unwrap_or(false) {
        vec![Conflict::CompetingFirewall]
    } else {
        Vec::new()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ChainKey {
    family: u8,
    table: String,
    name: String,
}

struct ChainInfo {
    key: ChainKey,
    is_output_nat: bool,
}

struct RuleInfo {
    key: ChainKey,
    matches_bootstrap_port: bool,
    has_redirect_verdict: bool,
    jump_or_goto_target: Option<String>,
}

async fn competing_redirect_exists() -> Option<bool> {
    let chains = dump(
        NfTablesMessage::GetChain(ChainMessage { attributes: vec![] }),
        parse_chain,
    )
    .await?;
    let rules = dump(
        NfTablesMessage::GetRule(RuleMessage { attributes: vec![] }),
        parse_rule,
    )
    .await?;

    let mut rules_by_chain: HashMap<ChainKey, Vec<RuleInfo>> = HashMap::new();
    for rule in rules {
        rules_by_chain
            .entry(rule.key.clone())
            .or_default()
            .push(rule);
    }

    let mut to_visit: Vec<ChainKey> = chains
        .into_iter()
        .filter(|chain| chain.is_output_nat && chain.key.table != OWN_TABLE_NAME)
        .map(|chain| chain.key)
        .collect();

    let mut visited: HashSet<ChainKey> = HashSet::new();

    while let Some(key) = to_visit.pop() {
        if !visited.insert(key.clone()) {
            continue;
        }

        for rule in rules_by_chain.get(&key).into_iter().flatten() {
            if rule.matches_bootstrap_port && rule.has_redirect_verdict {
                return Some(true);
            }

            if let Some(target) = &rule.jump_or_goto_target {
                to_visit.push(ChainKey {
                    family: key.family,
                    table: key.table.clone(),
                    name: target.clone(),
                });
            }
        }
    }

    Some(false)
}

fn parse_chain(family: u8, msg: NfTablesMessage) -> Option<ChainInfo> {
    let NfTablesMessage::NewChain(chain) = msg else {
        return None;
    };

    let mut table = None;
    let mut name = None;
    let mut chain_type = None;
    let mut is_output_nat = false;

    for attr in chain.attributes.into_iter() {
        match attr {
            ChainAttribute::Table(t) => table = Some(t),
            ChainAttribute::Name(n) => name = Some(n),
            ChainAttribute::Type(t) => chain_type = Some(t),
            ChainAttribute::Hook(hooks) => {
                for hook in hooks.into_iter() {
                    if let Hook::Number(HookNumber::Inet(InetHookNumber::LocalOut)) = hook {
                        // LOCAL_OUT is NF_INET_LOCAL_OUT (3)
                        is_output_nat = true;
                    } else if let Hook::Number(HookNumber::Other(3)) = hook {
                        is_output_nat = true;
                    }
                }
            }
            _ => {}
        }
    }

    if chain_type.as_deref() != Some("nat") {
        is_output_nat = false;
    }

    Some(ChainInfo {
        key: ChainKey {
            family,
            table: table?,
            name: name?,
        },
        is_output_nat,
    })
}

fn parse_rule(family: u8, msg: NfTablesMessage) -> Option<RuleInfo> {
    let NfTablesMessage::NewRule(rule) = msg else {
        return None;
    };

    let mut table = None;
    let mut chain = None;
    let mut expressions = vec![];

    for attr in rule.attributes.into_iter() {
        match attr {
            RuleAttribute::Table(t) => table = Some(t),
            RuleAttribute::Chain(c) => chain = Some(c),
            RuleAttribute::Expressions(exprs) => {
                for e in exprs.into_iter() {
                    if let ListAttribute::Element(expr_attrs) = e {
                        let mut name = None;
                        let mut data = None;
                        for ea in expr_attrs.into_iter() {
                            match ea {
                                ExpressionAttribute::Name(n) => name = Some(n),
                                ExpressionAttribute::Data(d) => data = Some(d),
                                _ => {}
                            }
                        }
                        if let (Some(name), Some(data)) = (name, data) {
                            expressions.push((name, data));
                        }
                    }
                }
            }
            _ => {}
        }
    }

    let mut dport_reg: Option<u32> = None;
    let mut matches_bootstrap_port = false;
    let mut has_redirect_verdict = false;
    let mut jump_or_goto_target: Option<String> = None;

    for (_name, expr) in expressions.into_iter() {
        match expr {
            Expressions::Payload(p) => {
                let mut base = None;
                let mut offset = None;
                let mut len = None;
                let mut dreg = None;
                for pa in p.into_iter() {
                    match pa {
                        Payload::Base(b) => base = Some(b),
                        Payload::Offset(o) => offset = Some(o),
                        Payload::Len(l) => len = Some(l),
                        Payload::DestinationRegister(Register::Other(d)) => dreg = Some(d),
                        Payload::DestinationRegister(Register::Reg1) => dreg = Some(1),
                        Payload::DestinationRegister(Register::Reg2) => dreg = Some(2),
                        Payload::DestinationRegister(Register::Reg3) => dreg = Some(3),
                        Payload::DestinationRegister(Register::Reg4) => dreg = Some(4),
                        _ => {}
                    }
                }
                // Transport-header offset 2, length 2 is the dport field
                // for both TCP and UDP (sport occupies offset 0).
                // NFT_PAYLOAD_TRANSPORT_HEADER = 2
                if base == Some(2) && offset == Some(2) && len == Some(2) {
                    dport_reg = dreg;
                }
            }
            Expressions::Cmp(c) => {
                let mut sreg = None;
                let mut cmp_data = None;
                for ca in c.into_iter() {
                    match ca {
                        Cmp::SourceRegister(Register::Other(s)) => sreg = Some(s),
                        Cmp::SourceRegister(Register::Reg1) => sreg = Some(1),
                        Cmp::SourceRegister(Register::Reg2) => sreg = Some(2),
                        Cmp::SourceRegister(Register::Reg3) => sreg = Some(3),
                        Cmp::SourceRegister(Register::Reg4) => sreg = Some(4),
                        Cmp::Data(DataAttribute::Value(d)) => cmp_data = Some(d),
                        _ => {}
                    }
                }
                if sreg.is_some()
                    && sreg == dport_reg
                    && let Some(d) = cmp_data
                    && let Some(port) = as_be_uint(&d)
                    && BOOTSTRAP_DNS_PORTS.contains(&port)
                {
                    matches_bootstrap_port = true;
                }
            }
            Expressions::Immediate(i) => {
                for ia in i.into_iter() {
                    if let Immediate::Data(DataAttribute::Verdict(v)) = ia {
                        let mut code = None;
                        let mut target_chain = None;
                        for va in v.into_iter() {
                            match va {
                                VerdictAttribute::Code(c) => {
                                    let c_code: i32 = match c {
                                        Verdict::Jump => -3,
                                        Verdict::Goto => -4,
                                        Verdict::Other(o) => o as i32,
                                        _ => 0,
                                    };
                                    code = Some(c_code);
                                }
                                VerdictAttribute::Chain(c) => {
                                    target_chain = Some(c);
                                }
                                _ => {}
                            }
                        }
                        if code == Some(-3) || code == Some(-4) {
                            // NFT_JUMP = -3, NFT_GOTO = -4
                            jump_or_goto_target = target_chain;
                        }
                    }
                }
            }
            Expressions::Other {
                expression_type,
                attributes,
            } => match expression_type.as_str() {
                "range" => {
                    let mut sreg = None;
                    let mut from_data = None;
                    let mut to_data = None;
                    for attr in attributes.iter() {
                        if attr.kind() == NFTA_RANGE_SREG {
                            sreg = as_be_u32(&attr_value(attr));
                        } else if attr.kind() == NFTA_RANGE_FROM_DATA {
                            from_data = attr_data_value(&attr_value(attr));
                        } else if attr.kind() == NFTA_RANGE_TO_DATA {
                            to_data = attr_data_value(&attr_value(attr));
                        }
                    }
                    if sreg.is_some()
                        && sreg == dport_reg
                        && let (Some(from), Some(to)) = (from_data, to_data)
                    {
                        matches_bootstrap_port |= BOOTSTRAP_DNS_PORTS
                            .iter()
                            .any(|&port| (from..=to).contains(&port));
                    }
                }
                "redir" => has_redirect_verdict = true,
                "nat" => {
                    for attr in attributes.iter() {
                        if attr.kind() == NFTA_NAT_TYPE
                            && as_be_u32(&attr_value(attr)) == Some(NFT_NAT_DNAT)
                        {
                            has_redirect_verdict = true;
                        }
                    }
                }
                "target" => {
                    for attr in attributes.iter() {
                        if attr.kind() == NFTA_TARGET_NAME
                            && let Some(tname) = as_cstr(&attr_value(attr))
                            && (tname.eq_ignore_ascii_case("REDIRECT")
                                || tname.eq_ignore_ascii_case("DNAT"))
                        {
                            has_redirect_verdict = true;
                        }
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }

    Some(RuleInfo {
        key: ChainKey {
            family,
            table: table?,
            name: chain?,
        },
        matches_bootstrap_port,
        has_redirect_verdict,
        jump_or_goto_target,
    })
}

fn as_be_u32(value: &[u8]) -> Option<u32> {
    Some(u32::from_be_bytes(value.try_into().ok()?))
}

fn as_be_uint(value: &[u8]) -> Option<u64> {
    match value.len() {
        1 => Some(value[0] as u64),
        2 => Some(u16::from_be_bytes(value.try_into().ok()?) as u64),
        4 => Some(u32::from_be_bytes(value.try_into().ok()?) as u64),
        8 => Some(u64::from_be_bytes(value.try_into().ok()?)),
        _ => None,
    }
}

fn as_cstr(value: &[u8]) -> Option<&str> {
    let value = value.split(|&byte| byte == 0).next().unwrap_or(value);
    std::str::from_utf8(value).ok()
}

fn attr_value(attr: &impl netlink_packet_core::Nla) -> Vec<u8> {
    let mut buf = vec![0; attr.value_len()];
    attr.emit_value(&mut buf);
    buf
}

fn attr_data_value(buf: &[u8]) -> Option<u64> {
    // Basic NLA parsing for nested NFTA_DATA_VALUE
    if buf.len() < 4 {
        return None;
    }
    let len = u16::from_ne_bytes([buf[0], buf[1]]) as usize;
    let ty = u16::from_ne_bytes([buf[2], buf[3]]) & 0x3FFF;
    if ty == NFTA_DATA_VALUE && len <= buf.len() {
        as_be_uint(&buf[4..len])
    } else {
        None
    }
}

async fn dump<T: Send + 'static>(
    request_msg: NfTablesMessage,
    parse: impl Fn(u8, NfTablesMessage) -> Option<T> + Send + 'static,
) -> Option<Vec<T>> {
    tokio::task::spawn_blocking(move || dump_blocking(request_msg, parse))
        .await
        .inspect_err(|error| {
            tracing::debug!("firewall conflict scan: netlink dump task panicked: {error}")
        })
        .ok()?
}

fn dump_blocking<T>(
    request_msg: NfTablesMessage,
    parse: impl Fn(u8, NfTablesMessage) -> Option<T>,
) -> Option<Vec<T>> {
    let mut socket = Socket::new(NETLINK_NETFILTER)
        .inspect_err(|error| {
            tracing::debug!("firewall conflict scan: failed to open netlink socket: {error}")
        })
        .ok()?;

    socket
        .bind_auto()
        .inspect_err(|error| {
            tracing::debug!("firewall conflict scan: failed to bind netlink socket: {error}")
        })
        .ok()?;

    socket
        .connect(&SocketAddr::new(0, 0))
        .inspect_err(|error| {
            tracing::debug!("firewall conflict scan: failed to connect netlink socket: {error}")
        })
        .ok()?;

    let mut nl_hdr = NetlinkHeader::default();
    nl_hdr.flags = NLM_F_REQUEST | NLM_F_DUMP;
    let mut packet = NetlinkMessage::new(
        nl_hdr,
        NetlinkPayload::from(NetfilterMessage::new(
            NetfilterHeader::new(NetfilterProtoFamily::Unspec, 0, 0),
            NetfilterMessageInner::NfTables(request_msg),
        )),
    );

    packet.finalize();

    let mut buf = vec![0; packet.buffer_len()];
    packet.serialize(&mut buf[..]);

    socket
        .send(&buf[..], 0)
        .inspect_err(|error| {
            tracing::debug!("firewall conflict scan: failed to send netlink request: {error}")
        })
        .ok()?;

    let mut results = Vec::new();
    let mut receive_buffer = vec![0; 32 * 1024];

    loop {
        let size = socket
            .recv(&mut &mut receive_buffer[..], 0)
            .inspect_err(|error| {
                tracing::debug!("firewall conflict scan: netlink recv failed: {error}")
            })
            .ok()?;

        let mut offset = 0;
        let mut done = false;

        loop {
            let bytes = &receive_buffer[offset..];
            let rx_packet = match <NetlinkMessage<NetfilterMessage>>::deserialize(bytes) {
                Ok(p) => p,
                Err(e) => {
                    tracing::debug!("firewall conflict scan: malformed netlink message: {e}");
                    break;
                }
            };

            if matches!(rx_packet.payload, NetlinkPayload::Done(_)) {
                done = true;
                break;
            }
            if matches!(rx_packet.payload, NetlinkPayload::Error(_)) {
                tracing::debug!("firewall conflict scan: netlink returned an error response");
                return None;
            }

            if let NetlinkPayload::InnerMessage(nf_msg) = rx_packet.payload {
                let family = u8::from(nf_msg.header.family);
                if let NetfilterMessageInner::NfTables(nft_msg) = nf_msg.inner
                    && let Some(item) = parse(family, nft_msg)
                {
                    results.push(item);
                }
            }

            offset += rx_packet.header.length as usize;
            if offset == size || rx_packet.header.length == 0 {
                break;
            }
        }

        if done {
            break;
        }
    }

    Some(results)
}
