#!/bin/bash
#
# Updates the iOS core using nightly/release builds.
# Source: https://builds.cdn.nymte.ch/nym-vpn-client/nym-vpn-core/
#
# Must be run from nym-vpn-apple/Scripts.

set -euo pipefail
set -E

error_handler() {
  echo "Error occurred in script at line: ${1}. Exiting."
  exit 1
}
trap 'error_handler $LINENO' ERR

BASE_URL="https://builds.cdn.nymte.ch/nym-vpn-client/nym-vpn-core"

# -----------------------------------------------------------------------------
# 0) Determine build tag (default from git branch)
#    This may be overridden by FETCHCORE_TAG/FETCHCORE_FOLDER from FetchCore.sh
# -----------------------------------------------------------------------------
current_branch="$(git rev-parse --abbrev-ref HEAD 2>/dev/null || true)"
if [[ "$current_branch" =~ ^release/ ]]; then
  TAG="$current_branch"
else
  TAG="develop"
fi

# Orchestrator override
OVERRIDDEN="0"
if [[ -n "${FETCHCORE_TAG:-}" && -n "${FETCHCORE_FOLDER:-}" ]]; then
  TAG="${FETCHCORE_TAG}"
  latest_folder="${FETCHCORE_FOLDER}"
  OVERRIDDEN="1"
  echo "Override: Using TAG=${TAG}, FOLDER=${latest_folder} from FetchCore.sh"
fi

TAG_URL="${BASE_URL}/${TAG}"

echo "Using build tag: ${TAG}"
echo "Base folder: ${TAG_URL}"

# -----------------------------------------------------------------------------
# 1) Find latest timestamp folder (unless orchestrator provided it)
#    Garage's web vhost serves no directory listing; resolve via latest.json.
# -----------------------------------------------------------------------------
if [[ "$OVERRIDDEN" != "1" ]]; then
  echo "Resolving latest build from: ${TAG_URL}/latest.json"
  manifest="$(curl -fLs "${TAG_URL}/latest.json")"
  latest_folder="$(echo "$manifest" | grep -oP '"timestamp"\s*:\s*"\K[0-9]+')"
  if [[ -z "${latest_folder}" ]]; then
    echo "❌ Error: Could not resolve the latest build from ${TAG_URL}/latest.json"
    exit 1
  fi
fi

echo "Latest timestamp folder: ${latest_folder}"
RELEASE_URL="${TAG_URL}/${latest_folder}"

# -----------------------------------------------------------------------------
# 2) Resolve the iOS asset, download, extract
#    No directory listing on Garage: the iOS asset is co-located in the build
#    dir and named "<shared_slug>_ios_universal.zip". Use the slug exported by
#    FetchCore.sh, or lift it from this build's manifest when running standalone.
# -----------------------------------------------------------------------------
if [[ -n "${FETCHCORE_SLUG:-}" ]]; then
  shared_slug="${FETCHCORE_SLUG}"
else
  shared_slug="$(echo "${manifest:-}" | grep -Eo 'nym-vpn-core-v[0-9]+\.[0-9]+\.[0-9]+(-(?:dev|beta)\.[0-9]{12})?' | head -n 1)"
fi
if [[ -z "$shared_slug" ]]; then
  echo "❌ Error: Could not determine the build version slug (set FETCHCORE_SLUG or ensure ${RELEASE_URL}/manifest.json is reachable)."
  exit 1
fi

ios_asset="${shared_slug}_ios_universal.zip"
IOS_ASSET_URL="${RELEASE_URL}/${ios_asset}"
ios_zip_name="$(basename "$IOS_ASSET_URL")"

echo "iOS download link: ${IOS_ASSET_URL}"

# Cleanup any leftovers
rm -f "$ios_zip_name"

# Download ZIP
curl -fL -o "$ios_zip_name" "$IOS_ASSET_URL"
echo "✅ iOS ZIP downloaded: $ios_zip_name"

# Ensure unzip is available
if ! command -v unzip >/dev/null 2>&1; then
  echo "❌ Error: 'unzip' not found. Please install it (e.g., 'brew install unzip')"
  exit 1
fi

# Extract ZIP directly (contains its own top-level folder)
unzip -q -o "$ios_zip_name"
echo "✅ iOS ZIP extracted."

# Determine extracted folder name (first entry in ZIP)
extracted_folder="$(unzip -Z1 "$ios_zip_name" | head -n 1 | cut -d/ -f1)"
echo "Extracted folder: $extracted_folder"

# -----------------------------------------------------------------------------
# 3) Copy NymVPNLib into project root
# -----------------------------------------------------------------------------
if [[ -d "$extracted_folder/NymVPNLib" ]]; then
  echo "Copying NymVPNLib into project root (../).."
  rm -rf ../NymVPNLib
  cp -a "$extracted_folder/NymVPNLib" ../
  echo "✅ NymVPNLib copied to nym-vpn-apple/"
else
  echo "❌ Error: NymVPNLib not found in $extracted_folder"
  exit 1
fi

# -----------------------------------------------------------------------------
# 4) Cleanup extracted folder + ZIP
# -----------------------------------------------------------------------------
rm -f "$ios_zip_name"
rm -rf "$extracted_folder"
echo "✅ Removed ZIP and extracted folder from Scripts."

echo "🎉 Done. NymVPNLib is updated in project root."
