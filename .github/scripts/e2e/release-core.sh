#!/usr/bin/env bash
# E2E helpers for published nym-vpn-v* linux_x86_64 core archives.
#   release-core.sh archive-name <tag>
#   release-core.sh latest-beta
#   release-core.sh pick-beta   # stdin: gh release list JSON
#   release-core.sh fetch <tag> <dest_dir>
set -euo pipefail

BETA_TAG_RE='^nym-vpn-v[0-9]+[.][0-9]+[.][0-9]+-beta[.][0-9]+$'

usage() {
  echo "usage: $0 archive-name <tag> | latest-beta | pick-beta | fetch <tag> <dest_dir>" >&2
  exit 2
}

archive_name() {
  local tag="${1:-}"
  [[ -n "$tag" ]] || usage
  [[ "$tag" == nym-vpn-v* ]] || {
    echo "error: tag must start with nym-vpn-v (got: ${tag})" >&2
    exit 1
  }
  local version="${tag#nym-vpn-v}"
  [[ -n "$version" && "$version" != "$tag" ]] || {
    echo "error: could not strip nym-vpn-v prefix from tag: ${tag}" >&2
    exit 1
  }
  local dir="nym-vpn-core-v${version}_linux_x86_64"
  echo "version=${version}"
  echo "archive=${dir}.tar.gz"
  echo "archive_dir=${dir}"
}

pick_beta() {
  jq -r --arg re "$BETA_TAG_RE" '
    [
      .[]
      | select(.tagName | test($re))
    ]
    | sort_by(.publishedAt)
    | reverse
    | .[0].tagName // empty
  '
}

latest_beta() {
  local tag
  tag="$(gh release list --limit 200 --json tagName,publishedAt | pick_beta)"
  [[ -n "$tag" ]] || {
    echo "error: no published nym-vpn-v*-beta.* release found" >&2
    exit 1
  }
  echo "$tag"
}

load_archive_vars() {
  local tag="$1" line key value
  version="" archive="" archive_dir=""
  while IFS= read -r line; do
    key="${line%%=*}"
    value="${line#*=}"
    case "$key" in
      version) version="$value" ;;
      archive) archive="$value" ;;
      archive_dir) archive_dir="$value" ;;
    esac
  done < <(archive_name "$tag")
  [[ -n "$archive" && -n "$archive_dir" ]] || {
    echo "error: failed to resolve archive names for ${tag}" >&2
    exit 1
  }
}

fetch() {
  local tag="${1:-}" dest="${2:-}"
  [[ -n "$tag" && -n "$dest" ]] || usage

  local version archive archive_dir work src bin
  load_archive_vars "$tag"

  mkdir -p "$dest"
  work="$(mktemp -d)"

  echo "Downloading ${archive} from release ${tag}"
  gh release download "$tag" --pattern "$archive" --dir "$work" --clobber
  tar -xzf "${work}/${archive}" -C "$work"

  src="${work}/${archive_dir}"
  for bin in nym-vpnd nym-vpnc nym-socks5-proxy; do
    [[ -f "${src}/${bin}" ]] || {
      rm -rf "$work"
      echo "error: ${bin} missing from ${archive}" >&2
      exit 1
    }
    cp -f "${src}/${bin}" "${dest}/${bin}"
    chmod +x "${dest}/${bin}"
  done
  rm -rf "$work"
  echo "Installed release core binaries into ${dest}"
}

cmd="${1:-}"
shift || true
case "$cmd" in
  archive-name) archive_name "${1:-}" ;;
  pick-beta) pick_beta ;;
  latest-beta) latest_beta ;;
  fetch) fetch "${1:-}" "${2:-}" ;;
  *) usage ;;
esac
