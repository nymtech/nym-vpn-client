#!/usr/bin/env bash
# -----------------------------------------------------------------------------
# Force re-exec under real Bash (not POSIX mode), even if called via `sh`.
# -----------------------------------------------------------------------------
if [ -z "${BASH_VERSION:-}" ] || ( command -v shopt >/dev/null 2>&1 && shopt -oq posix ); then
  exec /usr/bin/env bash "$0" "$@"
fi

#
# Orchestrates fetching iOS + macOS core and updates libVersion in AppVersionProvider.swift
# Must be run from nym-vpn-apple/Scripts.
#

set -euo pipefail
set -E

error_handler() {
  echo "Error occurred in script at line: ${1}. Exiting."
  exit 1
}
trap 'error_handler $LINENO' ERR

BASE_URL="https://builds.ci.nymte.ch/nym-vpn-client/nym-vpn-core"
MAX_DISTANCE=100

# -----------------------------------------------------------------------------
# 0) Determine build tag (branch to fetch from)
#    Priority:
#      1) CI base branch (Bitrise: BITRISE_GIT_BRANCH_DEST / BITRISEIO_GIT_BRANCH_DEST)
#      2) Local resolve: closest of origin/develop and origin/release/* (within MAX_DISTANCE)
#      3) Fallback: develop
# -----------------------------------------------------------------------------

get_ci_base_branch() {
  if [[ -n "${BITRISE_GIT_BRANCH_DEST:-}" ]]; then
    echo "${BITRISE_GIT_BRANCH_DEST}"
    return 0
  fi
  if [[ -n "${BITRISEIO_GIT_BRANCH_DEST:-}" ]]; then
    echo "${BITRISEIO_GIT_BRANCH_DEST}"
    return 0
  fi
  return 1
}

ensure_origin_fetched() {
  if git remote get-url origin >/dev/null 2>&1; then
    git fetch origin --quiet || true
  fi
}

# Shallow fetch a single branch head into refs/remotes/origin/<name>
fetch_remote_head_if_missing() {
  local name="$1"
  git show-ref --verify --quiet "refs/remotes/origin/${name}" && return 0
  git fetch --no-tags --depth=1 origin "refs/heads/${name}:refs/remotes/origin/${name}" >/dev/null 2>&1 || true
}

determine_tag_locally() {
  ensure_origin_fetched

  # Build candidates = develop + all release/*
  local candidates=("develop")

  # List release/* heads without process substitution; use a temp file (bash 3.2 friendly).
  local tmpfile
  tmpfile="$(mktemp)"
  git ls-remote --heads origin 'release/*' 2>/dev/null >"$tmpfile" || true

  while IFS=$'\t' read -r sha ref; do
    [ -z "${ref:-}" ] && continue
    local name="${ref#refs/heads/}"
    candidates+=("$name")
  done <"$tmpfile"
  rm -f "$tmpfile"

  # Ensure each candidate exists locally (shallow)
  local c
  for c in "${candidates[@]}"; do
    fetch_remote_head_if_missing "$c"
  done

  local current_ref="HEAD"
  local best_base=""
  local best_distance=$((MAX_DISTANCE + 1))

  for c in "${candidates[@]}"; do
    if git show-ref --verify --quiet "refs/remotes/origin/$c"; then
      local merge_base
      merge_base="$(git merge-base "$current_ref" "refs/remotes/origin/$c" 2>/dev/null || true)"
      if [[ -n "$merge_base" ]]; then
        local distance
        distance="$(git rev-list --count "$merge_base..$current_ref" 2>/dev/null || echo 999999)"
        if [[ "$distance" =~ ^[0-9]+$ ]] && (( distance <= MAX_DISTANCE )) && (( distance < best_distance )); then
          best_distance=$distance
          best_base="$c"
        fi
      fi
    fi
  done

  if [[ -n "$best_base" ]]; then
    echo "$best_base"
  else
    echo "develop"
  fi
}

determine_tag() {
  # 1) CI-provided base branch (Bitrise PR builds)
  if ci_base="$(get_ci_base_branch)"; then
    echo "$ci_base"
    return 0
  fi
  # 2) Local resolution (develop + release/*)
  determine_tag_locally
}

# Decide TAG
TAG="$(determine_tag)"
echo "Using build tag: ${TAG}"

TAG_URL="${BASE_URL}/${TAG}"
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
# -----------------------------------------------------------------------------
echo "Fetching release page content from: ${RELEASE_URL}"
release_page_content="$(curl -Ls "$RELEASE_URL")"
if [[ -z "$release_page_content" ]]; then
  echo "❌ Error: Release page content is empty at ${RELEASE_URL}"
  exit 1
fi

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
# -----------------------------------------------------------------------------
export FETCHCORE_TAG="$TAG"
export FETCHCORE_FOLDER="$latest_folder"

# -----------------------------------------------------------------------------
# 5) Run platform scripts (use bash explicitly)
# -----------------------------------------------------------------------------
bash FetchIOSCore.sh
bash FetchMacOSCore.sh

echo "🎉 Done. iOS/macOS cores fetched and libVersion updated."
