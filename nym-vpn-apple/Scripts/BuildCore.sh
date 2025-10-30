#!/usr/bin/env bash
set -euo pipefail

# Resolve paths relative to this script
SCRIPT_DIR="$(cd -- "$(dirname "$0")" && pwd)"
APPLE_ROOT="$(cd -- "${SCRIPT_DIR}/.." && pwd)"
CLIENT_ROOT="$(cd -- "${APPLE_ROOT}/.." && pwd)"
CORE_ROOT="$(cd -- "${CLIENT_ROOT}/nym-vpn-core" && pwd)"

echo "[BuildCore] CORE_ROOT=${CORE_ROOT}"
echo "[BuildCore] APPLE_ROOT=${APPLE_ROOT}"
echo "[BuildCore] CLIENT_ROOT=${CLIENT_ROOT}"

# Configure sccache if available
if command -v sccache &>/dev/null; then
  export RUSTC_WRAPPER="$(which sccache)"
  export SCCACHE_DIR="${HOME}/.cache/sccache"
  export SCCACHE_CACHE_SIZE="50G"
  export SCCACHE_IDLE_TIMEOUT="0"
  echo "[BuildCore] Using sccache at ${RUSTC_WRAPPER}"
else
  echo "[BuildCore] ⚠️ sccache not found, skipping cache setup"
fi

# 1) Build iOS
cd "${CORE_ROOT}"
make -f iOS.mk

# 2) Copy NymVPNLib (from nym-vpn-lib-uniffi) → apple repo root
LIB_SRC="${CORE_ROOT}/crates/nym-vpn-lib-uniffi/NymVPNLib"
LIB_DEST="${APPLE_ROOT}/NymVPNLib"
rm -rf "${LIB_DEST}"
cp -R "${LIB_SRC}" "${LIB_DEST}"
echo "[BuildCore] Copied NymVPNLib → ${LIB_DEST}"

# 3) Build macOS (produces upload/mac/nym-vpnd if macOS.mk has vpnd targets)
make -f macOS.mk

# 4) Copy NymVPNRpc (from nym-vpn-rpc-uniffi) → apple repo root
RPC_SRC="${CORE_ROOT}/crates/nym-vpn-rpc-uniffi/NymVPNRpc"
RPC_DEST="${APPLE_ROOT}/NymVPNRpc"
rm -rf "${RPC_DEST}"
cp -R "${RPC_SRC}" "${RPC_DEST}"
echo "[BuildCore] Copied NymVPNRpc → ${RPC_DEST}"

# 5) Copy the universal nym-vpnd → apple Daemon as net.nymtech.vpn.helper
VPND_SRC="${CORE_ROOT}/upload/mac/nym-vpnd"
VPND_DEST_DIR="${APPLE_ROOT}/Daemon"
VPND_DEST="${VPND_DEST_DIR}/net.nymtech.vpn.helper"
if [[ ! -f "${VPND_SRC}" ]]; then
  echo "[BuildCore][ERROR] ${VPND_SRC} not found. Make sure macOS.mk builds vpnd-universal."
  exit 1
fi
mkdir -p "${VPND_DEST_DIR}"
cp -f "${VPND_SRC}" "${VPND_DEST}"
chmod +x "${VPND_DEST}"
echo "[BuildCore] Copied nym-vpnd → ${VPND_DEST}"

# Print sccache stats
if command -v sccache &>/dev/null; then
  echo "[BuildCore] 🧱 sccache stats:"
  sccache --show-stats || true
fi

echo "[BuildCore] ✅ Finished."