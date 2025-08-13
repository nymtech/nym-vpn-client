// Copyright 2025 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr},
    path::PathBuf,
};

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
