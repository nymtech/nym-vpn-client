#!/usr/bin/env bash
# E2E helpers for published nym-vpn-v* linux_x86_64 core archives.
#   release-core.sh archive-name <tag>
#   release-core.sh validate-tag <tag>
#   release-core.sh latest-beta
#   release-core.sh pick-beta   # stdin: gh release list JSON
#   release-core.sh fetch <tag> <dest_dir>
set -euo pipefail

# Ship: nym-vpn-vMAJOR.MINOR.PATCH
# Pre:  ...-beta.N | ...-nightly.YYYYMMDD (or similar alnum suffix)
RELEASE_TAG_RE='^nym-vpn-v[0-9]+[.][0-9]+[.][0-9]+(-(beta|nightly)[.][0-9A-Za-z._-]+)?$'
BETA_TAG_RE='^nym-vpn-v[0-9]+[.][0-9]+[.][0-9]+-beta[.][0-9]+$'

usage() {
  echo "usage: $0 archive-name <tag> | validate-tag <tag> | latest-beta | pick-beta | fetch <tag> <dest_dir>" >&2
  exit 2
}

validate_tag() {
  local tag="${1:-}"
  [[ -n "$tag" ]] || {
    echo "error: empty tag" >&2
    exit 1
  }
  if [[ ! "$tag" =~ $RELEASE_TAG_RE ]]; then
    echo "error: tag must match nym-vpn-vMAJOR.MINOR.PATCH[-beta.N|-nightly.DATE] (got: ${tag})" >&2
    exit 1
  fi
}

archive_name() {
  local tag="${1:-}"
  validate_tag "$tag"
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
      | select(
          (.tagName | type == "string")
          and (.tagName | test($re))
        )
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

  local version archive archive_dir work src bin checksum
  load_archive_vars "$tag"
  checksum="${archive}.sha256sum"

  mkdir -p "$dest"
  work="$(mktemp -d)"

  echo "Downloading ${archive} and ${checksum} from release ${tag}"
  gh release download "$tag" --pattern "$archive" --pattern "$checksum" --dir "$work" --clobber
  [[ -f "${work}/${checksum}" ]] || {
    rm -rf "$work"
    echo "error: missing ${checksum} for ${tag}" >&2
    exit 1
  }
  (
    cd "$work"
    sha256sum -c "$checksum"
  )

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
  validate-tag) validate_tag "${1:-}" ;;
  pick-beta) pick_beta ;;
  latest-beta) latest_beta ;;
  fetch) fetch "${1:-}" "${2:-}" ;;
  *) usage ;;
esac
