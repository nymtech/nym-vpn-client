// Copyright 2026 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Detects a competing nftables NAT rule that redirects NymVPN's bootstrap
//! DNS traffic (DNS-over-TLS/HTTPS to hardcoded Cloudflare/Quad9 servers -
//! see `nym_http_api_client::dns`'s `default_nameserver_group`) before
//! NymVPN's own firewall chain ever evaluates it.
//!
//! Ideally we would have liked to use the nftnl crate to read the nf_tables,
//! however it just supports writing, not reading. Calling the "nft" command
//! is poor as the output may change, so we have to parse the chains and rules
//! ourselves.

use std::collections::{HashMap, HashSet};

use mnl::{Bus, Socket};

use crate::Conflict;

/// Raw netlink/nftables constants this module decodes against. Kept
/// separate from the decoding logic so each one can be checked directly
/// against the kernel headers it's transcribed from.
mod raw {
    // generic netlink (linux/netlink.h)
    pub(super) const NLM_F_REQUEST: u16 = 0x01;
    pub(super) const NLM_F_DUMP: u16 = 0x300; // NLM_F_ROOT | NLM_F_MATCH
    pub(super) const NLMSG_ERROR: u16 = 0x2;
    pub(super) const NLMSG_DONE: u16 = 0x3;
    pub(super) const NLA_TYPE_MASK: u16 = 0x3FFF; // strips NLA_F_NESTED/NLA_F_NET_BYTEORDER

    // netfilter netlink framing (linux/netfilter/nfnetlink.h)
    pub(super) const NFNL_SUBSYS_NFTABLES: u16 = 10;
    pub(super) const NFPROTO_UNSPEC: u8 = 0;
    pub(super) const NFNETLINK_V0: u8 = 0;

    // nftables netlink messages/attributes (linux/netfilter/nf_tables.h)
    pub(super) const NFT_MSG_NEWCHAIN: u16 = 3;
    pub(super) const NFT_MSG_GETCHAIN: u16 = 4;
    pub(super) const NFT_MSG_NEWRULE: u16 = 6;
    pub(super) const NFT_MSG_GETRULE: u16 = 7;

    pub(super) const NFTA_CHAIN_TABLE: u16 = 1;
    pub(super) const NFTA_CHAIN_NAME: u16 = 3;
    pub(super) const NFTA_CHAIN_HOOK: u16 = 4;
    pub(super) const NFTA_CHAIN_TYPE: u16 = 7;
    pub(super) const NFTA_HOOK_HOOKNUM: u16 = 1;

    pub(super) const NFTA_RULE_TABLE: u16 = 1;
    pub(super) const NFTA_RULE_CHAIN: u16 = 2;
    pub(super) const NFTA_RULE_EXPRESSIONS: u16 = 4;

    pub(super) const NFTA_LIST_ELEM: u16 = 1;
    pub(super) const NFTA_EXPR_NAME: u16 = 1;
    pub(super) const NFTA_EXPR_DATA: u16 = 2;

    pub(super) const NFTA_PAYLOAD_DREG: u16 = 1;
    pub(super) const NFTA_PAYLOAD_BASE: u16 = 2;
    pub(super) const NFTA_PAYLOAD_OFFSET: u16 = 3;
    pub(super) const NFTA_PAYLOAD_LEN: u16 = 4;
    /// enum nft_payload_bases - the transport header (where a dport lives,
    /// at byte offset 2, regardless of TCP vs UDP).
    pub(super) const NFT_PAYLOAD_TRANSPORT_HEADER: u32 = 2;

    pub(super) const NFTA_CMP_SREG: u16 = 1;
    pub(super) const NFTA_CMP_DATA: u16 = 3;

    pub(super) const NFTA_RANGE_SREG: u16 = 1;
    pub(super) const NFTA_RANGE_FROM_DATA: u16 = 3;
    pub(super) const NFTA_RANGE_TO_DATA: u16 = 4;

    pub(super) const NFTA_DATA_VALUE: u16 = 1;
    pub(super) const NFTA_DATA_VERDICT: u16 = 2;

    pub(super) const NFTA_IMMEDIATE_DATA: u16 = 2;

    pub(super) const NFTA_VERDICT_CODE: u16 = 1;
    pub(super) const NFTA_VERDICT_CHAIN: u16 = 2;
    /// enum nft_verdicts.
    pub(super) const NFT_JUMP: i32 = -3;
    pub(super) const NFT_GOTO: i32 = -4;

    pub(super) const NFTA_NAT_TYPE: u16 = 1;
    /// enum nft_nat_types.
    pub(super) const NFT_NAT_DNAT: u32 = 1;

    /// enum nf_inet_hooks - the "output" hook (NF_INET_LOCAL_OUT).
    pub(super) const NF_INET_LOCAL_OUT: u32 = 3;

    // xt-compat expression attributes (linux/netfilter/nf_tables_compat.h) -
    // how a rule inserted via the legacy iptables-nft layer (as AdGuard's
    // is) represents its target, since nftables doesn't have a native
    // expression for every iptables extension.
    pub(super) const NFTA_TARGET_NAME: u16 = 1;
}

const OWN_TABLE_NAME: &str = "nym";

const BOOTSTRAP_DNS_PORTS: [u64; 2] = [443, 853];

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
    let chains = dump(raw::NFT_MSG_GETCHAIN, raw::NFT_MSG_NEWCHAIN, parse_chain).await?;
    let rules = dump(raw::NFT_MSG_GETRULE, raw::NFT_MSG_NEWRULE, parse_rule).await?;

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

fn parse_chain(family: u8, attrs_buf: &[u8]) -> Option<ChainInfo> {
    let table = as_cstr(attr(attrs_buf, raw::NFTA_CHAIN_TABLE)?)?.to_string();
    let name = as_cstr(attr(attrs_buf, raw::NFTA_CHAIN_NAME)?)?.to_string();

    let chain_type = attr(attrs_buf, raw::NFTA_CHAIN_TYPE).and_then(as_cstr);
    let hook_num = attr(attrs_buf, raw::NFTA_CHAIN_HOOK)
        .and_then(|hook| attr(hook, raw::NFTA_HOOK_HOOKNUM))
        .and_then(as_be_u32);

    let is_output_nat = chain_type == Some("nat") && hook_num == Some(raw::NF_INET_LOCAL_OUT);

    Some(ChainInfo {
        key: ChainKey {
            family,
            table,
            name,
        },
        is_output_nat,
    })
}

fn parse_rule(family: u8, attrs_buf: &[u8]) -> Option<RuleInfo> {
    let table = as_cstr(attr(attrs_buf, raw::NFTA_RULE_TABLE)?)?.to_string();
    let chain = as_cstr(attr(attrs_buf, raw::NFTA_RULE_CHAIN)?)?.to_string();
    let exprs = attr(attrs_buf, raw::NFTA_RULE_EXPRESSIONS).unwrap_or(&[]);

    let mut dport_reg: Option<u32> = None;
    let mut matches_bootstrap_port = false;
    let mut has_redirect_verdict = false;
    let mut jump_or_goto_target: Option<String> = None;

    for elem in attrs(exprs).filter(|elem| elem.ty == raw::NFTA_LIST_ELEM) {
        let Some(name) = attr(elem.value, raw::NFTA_EXPR_NAME).and_then(as_cstr) else {
            continue;
        };
        let data = attr(elem.value, raw::NFTA_EXPR_DATA).unwrap_or(&[]);

        match name {
            "payload" => {
                let base = attr(data, raw::NFTA_PAYLOAD_BASE).and_then(as_be_u32);
                let offset = attr(data, raw::NFTA_PAYLOAD_OFFSET).and_then(as_be_u32);
                let len = attr(data, raw::NFTA_PAYLOAD_LEN).and_then(as_be_u32);

                // Transport-header offset 2, length 2 is the dport field
                // for both TCP and UDP (sport occupies offset 0).
                if base == Some(raw::NFT_PAYLOAD_TRANSPORT_HEADER)
                    && offset == Some(2)
                    && len == Some(2)
                {
                    dport_reg = attr(data, raw::NFTA_PAYLOAD_DREG).and_then(as_be_u32);
                }
            }
            "cmp" => {
                let sreg = attr(data, raw::NFTA_CMP_SREG).and_then(as_be_u32);
                if sreg.is_some() && sreg == dport_reg {
                    matches_bootstrap_port |= attr(data, raw::NFTA_CMP_DATA)
                        .and_then(|value| attr(value, raw::NFTA_DATA_VALUE))
                        .and_then(as_be_uint)
                        .is_some_and(|port| BOOTSTRAP_DNS_PORTS.contains(&port));
                }
            }
            "range" => {
                let sreg = attr(data, raw::NFTA_RANGE_SREG).and_then(as_be_u32);
                if sreg.is_some() && sreg == dport_reg {
                    let from = attr(data, raw::NFTA_RANGE_FROM_DATA)
                        .and_then(|value| attr(value, raw::NFTA_DATA_VALUE))
                        .and_then(as_be_uint);
                    let to = attr(data, raw::NFTA_RANGE_TO_DATA)
                        .and_then(|value| attr(value, raw::NFTA_DATA_VALUE))
                        .and_then(as_be_uint);
                    if let (Some(from), Some(to)) = (from, to) {
                        matches_bootstrap_port |= BOOTSTRAP_DNS_PORTS
                            .iter()
                            .any(|&port| (from..=to).contains(&port));
                    }
                }
            }
            // Native redirect expression.
            "redir" => has_redirect_verdict = true,
            // Native NAT expression - only DNAT counts as a redirect.
            "nat" => {
                if attr(data, raw::NFTA_NAT_TYPE).and_then(as_be_u32) == Some(raw::NFT_NAT_DNAT) {
                    has_redirect_verdict = true;
                }
            }
            // Opaque xt-compat target (how a rule inserted via the legacy
            // iptables-nft layer - like AdGuard's - represents `REDIRECT`
            // or `DNAT`, since nftables has no native expression for every
            // iptables extension).
            "target" => {
                has_redirect_verdict |= attr(data, raw::NFTA_TARGET_NAME)
                    .and_then(as_cstr)
                    .is_some_and(|name| {
                        name.eq_ignore_ascii_case("REDIRECT") || name.eq_ignore_ascii_case("DNAT")
                    });
            }
            // `jump`/`goto` are represented as an "immediate" expression
            // whose loaded data is a verdict, not a distinct expression
            // type of their own.
            "immediate" => {
                if let Some(verdict) = attr(data, raw::NFTA_IMMEDIATE_DATA)
                    .and_then(|value| attr(value, raw::NFTA_DATA_VERDICT))
                {
                    let code = attr(verdict, raw::NFTA_VERDICT_CODE)
                        .and_then(as_be_u32)
                        .map(|code| code as i32);
                    if code == Some(raw::NFT_JUMP) || code == Some(raw::NFT_GOTO) {
                        jump_or_goto_target = attr(verdict, raw::NFTA_VERDICT_CHAIN)
                            .and_then(as_cstr)
                            .map(str::to_string);
                    }
                }
            }
            _ => {}
        }
    }

    Some(RuleInfo {
        key: ChainKey {
            family,
            table,
            name: chain,
        },
        matches_bootstrap_port,
        has_redirect_verdict,
        jump_or_goto_target,
    })
}

/// Runs a `NLM_F_DUMP` request for `request_type` (e.g.
/// `NFT_MSG_GETCHAIN`), decoding every `response_type` reply (e.g.
/// `NFT_MSG_NEWCHAIN`) with `parse` until the dump completes. Returns `None`
/// if the socket/request/response handling itself fails; a malformed
/// individual chain/rule is simply skipped (via `parse` returning `None`)
/// rather than failing the whole dump.
async fn dump<T: Send + 'static>(
    request_type: u16,
    response_type: u16,
    parse: impl Fn(u8, &[u8]) -> Option<T> + Send + 'static,
) -> Option<Vec<T>> {
    tokio::task::spawn_blocking(move || dump_blocking(request_type, response_type, parse))
        .await
        .inspect_err(|error| {
            tracing::debug!("firewall conflict scan: netlink dump task panicked: {error}")
        })
        .ok()?
}

fn dump_blocking<T>(
    request_type: u16,
    response_type: u16,
    parse: impl Fn(u8, &[u8]) -> Option<T>,
) -> Option<Vec<T>> {
    let socket = Socket::new(Bus::Netfilter)
        .inspect_err(|error| {
            tracing::debug!("firewall conflict scan: failed to open netlink socket: {error}")
        })
        .ok()?;

    socket
        .send(&build_dump_request(request_type))
        .inspect_err(|error| {
            tracing::debug!("firewall conflict scan: failed to send netlink request: {error}")
        })
        .ok()?;

    let mut results = Vec::new();
    let mut buffer = vec![0u8; 32 * 1024];

    loop {
        let messages = socket
            .recv(&mut buffer)
            .inspect_err(|error| {
                tracing::debug!("firewall conflict scan: netlink recv failed: {error}")
            })
            .ok()?;

        let mut done = false;

        for message in messages {
            let message = message
                .inspect_err(|error| {
                    tracing::debug!("firewall conflict scan: malformed netlink message: {error}")
                })
                .ok()?;

            let Some(msg_type) = nlmsg_type(message) else {
                continue;
            };

            if msg_type == raw::NLMSG_DONE {
                done = true;
                break;
            }
            if msg_type == raw::NLMSG_ERROR {
                tracing::debug!("firewall conflict scan: netlink returned an error response");
                return None;
            }

            let subsys = msg_type >> 8;
            let op = msg_type & 0x00FF;
            if subsys != raw::NFNL_SUBSYS_NFTABLES || op != response_type {
                continue;
            }

            let Some(body) = nlmsg_body(message) else {
                continue;
            };
            // nfgenmsg: family (1 byte) + version (1 byte) + res_id (2
            // bytes), followed by the attribute list.
            if body.len() < 4 {
                continue;
            }
            let family = body[0];

            if let Some(item) = parse(family, &body[4..]) {
                results.push(item);
            }
        }

        if done {
            break;
        }
    }

    Some(results)
}

/// Builds a bare `NLM_F_DUMP` request for `msg_type` - just the
/// `nlmsghdr` + `nfgenmsg` headers, no filtering attributes, matching every
/// object of that kind.
fn build_dump_request(msg_type: u16) -> [u8; 20] {
    let mut msg = [0u8; 20];

    let nlmsg_len: u32 = msg.len() as u32;
    let nlmsg_type: u16 = (raw::NFNL_SUBSYS_NFTABLES << 8) | msg_type;
    let nlmsg_flags: u16 = raw::NLM_F_REQUEST | raw::NLM_F_DUMP;

    msg[0..4].copy_from_slice(&nlmsg_len.to_ne_bytes());
    msg[4..6].copy_from_slice(&nlmsg_type.to_ne_bytes());
    msg[6..8].copy_from_slice(&nlmsg_flags.to_ne_bytes());
    // nlmsg_seq (8..12) and nlmsg_pid (12..16) are left zeroed - this
    // module only ever has one outstanding request per socket, so there's
    // nothing to disambiguate against.
    msg[16] = raw::NFPROTO_UNSPEC;
    msg[17] = raw::NFNETLINK_V0;
    // res_id (18..20) is left zeroed - unused for a plain dump-everything
    // request.

    msg
}

fn nlmsg_type(message: &[u8]) -> Option<u16> {
    Some(u16::from_ne_bytes(message.get(4..6)?.try_into().ok()?))
}

/// The message body after the 16-byte `nlmsghdr`.
fn nlmsg_body(message: &[u8]) -> Option<&[u8]> {
    message.get(16..)
}

/// One netlink attribute: its (flag-masked) type and raw value bytes.
struct Attr<'a> {
    ty: u16,
    value: &'a [u8],
}

/// Walks a netlink attribute (TLV) list, respecting 4-byte alignment and
/// masking off the `NLA_F_NESTED`/`NLA_F_NET_BYTEORDER` flag bits each
/// attribute's type may carry.
fn attrs(buf: &[u8]) -> impl Iterator<Item = Attr<'_>> {
    struct Attrs<'a>(&'a [u8]);

    impl<'a> Iterator for Attrs<'a> {
        type Item = Attr<'a>;

        fn next(&mut self) -> Option<Attr<'a>> {
            const HEADER_LEN: usize = 4;
            if self.0.len() < HEADER_LEN {
                self.0 = &[];
                return None;
            }

            let len = u16::from_ne_bytes([self.0[0], self.0[1]]) as usize;
            let ty = u16::from_ne_bytes([self.0[2], self.0[3]]) & raw::NLA_TYPE_MASK;

            if len < HEADER_LEN || len > self.0.len() {
                self.0 = &[];
                return None;
            }

            let value = &self.0[HEADER_LEN..len];
            let padded = len.next_multiple_of(4).min(self.0.len());
            self.0 = &self.0[padded..];

            Some(Attr { ty, value })
        }
    }

    Attrs(buf)
}

fn attr(buf: &[u8], ty: u16) -> Option<&[u8]> {
    attrs(buf).find(|attr| attr.ty == ty).map(|attr| attr.value)
}

/// Interprets `value` as a NUL-terminated (or unterminated) UTF-8 string,
/// as used for `NLA_STRING`/`NLA_NUL_STRING` attributes.
fn as_cstr(value: &[u8]) -> Option<&str> {
    let value = value.split(|&byte| byte == 0).next().unwrap_or(value);
    std::str::from_utf8(value).ok()
}

fn as_be_u32(value: &[u8]) -> Option<u32> {
    Some(u32::from_be_bytes(value.try_into().ok()?))
}

/// Interprets `value` as a big-endian integer of whatever width it is (1,
/// 2, 4, or 8 bytes) - used for comparison-data values (e.g. a port),
/// which are exactly as wide as the field they're compared against, not a
/// fixed width.
fn as_be_uint(value: &[u8]) -> Option<u64> {
    match value.len() {
        1 => Some(value[0] as u64),
        2 => Some(u16::from_be_bytes(value.try_into().ok()?) as u64),
        4 => Some(u32::from_be_bytes(value.try_into().ok()?) as u64),
        8 => Some(u64::from_be_bytes(value.try_into().ok()?)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Builds one netlink attribute: a 4-byte header (length, type) plus
    /// `value`, padded to a 4-byte boundary.
    fn build_attr(ty: u16, value: &[u8]) -> Vec<u8> {
        let len = (4 + value.len()) as u16;
        let mut out = Vec::new();
        out.extend_from_slice(&len.to_ne_bytes());
        out.extend_from_slice(&ty.to_ne_bytes());
        out.extend_from_slice(value);
        while out.len() % 4 != 0 {
            out.push(0);
        }
        out
    }

    fn u32_attr(ty: u16, value: u32) -> Vec<u8> {
        build_attr(ty, &value.to_be_bytes())
    }

    fn str_attr(ty: u16, value: &str) -> Vec<u8> {
        let mut bytes = value.as_bytes().to_vec();
        bytes.push(0);
        build_attr(ty, &bytes)
    }

    /// A nested attribute: `ty`'s value is itself the concatenation of
    /// `inner`'s attributes.
    fn nested_attr(ty: u16, inner: &[Vec<u8>]) -> Vec<u8> {
        build_attr(ty, &inner.concat())
    }

    fn data_value_attr(ty: u16, port: u16) -> Vec<u8> {
        nested_attr(ty, &[build_attr(raw::NFTA_DATA_VALUE, &port.to_be_bytes())])
    }

    /// A `payload` expression list element loading the dport field (2
    /// bytes at transport-header offset 2) into `dreg`.
    fn dport_payload_expr(dreg: u32) -> Vec<u8> {
        let data = nested_attr(
            raw::NFTA_EXPR_DATA,
            &[
                u32_attr(raw::NFTA_PAYLOAD_DREG, dreg),
                u32_attr(raw::NFTA_PAYLOAD_BASE, raw::NFT_PAYLOAD_TRANSPORT_HEADER),
                u32_attr(raw::NFTA_PAYLOAD_OFFSET, 2),
                u32_attr(raw::NFTA_PAYLOAD_LEN, 2),
            ],
        );
        nested_attr(
            raw::NFTA_LIST_ELEM,
            &[str_attr(raw::NFTA_EXPR_NAME, "payload"), data],
        )
    }

    /// A `range` expression list element testing `sreg` against
    /// `from..=to`.
    fn range_expr(sreg: u32, from: u16, to: u16) -> Vec<u8> {
        let data = nested_attr(
            raw::NFTA_EXPR_DATA,
            &[
                u32_attr(raw::NFTA_RANGE_SREG, sreg),
                data_value_attr(raw::NFTA_RANGE_FROM_DATA, from),
                data_value_attr(raw::NFTA_RANGE_TO_DATA, to),
            ],
        );
        nested_attr(
            raw::NFTA_LIST_ELEM,
            &[str_attr(raw::NFTA_EXPR_NAME, "range"), data],
        )
    }

    /// A `cmp` expression list element testing `sreg` against a single
    /// port.
    fn cmp_port_expr(sreg: u32, port: u16) -> Vec<u8> {
        let data = nested_attr(
            raw::NFTA_EXPR_DATA,
            &[
                u32_attr(raw::NFTA_CMP_SREG, sreg),
                data_value_attr(raw::NFTA_CMP_DATA, port),
            ],
        );
        nested_attr(
            raw::NFTA_LIST_ELEM,
            &[str_attr(raw::NFTA_EXPR_NAME, "cmp"), data],
        )
    }

    /// An xt-compat `target` expression list element (how a rule inserted
    /// via the legacy iptables-nft layer - like AdGuard's REDIRECT -
    /// represents its verdict).
    fn xt_target_expr(name: &str) -> Vec<u8> {
        let data = nested_attr(
            raw::NFTA_EXPR_DATA,
            &[str_attr(raw::NFTA_TARGET_NAME, name)],
        );
        nested_attr(
            raw::NFTA_LIST_ELEM,
            &[str_attr(raw::NFTA_EXPR_NAME, "target"), data],
        )
    }

    /// A native `redirect` expression list element.
    fn native_redirect_expr() -> Vec<u8> {
        nested_attr(
            raw::NFTA_LIST_ELEM,
            &[
                str_attr(raw::NFTA_EXPR_NAME, "redir"),
                nested_attr(raw::NFTA_EXPR_DATA, &[]),
            ],
        )
    }

    /// An `immediate` expression list element carrying a jump/goto verdict
    /// to `target_chain`.
    fn jump_expr(code: i32, target_chain: &str) -> Vec<u8> {
        let verdict = nested_attr(
            raw::NFTA_DATA_VERDICT,
            &[
                u32_attr(raw::NFTA_VERDICT_CODE, code as u32),
                str_attr(raw::NFTA_VERDICT_CHAIN, target_chain),
            ],
        );
        let immediate_data = nested_attr(raw::NFTA_IMMEDIATE_DATA, &[verdict]);
        let data = nested_attr(raw::NFTA_EXPR_DATA, &[immediate_data]);
        nested_attr(
            raw::NFTA_LIST_ELEM,
            &[str_attr(raw::NFTA_EXPR_NAME, "immediate"), data],
        )
    }

    fn chain_attrs(
        table: &str,
        name: &str,
        chain_type: Option<&str>,
        hook_num: Option<u32>,
    ) -> Vec<u8> {
        let mut out = vec![
            str_attr(raw::NFTA_CHAIN_TABLE, table),
            str_attr(raw::NFTA_CHAIN_NAME, name),
        ];
        if let Some(chain_type) = chain_type {
            out.push(str_attr(raw::NFTA_CHAIN_TYPE, chain_type));
        }
        if let Some(hook_num) = hook_num {
            out.push(nested_attr(
                raw::NFTA_CHAIN_HOOK,
                &[u32_attr(raw::NFTA_HOOK_HOOKNUM, hook_num)],
            ));
        }
        out.concat()
    }

    fn rule_attrs(table: &str, chain: &str, exprs: &[Vec<u8>]) -> Vec<u8> {
        [
            str_attr(raw::NFTA_RULE_TABLE, table),
            str_attr(raw::NFTA_RULE_CHAIN, chain),
            nested_attr(raw::NFTA_RULE_EXPRESSIONS, exprs),
        ]
        .concat()
    }

    #[test]
    fn parses_a_chain_hooked_at_output_with_type_nat() {
        let attrs = chain_attrs("nat", "OUTPUT", Some("nat"), Some(raw::NF_INET_LOCAL_OUT));
        let chain = parse_chain(2, &attrs).unwrap();
        assert!(chain.is_output_nat);
        assert_eq!(chain.key.table, "nat");
        assert_eq!(chain.key.name, "OUTPUT");
    }

    #[test]
    fn ignores_a_nat_chain_not_on_the_output_hook() {
        // NF_INET_POST_ROUTING = 4, not the output hook.
        let attrs = chain_attrs("nat", "POSTROUTING", Some("nat"), Some(4));
        let chain = parse_chain(2, &attrs).unwrap();
        assert!(!chain.is_output_nat);
    }

    #[test]
    fn ignores_a_filter_chain_on_the_output_hook() {
        let attrs = chain_attrs("nym", "output", None, Some(raw::NF_INET_LOCAL_OUT));
        let chain = parse_chain(1, &attrs).unwrap();
        assert!(!chain.is_output_nat);
    }

    #[test]
    fn detects_a_compat_redirect_rule_reached_via_a_range_match() {
        // Mirrors AdGuard's real rule: `tcp dport 80-5221 ... REDIRECT`,
        // inserted through the legacy iptables-nft compat layer.
        let exprs = [
            dport_payload_expr(1),
            range_expr(1, 80, 5221),
            xt_target_expr("REDIRECT"),
        ];
        let rule = parse_rule(2, &rule_attrs("nat", "AGCLI", &exprs)).unwrap();
        assert!(rule.matches_bootstrap_port);
        assert!(rule.has_redirect_verdict);
    }

    #[test]
    fn detects_a_native_redirect_rule_reached_via_a_cmp_match() {
        let exprs = [
            dport_payload_expr(1),
            cmp_port_expr(1, 853),
            native_redirect_expr(),
        ];
        let rule = parse_rule(2, &rule_attrs("other", "out", &exprs)).unwrap();
        assert!(rule.matches_bootstrap_port);
        assert!(rule.has_redirect_verdict);
    }

    #[test]
    fn ignores_a_redirect_that_does_not_cover_a_bootstrap_port() {
        let exprs = [
            dport_payload_expr(1),
            cmp_port_expr(1, 8080),
            xt_target_expr("REDIRECT"),
        ];
        let rule = parse_rule(2, &rule_attrs("nat", "AGCLI", &exprs)).unwrap();
        assert!(!rule.matches_bootstrap_port);
    }

    #[test]
    fn ignores_a_bootstrap_port_match_with_no_redirect_verdict() {
        let exprs = [dport_payload_expr(1), cmp_port_expr(1, 853)];
        let rule = parse_rule(2, &rule_attrs("other", "out", &exprs)).unwrap();
        assert!(rule.matches_bootstrap_port);
        assert!(!rule.has_redirect_verdict);
    }

    #[test]
    fn does_not_correlate_a_cmp_on_an_unrelated_register() {
        // The dport payload loads register 1, but the cmp tests register 2
        // (e.g. some other field entirely) - these must not be treated as
        // testing the same thing just because a port-shaped value appears.
        let exprs = [dport_payload_expr(1), cmp_port_expr(2, 853)];
        let rule = parse_rule(2, &rule_attrs("other", "out", &exprs)).unwrap();
        assert!(!rule.matches_bootstrap_port);
    }

    #[test]
    fn extracts_a_jump_target_from_an_immediate_verdict() {
        let exprs = [jump_expr(raw::NFT_JUMP, "AGCLI")];
        let rule = parse_rule(2, &rule_attrs("nat", "OUTPUT", &exprs)).unwrap();
        assert_eq!(rule.jump_or_goto_target.as_deref(), Some("AGCLI"));
    }

    #[test]
    fn extracts_a_goto_target_from_an_immediate_verdict() {
        let exprs = [jump_expr(raw::NFT_GOTO, "AGCLI")];
        let rule = parse_rule(2, &rule_attrs("nat", "OUTPUT", &exprs)).unwrap();
        assert_eq!(rule.jump_or_goto_target.as_deref(), Some("AGCLI"));
    }

    /// End-to-end over parsed records (not raw netlink bytes): mirrors
    /// AdGuard's real ruleset - a thin `nat`/`output` base chain
    /// (`OUTPUT`) that just jumps into a separate regular chain (`AGCLI`)
    /// holding the actual redirect rule.
    #[test]
    fn full_traversal_detects_a_conflict_reached_via_a_jump() {
        let chains = vec![
            parse_chain(
                2,
                &chain_attrs("nat", "OUTPUT", Some("nat"), Some(raw::NF_INET_LOCAL_OUT)),
            )
            .unwrap(),
        ];
        let dispatch_rule = parse_rule(
            2,
            &rule_attrs("nat", "OUTPUT", &[jump_expr(raw::NFT_JUMP, "AGCLI")]),
        )
        .unwrap();
        let redirect_rule = parse_rule(
            2,
            &rule_attrs(
                "nat",
                "AGCLI",
                &[
                    dport_payload_expr(1),
                    range_expr(1, 80, 5221),
                    xt_target_expr("REDIRECT"),
                ],
            ),
        )
        .unwrap();

        let mut rules_by_chain: HashMap<ChainKey, Vec<RuleInfo>> = HashMap::new();
        for rule in [dispatch_rule, redirect_rule] {
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
        let mut visited = HashSet::new();
        let mut found = false;

        while let Some(key) = to_visit.pop() {
            if !visited.insert(key.clone()) {
                continue;
            }
            for rule in rules_by_chain.get(&key).into_iter().flatten() {
                if rule.matches_bootstrap_port && rule.has_redirect_verdict {
                    found = true;
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

        assert!(found);
    }

    #[tokio::test]
    async fn manual_smoke_test_against_the_real_system() {
        println!("scan(): {:?}", detect().await);
    }
}
