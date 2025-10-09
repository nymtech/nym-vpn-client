#!/bin/bash

set -eu

# based on Debian requirements
sudo apt update && sudo apt install -y \
    git build-essential pkg-config libpcap-dev libssl-dev protobuf-compiler rootlesskit procps nftables wireguard dnsmasq curl

sudo apt install qemu-system qemu-utils podman
