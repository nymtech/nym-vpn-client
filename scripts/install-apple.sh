#!/usr/bin/env bash
# Local Debug build for nym-vpn-apple (macOS app or a plugged-in iPhone).
#
# --macos:
#   make -C nym-vpn-core -f macOS.mk swift-package RELEASE=false
#   stage NymVPNLib, xcodebuild NymVPNDaemon
#   Debug .app does not embed nym-vpnd. Helper: nym-vpn-apple/NymVPND/nym-vpnd
#
# --ios:
#   make -C nym-vpn-core -f iOS.mk swift-package RUST_TRIPLET=aarch64-apple-ios
#   stage NymVPNLib (ios-arm64), xcodebuild NymVPN, install via devicectl
#
# Staging one slice replaces the other. Build the platform you will run next.
#
# Usage:
#   ./scripts/install-apple.sh --macos --open
#   ./scripts/install-apple.sh --ios --open
#   make -C nym-vpn-apple install-macos OPEN=1
#   make -C nym-vpn-apple install-ios OPEN=1
#
# Disconnect VPN first (Xcode / device install break while a tunnel is up).

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

SKIP_CORE=0
REBUILD_CORE=0
CORE_ONLY=0
SKIP_APP=0
OPEN_APP=0
START_DAEMON=0
SKIP_DAEMON=0
SKIP_INSTALL=0
DRY_RUN=0
RELEASE=false
ARCH=arm64
PLATFORM=""
DEVICE_UDID=""
IOS_BUNDLE_ID=net.nymtech.vpn

usage() {
  cat <<'EOF'
install-apple.sh - local Debug build for nym-vpn-apple

Pass exactly one of --macos or --ios.
macOS: stage a macOS UniFFI slice and xcodebuild NymVPNDaemon.
iOS: stage ios-arm64 UniFFI, xcodebuild NymVPN, install on a phone.

The Debug Mac .app does not contain the LaunchDaemon. Helper:

  nym-vpn-apple/NymVPND/nym-vpnd

RPC socket: /var/run/nym-vpn.sock
System job (store install): net.nymtech.vpn.daemon

Options:
  --macos           Build the Mac app (scheme NymVPNDaemon)
  --ios             Build and install the iPhone app (scheme NymVPN)
  --udid ID         iOS device UDID (xcrun xctrace list devices)
  --skip-install    iOS: build only; do not install
  --skip-core       Do not rebuild UniFFI; fail if the needed slice is missing
  --rebuild-core    Always rebuild and restage NymVPNLib
  --core-only       Stage NymVPNLib, then exit (no xcodebuild)
  --skip-app        Stage core only if needed; do not xcodebuild
  --open            macOS: start helper + open .app. iOS: launch after install
  --start-daemon    macOS: unload the store helper and start NymVPND/nym-vpnd
  --skip-daemon     macOS: do not start the helper (overrides --open)
  --arch NAME       macOS only: arm64 (default), x86_64, or fat
  --release         cargo swift --release (slow; default is debug)
  --dry-run         Print the plan; change nothing
  -h, --help        Show this help

Staging replaces nym-vpn-apple/NymVPNLib. After --ios, a Mac run needs
--macos (it rebuilds the macos slice if that slice is gone).
EOF
}

log() { printf '[apple-%s] %s\n' "$PLATFORM" "$*"; }
die() { printf '[apple-%s] ERROR: %s\n' "$PLATFORM" "$*" >&2; exit 1; }

have() { command -v "$1" >/dev/null 2>&1; }

run() {
  if [[ "$DRY_RUN" -eq 1 ]]; then
    printf '[apple-%s] dry-run: %s\n' "$PLATFORM" "$*"
    return 0
  fi
  "$@"
}

parse_args() {
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --skip-core) SKIP_CORE=1; shift ;;
      --rebuild-core) REBUILD_CORE=1; shift ;;
      --core-only) CORE_ONLY=1; shift ;;
      --skip-app) SKIP_APP=1; shift ;;
      --ios) PLATFORM=ios; shift ;;
      --macos) PLATFORM=macos; shift ;;
      --udid|--device)
        [[ $# -ge 2 ]] || die "--udid requires a device identifier"
        DEVICE_UDID="$2"
        shift 2
        ;;
      --skip-install) SKIP_INSTALL=1; shift ;;
      --open) OPEN_APP=1; shift ;;
      --start-daemon) START_DAEMON=1; shift ;;
      --skip-daemon) SKIP_DAEMON=1; shift ;;
      --arch)
        [[ $# -ge 2 ]] || die "--arch requires arm64, x86_64, or fat"
        ARCH="$2"
        shift 2
        ;;
      --release) RELEASE=true; shift ;;
      --dry-run) DRY_RUN=1; shift ;;
      -h|--help) usage; exit 0 ;;
      *) die "Unknown option: $1 (see --help)" ;;
    esac
  done
  case "$ARCH" in
    arm64|x86_64|fat) ;;
    *) die "Unsupported --arch '$ARCH' (use arm64, x86_64, or fat)" ;;
  esac
  case "$PLATFORM" in
    ios|macos) ;;
    *) die "Pass --ios or --macos (see --help)" ;;
  esac
  if [[ "$SKIP_CORE" -eq 1 && "$REBUILD_CORE" -eq 1 ]]; then
    die "Use either --skip-core or --rebuild-core"
  fi
  if [[ "$PLATFORM" == macos && "$OPEN_APP" -eq 1 && "$SKIP_DAEMON" -eq 0 ]]; then
    START_DAEMON=1
  fi
  if [[ "$SKIP_DAEMON" -eq 1 ]]; then
    START_DAEMON=0
  fi
  if [[ "$PLATFORM" == ios && "$START_DAEMON" -eq 1 ]]; then
    die "--start-daemon is macOS only"
  fi
}

daemon_bin() {
  printf '%s/nym-vpn-apple/NymVPND/nym-vpnd' "$REPO_ROOT"
}

daemon_upload_bin() {
  printf '%s/nym-vpn-core/upload/mac/nym-vpnd' "$REPO_ROOT"
}

daemon_log() {
  printf '%s/nym-vpn-apple/build/nym-vpnd.local.log' "$REPO_ROOT"
}

print_daemon_howto() {
  local bin
  bin="$(daemon_bin)"
  log "Daemon binary: $bin"
  log "RPC socket: /var/run/nym-vpn.sock"
  log "Start local helper (unloads the store LaunchDaemon first):"
  log "  sudo launchctl bootout system/net.nymtech.vpn.daemon || true"
  log "  sudo env RUST_LOG=debug \"$bin\" -v run-with-args --disable-client-verification"
  log "Or keep the store helper: sudo launchctl kickstart -k system/net.nymtech.vpn.daemon"
}

ensure_daemon_bin() {
  local dest upload
  dest="$(daemon_bin)"
  upload="$(daemon_upload_bin)"

  if [[ "$DRY_RUN" -eq 1 ]]; then
    log "dry-run: ensure $dest"
    return 0
  fi

  if [[ -x "$dest" ]]; then
    log "Using existing $dest"
    return 0
  fi

  if [[ ! -x "$upload" ]]; then
    log "Building nym-vpnd (RELEASE=$RELEASE ARCH=$ARCH)"
    run make -C "$REPO_ROOT/nym-vpn-core" -f macOS.mk nym-vpnd \
      RELEASE="$RELEASE" ARCH="$ARCH"
  fi

  [[ -x "$upload" ]] || die "macOS.mk did not produce $upload"
  log "Staging $upload -> $dest"
  cp "$upload" "$dest"
  chmod +x "$dest"
}

start_local_daemon() {
  local bin log_file
  bin="$(daemon_bin)"
  log_file="$(daemon_log)"

  [[ "$DRY_RUN" -eq 1 || -x "$bin" ]] || die "Missing $bin"

  log "Unloading store helper system/net.nymtech.vpn.daemon"
  if [[ "$DRY_RUN" -eq 1 ]]; then
    log "dry-run: sudo launchctl bootout system/net.nymtech.vpn.daemon"
    log "dry-run: sudo env RUST_LOG=debug $bin -v run-with-args --disable-client-verification"
    return 0
  fi

  sudo launchctl bootout system/net.nymtech.vpn.daemon 2>/dev/null || true
  if pgrep -x nym-vpnd >/dev/null 2>&1; then
    sudo killall nym-vpnd 2>/dev/null || true
    sleep 1
  fi

  mkdir -p "$(dirname "$log_file")"
  log "Starting $bin (log $log_file)"
  sudo env RUST_LOG=debug "$bin" -v run-with-args --disable-client-verification \
    >>"$log_file" 2>&1 &
  local pid=$!
  sleep 1
  if ! kill -0 "$pid" 2>/dev/null; then
    die "nym-vpnd exited immediately; see $log_file"
  fi
  log "nym-vpnd pid=$pid"
  if [[ -S /var/run/nym-vpn.sock ]]; then
    log "RPC socket ready: /var/run/nym-vpn.sock"
  else
    log "RPC socket not up yet; the app retries every 5s"
  fi
}

find_repo_root() {
  local dir="$PWD"
  local candidate
  for candidate in "$SCRIPT_DIR/.." "$PWD" "$PWD/.." "$PWD/../.."; do
    if [[ -d "$candidate/nym-vpn-apple" && -d "$candidate/nym-vpn-core" ]]; then
      (cd "$candidate" && pwd)
      return 0
    fi
  done
  while [[ "$dir" != "/" ]]; do
    if [[ -d "$dir/nym-vpn-apple" && -d "$dir/nym-vpn-core" ]]; then
      printf '%s\n' "$dir"
      return 0
    fi
    dir="$(dirname "$dir")"
  done
  die "Could not find nym-vpn-client root (need nym-vpn-apple/ and nym-vpn-core/)"
}

xcframework_plist() {
  printf '%s/NymVPNLibUniffi.xcframework/Info.plist' "$1"
}

has_macos_slice() {
  local plist
  plist="$(xcframework_plist "$1")"
  [[ -f "$plist" ]] || return 1
  grep -q '<string>macos</string>' "$plist"
}

has_ios_device_slice() {
  [[ -d "$1/NymVPNLibUniffi.xcframework/ios-arm64" ]]
}

ensure_ios_rust_target() {
  rustup target list --installed | grep -qx 'aarch64-apple-ios' && return 0
  log "Adding rustup target aarch64-apple-ios"
  run rustup target add aarch64-apple-ios
}

list_ios_phones() {
  local tmp
  tmp="$(mktemp)"
  xcrun devicectl list devices --json-output "$tmp" >/dev/null
  python3 - "$tmp" <<'PY'
import json, sys
for dev in json.load(open(sys.argv[1])).get("result", {}).get("devices", []):
    hp = dev.get("hardwareProperties") or {}
    dp = dev.get("deviceProperties") or {}
    cp = dev.get("connectionProperties") or {}
    if hp.get("deviceType") != "iPhone":
        continue
    udid = hp.get("udid") or ""
    ident = dev.get("identifier") or ""
    name = dp.get("name") or "?"
    tunnel = cp.get("tunnelState") or "unavailable"
    print(f"{tunnel}\t{udid}\t{ident}\t{name}")
PY
  rm -f "$tmp"
}

resolve_ios_device() {
  local online offline line tunnel udid ident name
  online=""
  offline=""
  while IFS=$'\t' read -r tunnel udid ident name; do
    [[ -n "$udid" ]] || continue
    if [[ -n "$DEVICE_UDID" && "$DEVICE_UDID" != "$udid" && "$DEVICE_UDID" != "$ident" ]]; then
      continue
    fi
    if [[ "$tunnel" == unavailable ]]; then
      offline+="$udid	$ident	$name"$'\n'
    else
      online+="$udid	$ident	$name"$'\n'
    fi
  done < <(list_ios_phones)

  local count
  count="$(printf '%s' "$online" | grep -c . || true)"
  if [[ "$count" -eq 1 ]]; then
    printf '%s\n' "$(printf '%s' "$online" | head -n1 | cut -f2)"
    return 0
  fi
  if [[ "$count" -gt 1 ]]; then
    log "More than one iPhone is connected. Pass --udid:"
    printf '%s' "$online" | while IFS=$'\t' read -r udid ident name; do
      log "  $udid  $name"
    done
    die "Pass --udid <id>"
  fi

  log "No reachable iPhone (USB or network). Xcode only lists a phone that is unlocked and trusted."
  if [[ -n "$DEVICE_UDID" ]]; then
    log "Asked for $DEVICE_UDID but its tunnel is unavailable."
  fi
  if [[ -z "$offline" ]]; then
    log "  (none paired)"
  else
    printf '%s' "$offline" | while IFS=$'\t' read -r udid ident name; do
      log "  offline  $udid  $name"
    done
  fi
  die "Unlock the iPhone, tap Trust, leave Developer Mode on, keep the cable in. Then: $0 --ios --skip-core --open${DEVICE_UDID:+ --udid $DEVICE_UDID}"
}

stage_ios_lib() {
  local src="$REPO_ROOT/nym-vpn-core/crates/nym-vpn-lib-uniffi/NymVPNLib"
  local dest="$REPO_ROOT/nym-vpn-apple/NymVPNLib"

  ensure_ios_rust_target
  log "Building iOS device UniFFI package (RELEASE=$RELEASE aarch64-apple-ios)"
  run make -C "$REPO_ROOT/nym-vpn-core" -f iOS.mk swift-package \
    RELEASE="$RELEASE" RUST_TRIPLET=aarch64-apple-ios

  [[ "$DRY_RUN" -eq 1 ]] && return 0
  [[ -d "$src" ]] || die "cargo-swift did not produce $src"
  has_ios_device_slice "$src" || die "Built NymVPNLib has no ios-arm64 slice: $src"

  log "Staging $src -> $dest"
  rm -rf "$dest"
  cp -R "$src" "$dest"
  has_ios_device_slice "$dest" || die "Staged NymVPNLib still has no ios-arm64 slice"
  [[ -f "$dest/Package.swift" ]] || die "Staged NymVPNLib is missing Package.swift"
  log "ios-arm64 slice staged"
}

install_ios_app() {
  local ident="$1"
  local app="$2"
  log "Installing $app on $ident"
  run xcrun devicectl device install app --device "$ident" "$app"
}

launch_ios_app() {
  local ident="$1"
  log "Launching $IOS_BUNDLE_ID on $ident"
  run xcrun devicectl device process launch --device "$ident" "$IOS_BUNDLE_ID"
}

build_ios_app() {
  local apple="$REPO_ROOT/nym-vpn-apple"
  local derived="$apple/build/DerivedData"
  local dest="$apple/NymVPNLib"
  local ident

  if [[ "$DRY_RUN" -eq 0 ]]; then
    has_ios_device_slice "$dest" || die "$dest has no ios-arm64 slice. Re-run without --skip-core."
  fi

  # generic/platform=iOS does not require the phone to be online during compile.
  # -destination id=<udid> fails when the tunnel is down (xctrace "Devices Offline").
  log "xcodebuild NymVPN Debug generic/platform=iOS (derivedData=$derived)"
  mkdir -p "$derived"
  run xcodebuild \
    -workspace "$apple/NymVPN.xcworkspace" \
    -scheme NymVPN \
    -configuration Debug \
    -destination 'generic/platform=iOS' \
    -derivedDataPath "$derived" \
    -allowProvisioningUpdates \
    build

  local app="$derived/Build/Products/Debug-iphoneos/NymVPN.app"
  if [[ "$DRY_RUN" -eq 1 ]]; then
    if [[ "$SKIP_INSTALL" -eq 0 ]]; then
      ident="${DEVICE_UDID:-UDID}"
      install_ios_app "$ident" "$app"
      [[ "$OPEN_APP" -eq 1 ]] && launch_ios_app "$ident"
    fi
    return 0
  fi
  [[ -d "$app" ]] || die "Build finished but $app is missing"
  log "Built $app"

  if [[ "$SKIP_INSTALL" -eq 1 ]]; then
    log "Install skipped. Plug the phone in, then re-run with --ios --skip-core --open"
    return 0
  fi
  ident="$(resolve_ios_device)"
  log "Installing on Core Device $ident"
  install_ios_app "$ident" "$app"
  if [[ "$OPEN_APP" -eq 1 ]]; then
    launch_ios_app "$ident"
  else
    log "Launch: xcrun devicectl device process launch --device $ident $IOS_BUNDLE_ID"
  fi
}

require_tools() {
  [[ "$(uname -s)" == "Darwin" ]] || die "This script only runs on macOS"
  have make || die "make not found"
  have xcodebuild || die "xcodebuild not found (install Xcode)"
  have cargo || die "cargo not found"
  have cargo-swift || die "cargo-swift not found. Install: cargo install cargo-swift --version 0.11.1"
  have go || die "go not found (needed for libwg). brew install go"
  if [[ "$PLATFORM" == ios ]]; then
    have python3 || die "python3 not found (needed to read devicectl JSON)"
  fi
  if pgrep -x nym-vpnd >/dev/null 2>&1; then
    die "nym-vpnd is running. A Mac tunnel blocks Xcode device/USB. Stop it: sudo killall nym-vpnd"
  fi
}

stage_macos_lib() {
  local src="$REPO_ROOT/nym-vpn-core/crates/nym-vpn-lib-uniffi/NymVPNLib"
  local dest="$REPO_ROOT/nym-vpn-apple/NymVPNLib"

  log "Building macOS UniFFI package (RELEASE=$RELEASE ARCH=$ARCH)"
  run make -C "$REPO_ROOT/nym-vpn-core" -f macOS.mk swift-package \
    RELEASE="$RELEASE" ARCH="$ARCH"

  [[ "$DRY_RUN" -eq 1 ]] && return 0
  [[ -d "$src" ]] || die "cargo-swift did not produce $src"
  has_macos_slice "$src" || die "Built NymVPNLib has no macOS slice: $src"

  log "Staging $src -> $dest"
  rm -rf "$dest"
  cp -R "$src" "$dest"
  has_macos_slice "$dest" || die "Staged NymVPNLib still has no macOS slice"
  [[ -f "$dest/Package.swift" ]] || die "Staged NymVPNLib is missing Package.swift"
  log "macOS slice staged"
}

build_app() {
  local apple="$REPO_ROOT/nym-vpn-apple"
  local derived="$apple/build/DerivedData"
  local dest="$apple/NymVPNLib"

  if [[ "$DRY_RUN" -eq 0 ]]; then
    has_macos_slice "$dest" || die "$dest has no macOS slice. Re-run without --skip-core."
  fi

  log "xcodebuild NymVPNDaemon Debug (derivedData=$derived)"
  mkdir -p "$derived"
  run xcodebuild \
    -workspace "$apple/NymVPN.xcworkspace" \
    -scheme NymVPNDaemon \
    -configuration Debug \
    -destination 'platform=macOS' \
    -derivedDataPath "$derived" \
    build

  [[ "$DRY_RUN" -eq 1 ]] && return 0
  local app="$derived/Build/Products/Debug/NymVPN.app"
  [[ -d "$app" ]] || die "Build finished but $app is missing"
  log "Built $app"
  ensure_daemon_bin
  print_daemon_howto
  if [[ "$START_DAEMON" -eq 1 ]]; then
    start_local_daemon
  fi
  if [[ "$OPEN_APP" -eq 1 ]]; then
    log "Opening $app"
    open "$app"
  else
    log "Run: open \"$app\""
    log "Or Xcode: NymVPN.xcworkspace, scheme NymVPNDaemon, destination My Mac, Cmd+R"
  fi
}

main() {
  parse_args "$@"
  REPO_ROOT="$(find_repo_root)"
  local dest="$REPO_ROOT/nym-vpn-apple/NymVPNLib"

  log "repo=$REPO_ROOT platform=$PLATFORM"
  require_tools

  local need_core=0
  if [[ "$PLATFORM" == ios ]]; then
    if [[ "$SKIP_CORE" -eq 1 ]]; then
      has_ios_device_slice "$dest" || die "No ios-arm64 UniFFI slice. Drop --skip-core."
    elif [[ "$REBUILD_CORE" -eq 1 || "$CORE_ONLY" -eq 1 ]]; then
      need_core=1
    elif ! has_ios_device_slice "$dest"; then
      need_core=1
      log "NymVPNLib has no ios-arm64 slice; building core"
    else
      log "ios-arm64 slice already present; skip core (pass --rebuild-core to force)"
    fi
    if [[ "$need_core" -eq 1 ]]; then
      stage_ios_lib
    fi
    if [[ "$CORE_ONLY" -eq 1 || "$SKIP_APP" -eq 1 ]]; then
      log "done (app build skipped)"
      return 0
    fi
    build_ios_app
    return 0
  fi

  if [[ "$SKIP_CORE" -eq 1 ]]; then
    has_macos_slice "$dest" || die "No macOS UniFFI slice. Drop --skip-core."
  elif [[ "$REBUILD_CORE" -eq 1 || "$CORE_ONLY" -eq 1 ]]; then
    need_core=1
  elif ! has_macos_slice "$dest"; then
    need_core=1
    log "NymVPNLib has no macOS slice; building core"
  else
    log "macOS slice already present; skip core (pass --rebuild-core to force)"
  fi

  if [[ "$need_core" -eq 1 ]]; then
    stage_macos_lib
  fi

  if [[ "$CORE_ONLY" -eq 1 || "$SKIP_APP" -eq 1 ]]; then
    log "done (app build skipped)"
    if [[ "$START_DAEMON" -eq 1 ]]; then
      ensure_daemon_bin
      start_local_daemon
    else
      print_daemon_howto
    fi
    return 0
  fi

  build_app
}

main "$@"
