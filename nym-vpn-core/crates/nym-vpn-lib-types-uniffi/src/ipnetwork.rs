// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use ipnetwork::{IpNetwork, Ipv4Network, Ipv6Network};

uniffi::custom_type!(IpNetwork, String, {
    remote,
    try_lift: |val| Ok(IpNetwork::from_str(&val)?),
    lower: |val| val.to_string()
});

uniffi::custom_type!(Ipv4Network, String, {
    remote,
    try_lift: |val| Ok(Ipv4Network::from_str(&val)?),
    lower: |val| val.to_string()
});

uniffi::custom_type!(Ipv6Network, String, {
    remote,
    try_lift: |val| Ok(Ipv6Network::from_str(&val)?),
    lower: |val| val.to_string()
});
