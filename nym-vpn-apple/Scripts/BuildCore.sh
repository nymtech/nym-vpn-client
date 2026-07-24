#!/usr/bin/env bash
set -euo pipefail

RELEASE="${RELEASE:-true}"
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

# Santa's menu runtime gate (isSantaClaus reads IsCiBuild). A debug build flips
# it to true so the menu is reachable locally; a release build restores the
# $(IS_CI_BUILD) build-setting placeholder so the tracked plist stays clean.
# (The compile-time #if SANTA is handled by Package.swift: debug config defines
# it locally, ship Release-without-NYM_SANTA strips it.)
DAEMON_PLIST="${APPLE_ROOT}/NymVPNDaemon/Resources/Info.plist"
if [[ -f "${DAEMON_PLIST}" ]]; then
  if [[ "${RELEASE}" == "false" ]]; then
    # String, not bool: isRunningOnCI reads this via `as? String` (a bool cast
    # fails → Santa silently off). Tracked plist + CI Set both keep it a string.
    /usr/libexec/PlistBuddy -c "Set :IsCiBuild true" "${DAEMON_PLIST}" 2>/dev/null \
      || /usr/libexec/PlistBuddy -c "Add :IsCiBuild string true" "${DAEMON_PLIST}"
    echo "[BuildCore] 🎅 Santa's menu enabled (IsCiBuild=true)"
  else
    /usr/libexec/PlistBuddy -c 'Set :IsCiBuild $(IS_CI_BUILD)' "${DAEMON_PLIST}" 2>/dev/null || true
    echo "[BuildCore] Santa's menu plist flag restored to \$(IS_CI_BUILD)"
  fi
else
  echo "[BuildCore] ⚠️ ${DAEMON_PLIST} not found, skipping Santa plist flag"
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

# 3) Build macOS (produces upload/mac/nym-vpnd if macOS.mk has vpnd targets)
make -f macOS.mk libwg nym-setup nym-vpnd nym-socks5-proxy rpc-swift-package RELEASE="${RELEASE}"

# 4) Copy NymVPNRpc (from nym-vpn-rpc-uniffi) → apple repo root
RPC_SRC="${CORE_ROOT}/crates/nym-vpn-rpc-uniffi/NymVPNRpc"
RPC_DEST="${APPLE_ROOT}/NymVPNRpc"
rm -rf "${RPC_DEST}"
cp -R "${RPC_SRC}" "${RPC_DEST}"
echo "[BuildCore] Copied NymVPNRpc → ${RPC_DEST}"

# 5) Copy binaries to apple Daemon folder
VPND_SRC_DIR="${CORE_ROOT}/upload/mac"
VPND_DEST_DIR="${APPLE_ROOT}/NymVPND"
mkdir -p "${VPND_DEST_DIR}"
for f in nym-vpnd nym-socks5-proxy nym-setup; do
  if [[ ! -f "${VPND_SRC_DIR}/${f}" ]]; then
    echo "[BuildCore][ERROR] ${VPND_SRC_DIR}/${f} not found. Make sure macOS.mk builds vpnd-universal."
    exit 1
  fi
  cp -f "${VPND_SRC_DIR}/${f}" "${VPND_DEST_DIR}"
  chmod +x "${VPND_DEST_DIR}/${f}"
  echo "[BuildCore] Copied ${f} → ${VPND_DEST_DIR}"
done

# Print sccache stats
if command -v sccache &>/dev/null; then
  echo "[BuildCore] 🧱 sccache stats:"
  sccache --show-stats || true
fi

echo "[BuildCore] ✅ Finished."
