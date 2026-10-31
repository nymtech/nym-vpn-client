#!/usr/bin/env bash
# Usage: ./deploy-nm-vpn-plugin.sh [--debug] [--plugin-only] [--no-build] [CORE_DIR]
#   Builds the .deb packages with cargo deb, then installs them from CORE_DIR/target/debian
#   and restarts nym-vpnd / NetworkManager. Run as your normal user; sudo is used for installing.
#   --debug        fast unoptimized build (Cargo "ci" profile: debug build with line-table debug info)
#   --plugin-only  only build and install the NetworkManager plugin (nym-vpnd is left alone)
#   --no-build     skip building and install the newest .deb files already in target/debian
#   CORE_DIR       path to nym-vpn-core (default: the parent of the directory this script is in)
set -euo pipefail

if [[ $EUID -eq 0 ]]; then
  echo "run as your normal user (not sudo): building as root leaves root-owned files in target/"
  exit 1
fi

PLUGIN_ONLY=0
PROFILE=release
BUILD=1
CORE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
for arg in "$@"; do
  case "$arg" in
    --debug) PROFILE=ci ;;
    --plugin-only) PLUGIN_ONLY=1 ;;
    --no-build) BUILD=0 ;;
    -h|--help) sed -n '2,8p' "$0"; exit 0 ;;
    -*) echo "unknown flag: $arg"; exit 1 ;;
    *) CORE_DIR="$arg" ;;
  esac
done

cd "$CORE_DIR"
DEB_DIR="target/debian"

# Debug packages use faster (lighter) compression
DEB_FLAGS=(--no-build --profile "$PROFILE")
[[ $PROFILE == release ]] || DEB_FLAGS+=(--fast)

if (( BUILD )); then
  echo "==> Profile: $PROFILE"
  echo "==> Building NetworkManager plugin"
  cargo build --profile "$PROFILE" -p nym-nm-vpn-plugin
  make -C crates/nym-nm-vpn-plugin/editor
  cargo deb "${DEB_FLAGS[@]}" -p nym-nm-vpn-plugin

  if (( ! PLUGIN_ONLY )); then
    echo "==> Building nym-vpnd"
    # The nym-vpnd package also ships nym-exclude and nym-socks5-proxy, which cargo deb won't build
    cargo build --profile "$PROFILE" -p nym-vpnd -p nym-exclude -p nym-socks5-proxy
    cargo deb "${DEB_FLAGS[@]}" -p nym-vpnd
  fi
fi

[[ -d "$DEB_DIR" ]] || { echo "no such directory: $CORE_DIR/$DEB_DIR"; exit 1; }

# Pick the newest .deb of a package (prints nothing if none found)
latest() { ls -t "$DEB_DIR"/"$1"_*.deb 2>/dev/null | head -n1 || true; }

DEBS=()
NM_DEB=$(latest nym-vpn-networkmanager)
[[ -n "$NM_DEB" ]] || { echo "missing nym-vpn-networkmanager .deb in $CORE_DIR/$DEB_DIR"; exit 1; }
DEBS+=("$(realpath "$NM_DEB")")

if (( ! PLUGIN_ONLY )); then
  VPND_DEB=$(latest nym-vpnd)
  [[ -n "$VPND_DEB" ]] || { echo "missing nym-vpnd .deb in $CORE_DIR/$DEB_DIR"; exit 1; }
  DEBS=("$(realpath "$VPND_DEB")" "${DEBS[@]}")
fi

echo "==> Installing:"; printf '  %s\n' "${DEBS[@]}"

# Stop the tunnel and the old plugin process
sudo nmcli connection down NymVPN 2>/dev/null || true
sudo pkill -f /usr/libexec/nm-nymvpn-service 2>/dev/null || true

# --reinstall so a rebuild with the same version string still replaces the files
sudo apt-get install -y --reinstall --allow-downgrades "${DEBS[@]}"

sudo systemctl daemon-reload
if (( ! PLUGIN_ONLY )); then
  sudo systemctl restart nym-vpnd
fi
# Reload the plugin .so files, the .name file and the NymVPN keyfile
sudo systemctl restart NetworkManager

sleep 2
systemctl --no-pager --lines=0 status nym-vpnd NetworkManager || true
nmcli connection show NymVPN >/dev/null 2>&1 && echo "NymVPN connection present" || echo "NymVPN connection NOT found"
