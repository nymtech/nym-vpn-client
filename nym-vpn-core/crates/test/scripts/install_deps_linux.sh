#!/bin/bash
# Copyright 2016-2025 Mullvad VPN AB. All Rights Reserved.
# Copyright 2025 Nym Technologies SA <contact@nymtech.net>
# SPDX-License-Identifier: GPL-3.0-only

set -eu

# requirements for the host system running test-manager

sudo apt update && sudo apt install -y pkg-config \
    git \
    build-essential \
    pkg-config \
    protobuf-compiler \
    libpcap-dev \
    libssl-dev \
    libdbus-1-dev \
    procps \
    nftables \
    wireguard \
    dnsmasq \
    curl \
    qemu-system \
    qemu-utils \
    swtpm \
    ovmf \
    podman \
    rootlesskit \
    slirp4netns
