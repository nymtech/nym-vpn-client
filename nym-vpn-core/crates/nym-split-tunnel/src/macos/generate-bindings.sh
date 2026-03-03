#!/usr/bin/env bash

# This script generates bindings for certain pcap and pktap symbols.
# bindgen is required: cargo install bindgen-cli

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

curl https://raw.githubusercontent.com/apple-oss-distributions/xnu/refs/tags/xnu-3789.41.3/bsd/net/pktap.h -o include/pktap.h
curl https://raw.githubusercontent.com/apple-opensource/libpcap/refs/tags/67/libpcap/pcap.h -o include/pcap.h
curl https://raw.githubusercontent.com/apple-oss-distributions/xnu/refs/tags/xnu-3789.41.3/bsd/net/bpf.h -o include/bpf.h

bindgen "include/bindings.h" -o ./bindings.rs \
    --allowlist-item "^pktap_header" \
    --allowlist-item "^PTH_FLAG_DIR_OUT"
