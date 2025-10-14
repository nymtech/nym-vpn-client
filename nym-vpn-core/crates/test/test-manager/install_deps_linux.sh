#!/bin/bash
# Copyright 2016-2025 Mullvad VPN AB. All Rights Reserved.
# Copyright 2025 Nym Technologies SA <contact@nymtech.net>
# SPDX-License-Identifier: GPL-3.0-only

set -eu

# based on Debian requirements
sudo apt update && sudo apt install -y \
    git build-essential pkg-config libpcap-dev libssl-dev protobuf-compiler rootlesskit procps nftables wireguard dnsmasq curl

sudo apt install qemu-system qemu-utils podman
