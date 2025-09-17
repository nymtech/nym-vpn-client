#!/bin/bash
#
# Orchestrates fetching iOS + macOS core and updates libVersion in AppVersionProvider.swift
# Must be run from nym-vpn-apple/Scripts.

set -euo pipefail
set -E

error_handler() {
  echo "Error occurred in script at line: ${1}. Exiting."
  exit 1
}
trap 'error_handler $LINENO' ERR

BASE_URL="https://builds.ci.nymte.ch/nym-vpn-client/nym-vpn-core"

# -----------------------------------------------------------------------------
# 0) Determine build tag from git branch (matches your other scripts)
# -----------------------------------------------------------------------------
current_branch="$(git rev-parse --abbrev-ref HEAD 2>/dev/null || true)"

if [[ "$current_branch" =~ ^release/ ]]; then
  TAG="$current_branch"
else
  TAG="develop"
fi

TAG_URL="${BASE_URL}/${TAG}"
echo "Using build tag: ${TAG}"
echo "Fetching folder listing from: ${TAG_URL}"

# -----------------------------------------------------------------------------
# 1) Find latest timestamp folder
# -----------------------------------------------------------------------------
folder_listing="$(curl -Ls "$TAG_URL")"
latest_folder="$(echo "$folder_listing" | grep -Eo '[0-9]{12}/' | tr -d '/' | sort | tail -n 1)"

if [[ -z "${latest_folder}" ]]; then
  echo "❌ Error: Could not determine the latest timestamp folder from ${TAG_URL}"
  exit 1
fi

echo "Latest timestamp folder: ${latest_folder}"
RELEASE_URL="${TAG_URL}/${latest_folder}"

# -----------------------------------------------------------------------------
# 2) Extract the shared version slug from the release page (works for iOS/macOS)
#    Examples we capture:
#      nym-vpn-core-v1.16.0-beta.202509160310
#      nym-vpn-core-v1.16.0
# -----------------------------------------------------------------------------
echo "Fetching release page content from: ${RELEASE_URL}"
release_page_content="$(curl -Ls "$RELEASE_URL")"
if [[ -z "$release_page_content" ]]; then
  echo "❌ Error: Release page content is empty at ${RELEASE_URL}"
  exit 1
fi

# Find the first asset’s core slug (before the _ios/_macos suffix)
# Matches both dev/beta with timestamp and plain release without pre-release suffix
shared_slug="$(echo "$release_page_content" | grep -Eo 'nym-vpn-core-v[0-9]+\.[0-9]+\.[0-9]+(-(dev|beta)\.[0-9]{12})?' | head -n 1)"

if [[ -z "$shared_slug" ]]; then
  echo "❌ Error: Could not extract shared version slug from release page."
  exit 1
fi

echo "Shared slug: ${shared_slug}"

# LIB_VERSION = piece after 'nym-vpn-core-v'
LIB_VERSION="${shared_slug#nym-vpn-core-v}"

# Strip timestamp only if it's a -dev/-beta build; keep plain releases as-is
LIB_VERSION_NO_TIMESTAMP="$(echo "$LIB_VERSION" | sed -E 's/-(dev|beta)\.[0-9]{12}$/-\1/')"

echo "Resolved LIB_VERSION: ${LIB_VERSION}"
echo "Resolved LIB_VERSION_NO_TIMESTAMP: ${LIB_VERSION_NO_TIMESTAMP}"

# -----------------------------------------------------------------------------
# 3) Update AppVersionProvider.swift
# -----------------------------------------------------------------------------
app_version_file="../ServicesMutual/Sources/AppVersionProvider/AppVersionProvider.swift"
if [[ -f "$app_version_file" ]]; then
  # macOS/BSD sed: -i ''
  sed -i '' -E 's|(public static let libVersion = ")[^"]*(")|\1'"$LIB_VERSION_NO_TIMESTAMP"'\2|' "$app_version_file"
  echo "✅ libVersion updated to ${LIB_VERSION_NO_TIMESTAMP} in ${app_version_file}."
else
  echo "❌ Error: AppVersionProvider.swift file not found at ${app_version_file}"
  exit 1
fi

# -----------------------------------------------------------------------------
# 4) Export TAG and FOLDER so the platform scripts can use the SAME folder
#    (Optional, but ensures both scripts are perfectly in sync.)
# -----------------------------------------------------------------------------
export FETCHCORE_TAG="$TAG"
export FETCHCORE_FOLDER="$latest_folder"

# -----------------------------------------------------------------------------
# 5) Run platform scripts
# -----------------------------------------------------------------------------
sh FetchIOSCore.sh
sh FetchMacOSCore.sh

echo "🎉 Done. iOS/macOS cores fetched and libVersion updated."
