// Copyright 2024 - Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

fn main() -> std::io::Result<()> {
    tonic_prost_build::compile_protos("proto/nym_vpn_service.proto")
}
