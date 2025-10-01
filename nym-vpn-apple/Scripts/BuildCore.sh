#!/usr/bin/env bash
set -euo pipefail

# Resolve paths relative to this script
SCRIPT_DIR="$(cd -- "$(dirname "$0")" && pwd)"
APPLE_ROOT="$(cd -- "${SCRIPT_DIR}/.." && pwd)"
CORE_ROOT="$(cd -- "${APPLE_ROOT}/../nym-vpn-core" && pwd)"

echo "[BuildCore] CORE_ROOT=${CORE_ROOT}"
echo "[BuildCore] APPLE_ROOT=${APPLE_ROOT}"

# 1) Build iOS
cd "${CORE_ROOT}"
make -f iOS.mk

# 2) Copy NymVPNLib (from nym-vpn-lib-uniffi) → apple repo root
LIB_SRC="${CORE_ROOT}/crates/nym-vpn-lib-uniffi/NymVPNLib"
LIB_DEST="${APPLE_ROOT}/NymVPNLib"
rm -rf "${LIB_DEST}"
cp -R "${LIB_SRC}" "${LIB_DEST}"
echo "[BuildCore] Copied NymVPNLib → ${LIB_DEST}"

# 3) Build macOS
make -f macOS.mk

# 4) Copy NymVPNRpc (from nym-vpn-rpc-uniffi) → apple repo root
RPC_SRC="${CORE_ROOT}/crates/nym-vpn-rpc-uniffi/NymVPNRpc"
RPC_DEST="${APPLE_ROOT}/NymVPNRpc"
rm -rf "${RPC_DEST}"
cp -R "${RPC_SRC}" "${RPC_DEST}"
echo "[BuildCore] Copied NymVPNRpc → ${RPC_DEST}"

# 5) Cleanup core repo
cargo clean
echo "[BuildCore] cargo clean done and crates/ removed."

echo "[BuildCore] ✅ Finished."
