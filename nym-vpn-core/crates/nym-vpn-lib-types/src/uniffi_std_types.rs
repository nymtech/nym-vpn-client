// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

//! Conversions for common std types and types from external crates.

use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    path::PathBuf,
    str::FromStr,
};

use ipnetwork::{IpNetwork, Ipv4Network, Ipv6Network};
use time::OffsetDateTime;

uniffi::custom_type!(u128, String, {
    remote,
    try_lift: |val| Ok(u128::from_str(&val)?),
    lower: |val| val.to_string()
});

uniffi::custom_type!(PathBuf, String, {
    remote,
    try_lift: |val| Ok(PathBuf::from(val)),
    lower: |val| val.display().to_string()
});

uniffi::custom_type!(Ipv4Addr, String, {
    remote,
    try_lift: |val| Ok(Ipv4Addr::from_str(&val)?),
    lower: |val| val.to_string()
});

uniffi::custom_type!(Ipv6Addr, String, {
    remote,
    try_lift: |val| Ok(Ipv6Addr::from_str(&val)?),
    lower: |val| val.to_string()
});

uniffi::custom_type!(IpAddr, String, {
    remote,
    try_lift: |val| Ok(IpAddr::from_str(&val)?),
    lower: |val| val.to_string()
});

uniffi::custom_type!(SocketAddr, String, {
    remote,
    try_lift: |val| Ok(SocketAddr::from_str(&val)?),
    lower: |val| val.to_string()
});

uniffi::custom_type!(OffsetDateTime, i64, {
    remote,
    try_lift: |val| Ok(OffsetDateTime::from_unix_timestamp(val)?),
    lower: |val| val.unix_timestamp()
});

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
