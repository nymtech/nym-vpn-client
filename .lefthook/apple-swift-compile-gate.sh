#!/usr/bin/env bash
# Unsigned Release iOS compile gate for nym-vpn-apple changes.
# Mirrors CI strictness enough to catch cross-module Swift errors (access control,
# @MainActor / Sendable) before commit/push.
#
# Skip: SKIP_APPLE_SWIFT_COMPILE=1 git commit|push
# Manual: bash .lefthook/apple-swift-compile-gate.sh $(git diff --name-only develop...HEAD)

set -euo pipefail

if [[ "${SKIP_APPLE_SWIFT_COMPILE:-}" == "1" ]]; then
  echo "apple-swift-compile-gate: skipped (SKIP_APPLE_SWIFT_COMPILE=1)"
  exit 0
fi

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "apple-swift-compile-gate: skipped (not macOS)"
  exit 0
fi

if ! command -v xcodebuild >/dev/null 2>&1; then
  echo "apple-swift-compile-gate: skipped (xcodebuild not found)"
  exit 0
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
APPLE_ROOT="${REPO_ROOT}/nym-vpn-apple"

if [[ ! -d "${APPLE_ROOT}" ]]; then
  echo "apple-swift-compile-gate: skipped (nym-vpn-apple not found)"
  exit 0
fi

if [[ $# -eq 0 ]]; then
  exit 0
fi

add_unique() {
  local list_name="$1"
  local value="$2"
  local current="${!list_name}"
  case " ${current} " in
    *" ${value} "*) return ;;
  esac
  eval "${list_name}=\"\${current} ${value}\""
}

APPLE_FILES=""
for file in "$@"; do
  [[ -z "${file}" ]] && continue
  case "${file}" in
    nym-vpn-apple/*)
      case "${file}" in
        nym-vpn-apple/*/.build/*|nym-vpn-apple/*/build/*|nym-vpn-apple/*/.swiftpm/*)
          continue
          ;;
      esac
      add_unique APPLE_FILES "${file}"
      ;;
  esac
done

if [[ -z "${APPLE_FILES// }" ]]; then
  exit 0
fi

SPM_BUILDS=""
WORKSPACE_BUILDS=""
need_home=0
need_settings=0
need_credentials_manager=0
need_account_prefetch_gates=0
need_nymvpn=0
need_nymvpndaemon=0
need_nymmixnet_tunnel=0

for file in ${APPLE_FILES}; do
  case "${file}" in
    nym-vpn-apple/Home/*)
      need_home=1
      ;;
    nym-vpn-apple/Settings/*)
      need_settings=1
      ;;
    nym-vpn-apple/UIComponents/*|nym-vpn-apple/Routes/*)
      need_home=1
      need_settings=1
      ;;
    nym-vpn-apple/Theme/*)
      need_home=1
      need_settings=1
      need_credentials_manager=1
      ;;
    nym-vpn-apple/Services/Sources/AccountPrefetchGates/*|nym-vpn-apple/Services/Tests/CredentialsManagerTests/*AccountPrefetch*|nym-vpn-apple/Services/Tests/CredentialsManagerTests/*AuthCompletion*)
      need_account_prefetch_gates=1
      need_home=1
      ;;
    nym-vpn-apple/Services/*|nym-vpn-apple/ServicesIOS/*|nym-vpn-apple/ServicesMacOS/*|nym-vpn-apple/ServicesMutual/*)
      need_credentials_manager=1
      ;;
    nym-vpn-apple/NymVPN/*|nym-vpn-apple/NymVPNWidget/*|nym-vpn-apple/NymVPNmacOSWidgetExtension/*)
      need_nymvpn=1
      need_home=1
      need_settings=1
      ;;
    nym-vpn-apple/NymMixnetTunnel/*)
      need_nymmixnet_tunnel=1
      need_credentials_manager=1
      ;;
    nym-vpn-apple/NymVPNDaemon/*|nym-vpn-apple/NymVPND/*)
      need_nymvpndaemon=1
      ;;
    nym-vpn-apple/NymVPNLib/*)
      need_credentials_manager=1
      need_nymvpn=1
      ;;
    nym-vpn-apple/*)
      need_credentials_manager=1
      need_home=1
      ;;
  esac
done

if [[ "${need_account_prefetch_gates}" -eq 1 ]]; then
  add_unique SPM_BUILDS "Services:AccountPrefetchGates"
fi
if [[ "${need_credentials_manager}" -eq 1 ]]; then
  add_unique SPM_BUILDS "Services:CredentialsManager"
fi
if [[ "${need_home}" -eq 1 ]]; then
  add_unique SPM_BUILDS "Home:Home"
fi
if [[ "${need_settings}" -eq 1 ]]; then
  add_unique SPM_BUILDS "Settings:Settings"
fi
if [[ "${need_nymvpn}" -eq 1 ]]; then
  add_unique WORKSPACE_BUILDS "NymVPN"
fi
if [[ "${need_nymmixnet_tunnel}" -eq 1 ]]; then
  add_unique WORKSPACE_BUILDS "NymMixnetTunnel"
fi
if [[ "${need_nymvpndaemon}" -eq 1 ]]; then
  add_unique WORKSPACE_BUILDS "NymVPNDaemon"
fi

if [[ -z "${SPM_BUILDS// }" && -z "${WORKSPACE_BUILDS// }" ]]; then
  add_unique SPM_BUILDS "Services:CredentialsManager"
fi

pipe_xcodebuild() {
  if command -v xcbeautify >/dev/null 2>&1; then
    xcbeautify
  else
    cat
  fi
}

build_spm_scheme() {
  local package_dir="$1"
  local scheme="$2"
  echo ""
  echo "==> apple-swift-compile-gate: ${package_dir} / ${scheme} (Release, iOS, unsigned)"
  (
    cd "${APPLE_ROOT}/${package_dir}"
    xcodebuild build \
      -scheme "${scheme}" \
      -destination 'generic/platform=iOS' \
      -configuration Release \
      CODE_SIGNING_ALLOWED=NO \
      | pipe_xcodebuild
  )
}

build_workspace_scheme() {
  local scheme="$1"
  local xcframework="${APPLE_ROOT}/NymVPNLib/NymVPNLibUniffi.xcframework"

  if [[ ! -d "${xcframework}" ]]; then
    echo "apple-swift-compile-gate: warning - missing ${xcframework}"
    echo "  Run: cd nym-vpn-apple/scripts && sh FetchIOSCore.sh"
    echo "  Skipping workspace scheme ${scheme}."
    return 0
  fi

  echo ""
  echo "==> apple-swift-compile-gate: NymVPN.xcworkspace / ${scheme} (Release, iOS, unsigned)"
  (
    cd "${APPLE_ROOT}"
    xcodebuild build \
      -workspace NymVPN.xcworkspace \
      -scheme "${scheme}" \
      -destination 'generic/platform=iOS' \
      -configuration Release \
      CODE_SIGNING_ALLOWED=NO \
      | pipe_xcodebuild
  )
}

file_count=0
for _ in ${APPLE_FILES}; do
  file_count=$((file_count + 1))
done
echo "apple-swift-compile-gate: ${file_count} apple file(s)"

for key in ${SPM_BUILDS}; do
  package_dir="${key%%:*}"
  scheme="${key#*:}"
  build_spm_scheme "${package_dir}" "${scheme}"
done

for scheme in ${WORKSPACE_BUILDS}; do
  build_workspace_scheme "${scheme}"
done

echo ""
echo "apple-swift-compile-gate: passed"
