#!/bin/bash

# Updates the lib and daemon in the iOS+macOS project using nightly/release builds.
# Source: https://builds.ci.nymte.ch/nym-vpn-client/nym-vpn-core/
#
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
# 0) Determine build tag (default from git branch)
#    May be overridden by FETCHCORE_TAG/FETCHCORE_FOLDER from FetchCore.sh
# -----------------------------------------------------------------------------
current_branch="$(git rev-parse --abbrev-ref HEAD 2>/dev/null || true)"
if [[ "$current_branch" =~ ^release/ ]]; then
  TAG="$current_branch"
else
  TAG="develop"
fi

OVERRIDDEN="0"
if [[ -n "${FETCHCORE_TAG:-}" && -n "${FETCHCORE_FOLDER:-}" ]]; then
  TAG="${FETCHCORE_TAG}"
  latest_folder="${FETCHCORE_FOLDER}"
  OVERRIDDEN="1"
  echo "Override: Using TAG=${TAG}, FOLDER=${latest_folder} from FetchCore.sh"
fi

TAG_URL="${BASE_URL}/${TAG}"

# Choose the macOS asset pattern after TAG is finalized
if [[ "$TAG" =~ ^release/ ]]; then
  # Release builds may omit the -dev/-beta timestamp
  macos_pattern='nym-vpn-core-v[0-9]+\.[0-9]+\.[0-9]+(-(?:dev|beta)\.[0-9]{12})?_macos_universal\.tar\.gz'
else
  # Nightly/dev builds include -dev/-beta + 12-digit timestamp
  macos_pattern='nym-vpn-core-v[0-9]+\.[0-9]+\.[0-9]+-(?:dev|beta)\.[0-9]{12}_macos_universal\.tar\.gz'
fi

echo "Using build tag: ${TAG}"
echo "Base folder: ${TAG_URL}"

# -----------------------------------------------------------------------------
# 1) Find latest timestamp folder (unless orchestrator provided it)
# -----------------------------------------------------------------------------
if [[ "$OVERRIDDEN" != "1" ]]; then
  echo "Fetching folder listing from ${TAG_URL}..."
  folder_listing="$(curl -Ls "$TAG_URL")"
  latest_folder="$(echo "$folder_listing" | grep -Eo '[0-9]{12}/' | tr -d '/' | sort | tail -n 1)"
  if [[ -z "$latest_folder" ]]; then
    echo "❌ Error: Could not determine the latest timestamp folder from ${TAG_URL}"
    exit 1
  fi
fi

echo "Latest timestamp folder: ${latest_folder}"
RELEASE_URL="${TAG_URL}/${latest_folder}"

echo "Fetching release page content from ${RELEASE_URL}..."
release_page_content="$(curl -Ls "$RELEASE_URL")"
if [[ -z "$release_page_content" ]]; then
  echo "❌ Error: Release page content is empty. Please verify that ${RELEASE_URL} exists and is accessible."
  exit 1
fi

# -----------------------------------------------------------------------------
# 2) Discover macOS asset, download, extract
# -----------------------------------------------------------------------------
macos_asset="$(echo "$release_page_content" | grep -Eo "$macos_pattern" | head -n 1)"
if [[ -z "$macos_asset" ]]; then
  echo "❌ Error: Could not find macOS asset filename in the release page."
  echo "Pattern used: $macos_pattern"
  exit 1
fi

macos_download_link="${RELEASE_URL}/${macos_asset}"
echo "macOS Download link: ${macos_download_link}"
curl -fLO "$macos_download_link"
echo "macOS file downloaded successfully: $(basename "$macos_download_link")"

tar_file_name="$(basename "$macos_download_link")"
tar -xzf "$tar_file_name"
echo "✅ macOS file extracted successfully: $tar_file_name"

extracted_folder_name="$(tar -tf "$tar_file_name" | head -n 1 | cut -f1 -d"/")"

# Copy daemon and make executable
if [[ -f "../Daemon/nym-vpnd" ]]; then
  rm "../Daemon/nym-vpnd"
  echo "✅ Removed old nym-vpnd file."
fi

# Copy nym-setup and make executable
if [[ -f "../Daemon/nym-setup" ]]; then
  rm "../Daemon/nym-setup"
  echo "✅ Removed old nym-setup file."
fi

if [[ -f "${extracted_folder_name}/nym-vpnd" ]]; then
  cp "${extracted_folder_name}/nym-vpnd" "../Daemon/nym-vpnd"
  chmod +x "../Daemon/nym-vpnd"
  echo "✅ nym-vpnd copied successfully."
else
  echo "❌ Error: ${extracted_folder_name}/nym-vpnd not found."
  exit 1
fi

if [[ -f "${extracted_folder_name}/nym-setup" ]]; then
  cp "${extracted_folder_name}/nym-setup" "../Daemon/nym-setup"
  chmod +x "../Daemon/nym-vpnd"
  echo "✅ nym-setup copied successfully."
else
  echo "❌ Error: ${extracted_folder_name}/nym-setup not found."
  exit 1
fi

# Copy proto set
if [[ -d "${extracted_folder_name}/proto" ]]; then
  rm -rf "../ServicesMacOS/Sources/GRPCManager/proto"
  cp -a "${extracted_folder_name}/proto" "../ServicesMacOS/Sources/GRPCManager"
  echo "✅ proto directory copied to ../ServicesMacOS/Sources/GRPCManager (overwritten)."
else
  echo "❌ Error: ${extracted_folder_name}/proto not found."
  exit 1
fi

# Cleanup
if [[ -f "$tar_file_name" ]]; then
  echo "✅ Removing downloaded tar.gz file: $tar_file_name"
  rm -f "$tar_file_name"
else
  echo "❌ Downloaded tar.gz file not found: $tar_file_name"
fi

if [[ -d "$extracted_folder_name" ]]; then
  echo "Removing extracted folder: $extracted_folder_name"
  rm -rf "$extracted_folder_name"
  echo "Extracted folder removed successfully."
else
  echo "❌ Extracted folder not found: $extracted_folder_name"
fi

echo "✅ Cleanup completed."
