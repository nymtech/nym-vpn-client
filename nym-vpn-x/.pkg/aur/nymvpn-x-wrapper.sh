#! /bin/bash

# fix an issue with NVIDIA gpu
# https://github.com/nymtech/nym-vpn-client/issues/305
export WEBKIT_DISABLE_DMABUF_RENDERER=1

LOG_FILE=1 RUST_LOG=info,nym_vpn_x=trace /usr/bin/nymvpn-x
