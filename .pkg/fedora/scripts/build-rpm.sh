#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
CONTAINER_ENGINE=${CONTAINER_ENGINE:-podman}

usage() {
    echo "Usage: $0 aarch64|x86_64|all" >&2
}

if [[ $# -ne 1 ]]; then
    usage
    exit 2
fi

case "$1" in
    aarch64) architectures=(aarch64) ;;
    x86_64) architectures=(x86_64) ;;
    all) architectures=(aarch64 x86_64) ;;
    *) usage; exit 2 ;;
esac

if ! command -v "${CONTAINER_ENGINE}" >/dev/null 2>&1; then
    echo "${CONTAINER_ENGINE} is required." >&2
    if [[ "${CONTAINER_ENGINE}" == podman && "$(uname -s)" == Darwin ]]; then
        echo "Run scripts/bootstrap-podman-macos.sh first." >&2
    fi
    exit 1
fi

if ! "${CONTAINER_ENGINE}" info >/dev/null 2>&1; then
    echo "${CONTAINER_ENGINE} is installed but its service or VM is not running." >&2
    exit 1
fi

# shellcheck disable=SC1091
source "${ROOT_DIR}/sources.lock"

mkdir -p "${ROOT_DIR}/dist/SRPMS"

build_architecture() {
    local rpm_arch=$1
    local platform
    local image

    case "${rpm_arch}" in
        aarch64) platform=linux/arm64 ;;
        x86_64) platform=linux/amd64 ;;
        *) echo "Unsupported RPM architecture: ${rpm_arch}" >&2; return 2 ;;
    esac

    image="localhost/nym-vpn-rpm-builder:fedora44-${rpm_arch}"
    mkdir -p "${ROOT_DIR}/dist/${rpm_arch}"

    echo "Building Fedora 44 builder image for ${rpm_arch} (${platform})"
    "${CONTAINER_ENGINE}" build \
        --platform "${platform}" \
        --build-arg "RUST_VERSION=${RUST_VERSION}" \
        --build-arg "GO_VERSION=${GO_VERSION}" \
        --build-arg "GO_LINUX_AMD64_SHA256=${GO_LINUX_AMD64_SHA256}" \
        --build-arg "GO_LINUX_ARM64_SHA256=${GO_LINUX_ARM64_SHA256}" \
        --build-arg "NODE_VERSION=${NODE_VERSION}" \
        --build-arg "NODE_LINUX_X64_SHA256=${NODE_LINUX_X64_SHA256}" \
        --build-arg "NODE_LINUX_ARM64_SHA256=${NODE_LINUX_ARM64_SHA256}" \
        --build-arg "PROTOC_VERSION=${PROTOC_VERSION}" \
        --build-arg "PROTOC_LINUX_X86_64_SHA256=${PROTOC_LINUX_X86_64_SHA256}" \
        --build-arg "PROTOC_LINUX_AARCH_64_SHA256=${PROTOC_LINUX_AARCH_64_SHA256}" \
        --tag "${image}" \
        --file "${ROOT_DIR}/Containerfile" \
        "${ROOT_DIR}"

    echo "Compiling NymVPN ${NYM_VERSION} for ${rpm_arch}"
    "${CONTAINER_ENGINE}" run --rm \
        --platform "${platform}" \
        --volume "${ROOT_DIR}:/workspace" \
        --volume "nym-vpn-cargo-registry-${rpm_arch}:/opt/cargo/registry" \
        --volume "nym-vpn-cargo-git-${rpm_arch}:/opt/cargo/git" \
        --volume "nym-vpn-cargo-target-${rpm_arch}:/var/cache/nym-vpn-cargo-target" \
        --volume "nym-vpn-go-mod-${rpm_arch}:/root/go/pkg/mod" \
        --volume "nym-vpn-go-build-${rpm_arch}:/root/.cache/go-build" \
        --volume "nym-vpn-npm-cache-${rpm_arch}:/var/cache/npm" \
        "${image}" \
        /workspace/scripts/container-build.sh "${rpm_arch}"
}

for architecture in "${architectures[@]}"; do
    build_architecture "${architecture}"
done

(
    cd "${ROOT_DIR}/dist"
    : > SHA256SUMS
    while IFS= read -r artifact; do
        if command -v sha256sum >/dev/null 2>&1; then
            sha256sum "${artifact}" >> SHA256SUMS
        else
            shasum -a 256 "${artifact}" >> SHA256SUMS
        fi
    done < <(find . -type f ! -name SHA256SUMS | LC_ALL=C sort)
)

echo "Artifacts and logs are available under ${ROOT_DIR}/dist"
