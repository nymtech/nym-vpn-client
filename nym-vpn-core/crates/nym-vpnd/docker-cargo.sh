#!/usr/bin/env bash
set -euo pipefail

# override by exporting before calling, e.g.:
#   export RUST_VERSION=1.72.0 && ./docker-cargo.sh build --release
RUST_VERSION=1.89.0

# This script’s directory (.../crates/nym-vpnd)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Rust workspace root
WORKSPACE_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

IMAGE_NAME="nym-vpnd-builder:${RUST_VERSION}"
DOCKERFILE="${SCRIPT_DIR}/Dockerfile"

# Build the builder image
echo "🚧 Building ${IMAGE_NAME} (Rust ${RUST_VERSION})..."
docker build \
  --file "${DOCKERFILE}" \
  --tag "${IMAGE_NAME}" \
  --build-arg RUST_VERSION="${RUST_VERSION}" \
  "${WORKSPACE_ROOT}"

# mount the entire workspace (which includes the top‐level Cargo.toml)
echo "🚀 Running cargo $* in container..."
docker run --rm -t \
  --workdir /workspace \
  --volume "${WORKSPACE_ROOT}":/workspace \
  "${IMAGE_NAME}" \
  "$@"
