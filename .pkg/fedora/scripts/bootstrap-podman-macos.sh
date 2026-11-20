#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)

if [[ "$(uname -s)" != Darwin ]]; then
    echo "This bootstrap script is only for macOS." >&2
    exit 2
fi

if ! command -v brew >/dev/null 2>&1; then
    echo "Homebrew is required to install Podman." >&2
    exit 1
fi

if ! command -v podman >/dev/null 2>&1; then
    brew install podman
fi

# Podman reads AppleHV's Rosetta setting from containers.conf every time a
# machine starts. Keep the build-specific setting local to this project rather
# than changing the user's global Podman configuration.
if [[ "$(uname -m)" == arm64 ]]; then
    export CONTAINERS_CONF="${ROOT_DIR}/packaging/podman-machine.containers.conf"
fi

machine=nym-rpm-builder
if ! podman machine inspect "${machine}" >/dev/null 2>&1; then
    podman machine init \
        --cpus 8 \
        --memory 12288 \
        --swap 12288 \
        --disk-size 100 \
        "${machine}"
fi

if ! podman machine inspect "${machine}" --format '{{.State}}' | grep -Fq running; then
    podman machine start "${machine}"
fi

podman info >/dev/null
if [[ "$(uname -m)" == arm64 ]]; then
    test "$(podman machine inspect "${machine}" --format '{{.Rosetta}}')" = true || {
        echo "Podman did not enable Rosetta for amd64 container builds." >&2
        exit 1
    }
fi

echo "Podman machine ${machine} is ready (8 CPUs, 12 GiB RAM, 12 GiB swap, 100 GiB disk when newly created)."
