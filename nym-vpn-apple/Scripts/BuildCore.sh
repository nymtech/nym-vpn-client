#!/usr/bin/env bash
set -euo pipefail

# Parse flags
RELEASE="true"
while [[ $# -gt 0 ]]; do
  case "$1" in
    --debug)
      RELEASE="false"
      shift
      ;;
    *)
      echo "[BuildCore] Unknown option: $1"
      echo "Usage: $0 [--debug]"
      exit 1
      ;;
  esac
done

if [[ "${RELEASE}" == "true" ]]; then
  echo "[BuildCore] 🚀 Release build — requires code signing."
  echo "[BuildCore] For a debug build, run: $0 --debug"
else
  echo "[BuildCore] 🛠  Debug build"
fi

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
make -f iOS.mk RELEASE="${RELEASE}"

# 2) Copy NymVPNLib (from nym-vpn-lib-uniffi) → apple repo root
LIB_SRC="${CORE_ROOT}/crates/nym-vpn-lib-uniffi/NymVPNLib"
LIB_DEST="${APPLE_ROOT}/NymVPNLib"
rm -rf "${LIB_DEST}"
cp -R "${LIB_SRC}" "${LIB_DEST}"
echo "[BuildCore] Copied NymVPNLib → ${LIB_DEST}"

# 2b) Flatten xcframework headers for Xcode 26+ explicit module builds
XCODE_VER="$(xcodebuild -version 2>/dev/null | head -1 | awk '{print $2}')"
if [[ "$(printf '%s\n' "26.4" "${XCODE_VER}" | sort -V | head -1)" == "26.4" ]]; then
  for HEADERS_DIR in "${LIB_DEST}"/NymVPNLibUniffi.xcframework/*/Headers; do
    for SUBDIR in "${HEADERS_DIR}"/*/; do
      [[ -d "${SUBDIR}" ]] || continue
      cp -n "${SUBDIR}"* "${HEADERS_DIR}/" 2>/dev/null || true
    done
  done
  echo "[BuildCore] Flattened NymVPNLib xcframework headers (Xcode ${XCODE_VER})"
else
  echo "[BuildCore] Skipping header flatten (Xcode ${XCODE_VER} < 26.4)"
fi

# 3) Build macOS (produces upload/mac/nym-vpnd if macOS.mk has vpnd targets)
make -f macOS.mk libwg nym-setup nym-vpnd nym-socks5-proxy rpc-swift-package RELEASE="${RELEASE}"

# 4) Copy NymVPNRpc (from nym-vpn-rpc-uniffi) → apple repo root
RPC_SRC="${CORE_ROOT}/crates/nym-vpn-rpc-uniffi/NymVPNRpc"
RPC_DEST="${APPLE_ROOT}/NymVPNRpc"
rm -rf "${RPC_DEST}"
cp -R "${RPC_SRC}" "${RPC_DEST}"
echo "[BuildCore] Copied NymVPNRpc → ${RPC_DEST}"

# 4b) Flatten xcframework headers for Xcode 26+ explicit module builds
if [[ "$(printf '%s\n' "26.4" "${XCODE_VER}" | sort -V | head -1)" == "26.4" ]]; then
  for HEADERS_DIR in "${RPC_DEST}"/NymVPNRpcUniffi.xcframework/*/Headers; do
    for SUBDIR in "${HEADERS_DIR}"/*/; do
      [[ -d "${SUBDIR}" ]] || continue
      cp -n "${SUBDIR}"* "${HEADERS_DIR}/" 2>/dev/null || true
    done
  done
  echo "[BuildCore] Flattened NymVPNRpc xcframework headers (Xcode ${XCODE_VER})"
else
  echo "[BuildCore] Skipping header flatten (Xcode ${XCODE_VER} < 26.4)"
fi

# 5) Copy the universal nym-vpnd and nym-socks5-proxy → apple Daemon
VPND_SRC="${CORE_ROOT}/upload/mac/nym-vpnd"
SOCKS5_PROXY_SRC="${CORE_ROOT}/upload/mac/nym-socks5-proxy"
VPND_DEST_DIR="${APPLE_ROOT}/NymVPND"
VPND_DEST="${VPND_DEST_DIR}/nym-vpnd"
if [[ ! -f "${VPND_SRC}" ]]; then
  echo "[BuildCore][ERROR] ${VPND_SRC} not found. Make sure macOS.mk builds vpnd-universal."
  exit 1
fi
mkdir -p "${VPND_DEST_DIR}"
cp -f "${VPND_SRC}" "${SOCKS5_PROXY_SRC}" "${VPND_DEST}"
chmod +x "${VPND_DEST}"
echo "[BuildCore] Copied nym-vpnd and nym-socks5-proxy → ${VPND_DEST}"

# 6) Copy the universal nym-setup → apple Daemon
NYM_SETUP_SRC="${CORE_ROOT}/upload/mac/nym-setup"
NYM_SETUP_DEST="${VPND_DEST_DIR}/nym-setup"
if [[ ! -f "${NYM_SETUP_SRC}" ]]; then
  echo "[BuildCore][ERROR] ${NYM_SETUP_SRC} not found. Make sure macOS.mk builds nym-setup-universal."
  exit 1
fi
cp -f "${NYM_SETUP_SRC}" "${NYM_SETUP_DEST}"
chmod +x "${NYM_SETUP_DEST}"
echo "[BuildCore] Copied nym-setup → ${VPND_DEST}"

# Print sccache stats
if command -v sccache &>/dev/null; then
  echo "[BuildCore] 🧱 sccache stats:"
  sccache --show-stats || true
fi

echo "[BuildCore] ✅ Finished."
