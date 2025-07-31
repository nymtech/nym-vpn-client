// Copyright 2023 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

use std::net::{Ipv4Addr, Ipv6Addr};

use nym_ip_packet_requests::IpPair;

uniffi::custom_type!(
    IpPair,
    Vec<u8>, {
        remote,
        try_lift: |val| {
            let bytes: [u8; 20] = val.try_into().expect("Invalid length for IpPair byte representation");
            let ipv4_bytes: [u8; 4] = bytes[0..4].try_into().unwrap();
            let ipv6_bytes: [u8; 16] = bytes[4..20].try_into().unwrap();

            Ok(IpPair {
                ipv4: Ipv4Addr::from(ipv4_bytes),
                ipv6: Ipv6Addr::from(ipv6_bytes)
            })
        },
        lower: |val| {
            val.ipv4.octets().into_iter()
                .chain(val.ipv6.octets().into_iter())
                .collect()
        }
    }
);
