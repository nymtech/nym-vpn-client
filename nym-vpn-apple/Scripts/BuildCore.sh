#!/usr/bin/env bash
set -euo pipefail

# cd to the core repo relative to THIS script's location
cd "$(cd -- "$(dirname "$0")" && pwd)/../../nym-vpn-core"

make -f iOS.mk
make -f macOS.mk
