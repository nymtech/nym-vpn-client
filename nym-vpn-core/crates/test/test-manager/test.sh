#!/bin/bash
# Copyright 2016-2025 Mullvad VPN AB. All Rights Reserved.
# Copyright 2025 Nym Technologies SA <contact@nymtech.net>
# SPDX-License-Identifier: GPL-3.0-only

set -e

GREEN='\e[32m'
YELLOW='\e[33m'
BLUE='\e[34m'
NC='\e[0m'

export RUST_LOG=debug
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

if [ -n "${TEST_DIST_DIR:-}" ]; then
    # CI: script was SCP'd to VM, repo structure doesn't exist
    TEST_FRAMEWORK_ROOT="$(pwd)"
else
    TEST_FRAMEWORK_ROOT="$(realpath "$SCRIPT_DIR/..")"
fi

CARGO_WORKSPACE_ROOT="$(realpath "${TEST_FRAMEWORK_ROOT}/../.." 2>/dev/null || echo "")"
REPO_DIR="$(realpath "${TEST_FRAMEWORK_ROOT}/../../.." 2>/dev/null || echo "")"
OUTPUT_DIR="${TEST_FRAMEWORK_ROOT}/target/x86_64-unknown-linux-gnu/release"
PACKAGE_DIR="${PACKAGE_DIR:-${HOME}/.cache/nym-test/packages}"
CONTAINER_RUNNER="${CONTAINER_RUNNER:-podman}"
CARGO_REGISTRY_VOLUME_NAME="${CARGO_REGISTRY_VOLUME_NAME:-cargo-registry}"
IMAGE_TAG="nym-test-builder"

# Default values (can be overridden by env vars or CLI params)
NYM_TEST_QCOW_IMAGE="${NYM_TEST_QCOW_IMAGE:-}"
NYM_TEST_VM_CONFIG="${NYM_TEST_VM_CONFIG:-}"


echo "TEST_FRAMEWORK_ROOT=${TEST_FRAMEWORK_ROOT}"

function usage() {
    echo "Usage: $0 [options] <command>"
    echo ""
    echo "Available commands:"
    echo "  configure   - Configure a new VM BEFORE running run-tests"
    echo "  list        - List existing configurations"
    echo "  run-tests   - Build dependencies and run tests"
    echo "  run-vm      - Run a VM WITHOUT tests so you can inspect the VM tests will run in"
    echo ""
}

function check_required_vars() {
    local missing=0
    local qcow_image_var_name="NYM_TEST_QCOW_IMAGE"
    local vm_config_var_name="NYM_TEST_VM_CONFIG"
    local mnemonic_var_name="TEST_HARNESS_MNEMONIC"


    if [ -z "$NYM_TEST_QCOW_IMAGE" ]; then
        echo "Error: ${qcow_image_var_name} is not set"
        echo "  Set it via environment variable: export ${qcow_image_var_name}=/path/to/image.qcow2"
        missing=1
    fi

    if [ -z "$NYM_TEST_VM_CONFIG" ]; then
        echo "Error: ${vm_config_var_name} is not set"
        echo "  Set it via environment variable: export ${vm_config_var_name}=config-name"
        missing=1
    fi

    if [ -z "$TEST_HARNESS_MNEMONIC" ]; then
        echo "Error: ${mnemonic_var_name} is not set"
        echo "  Set it via environment variable: export ${mnemonic_var_name}=config-name"
        missing=1
    fi

    if [ $missing -eq 1 ]; then
        echo ""
        usage
        exit 1
    fi


    echo -e "${GREEN}Using VM configuration:${NC}"
    echo -e "\tQCOW Image: ${BLUE}${NYM_TEST_QCOW_IMAGE}${NC}"
    echo -e "\tVM Config:  ${BLUE}${NYM_TEST_VM_CONFIG}${NC}"
    echo ""
}

function build_wireguard_deps() {
    pushd "${REPO_DIR}"
    make build-wireguard
    popd
}

function help() {
    cargo run -p test-manager -- config vm set --help
}

function list() {
    if [ -n "${TEST_DIST_DIR:-}" ]; then
        "${TEST_DIST_DIR}/test-manager" config vm list
    else
        cargo run -p test-manager -- config vm list
    fi
}

function configure() {
    if [ -n "${TEST_DIST_DIR:-}" ]; then
        # CI mode: use pre-built test-manager binary
        local test_manager="${TEST_DIST_DIR}/test-manager"
    else
        # Local mode: build wireguard deps first, then use cargo run
        build_wireguard_deps
        local test_manager="cargo run -p test-manager --"
    fi

    pushd "${TEST_FRAMEWORK_ROOT}"
    $test_manager config vm set \
        ${NYM_TEST_VM_CONFIG} \
        "qemu" \
        "${NYM_TEST_QCOW_IMAGE}" \
        "linux" \
        --package-type "deb" \
        --architecture "x64" \
        --provisioner "ssh" \
        --ssh-user "test" \
        --ssh-password "test" \
        --vcpus 2 \
        --memory 1024
    popd

    list
}

function run_vm() {
    if [ -n "${TEST_DIST_DIR:-}" ]; then
        local test_manager="${TEST_DIST_DIR}/test-manager"
    else
        local test_manager="cargo run -p test-manager --"
    fi

    $test_manager run-vm ${NYM_TEST_VM_CONFIG} \
        --vnc 5901
        # --keep-changes
}

function build_all_in_container() {
    mkdir -p "${PACKAGE_DIR}"

    echo -e "======== ${YELLOW} Building container image${NC} ========"
    local rust_arg=""
    if [ -n "${RUST_VERSION:-}" ]; then
        rust_arg="--build-arg RUST_VERSION=${RUST_VERSION}"
    fi
    "$CONTAINER_RUNNER" build ${rust_arg} -t "${IMAGE_TAG}" -f "${TEST_FRAMEWORK_ROOT}/Dockerfile" "${REPO_DIR}"
    echo -e "======== ${GREEN} Container image ready${NC} ========"

    echo -e "======== ${YELLOW} Building all binaries in container${NC} ========"
    "$CONTAINER_RUNNER" run --rm \
        -v "${CARGO_REGISTRY_VOLUME_NAME}":/root/.cargo/registry:Z \
        -v "${REPO_DIR}":/build:z \
        -w /build \
        -v "${PACKAGE_DIR}":/packages:Z \
        "${IMAGE_TAG}" \
        /bin/bash -c "
            set -ex && \
            cd /build && ./wireguard/build-wireguard-go.sh && \
            cd /build/nym-vpn-core && \
            cargo build --package nym-vpnc --package nym-vpnd --release && \
            cd /build/nym-vpn-core/crates/test && \
            CARGO_TARGET_DIR=/build/nym-vpn-core/crates/test/target \
            cargo build --package test-runner \
                --release --target x86_64-unknown-linux-gnu && \
            cp /build/nym-vpn-core/target/release/nym-vpnc /packages/ && \
            cp /build/nym-vpn-core/target/release/nym-vpnd /packages/ && \
            cp /build/nym-vpn-core/crates/test/target/x86_64-unknown-linux-gnu/release/test-runner /packages/
        "
    echo -e "======== ${GREEN} All binaries built and copied to ${PACKAGE_DIR}${NC} ========"
}

function run_tests() {
    if [ -n "${TEST_DIST_DIR:-}" ]; then
        # CI: use pre-built binaries
        RUNNER_DIR="${TEST_DIST_DIR}"
        TEST_MANAGER="${TEST_DIST_DIR}/test-manager"
    else
        # Local: build everything in container
        build_all_in_container
        RUNNER_DIR="${PACKAGE_DIR}"
        TEST_MANAGER="cargo run -p test-manager --"
    fi

    local test_report_arg=()
    if [ -n "${TEST_REPORT:-}" ]; then
        test_report_arg=(--test-report "${TEST_REPORT}")
    fi

    local test_filters_args=()
    if [ -n "${TEST_FILTERS:-}" ]; then
        # Split space-separated filters into individual args
        read -ra test_filters_args <<< "${TEST_FILTERS}"
    fi

    local test_skip_args=()
    if [ -n "${SKIP_TESTS:-}" ]; then
        for skip in ${SKIP_TESTS}; do
            test_skip_args+=(--skip "$skip")
        done
    fi

    pushd "${TEST_FRAMEWORK_ROOT}"
    $TEST_MANAGER run-tests \
        --vm ${NYM_TEST_VM_CONFIG} \
        --vnc 5901 \
        --nym-mnemonic "${TEST_HARNESS_MNEMONIC}" \
        --runner-dir "${RUNNER_DIR}" \
        "${test_report_arg[@]}" \
        "${test_skip_args[@]}" \
        --verbose "${test_filters_args[@]}"
    popd
}

# Parse command-line arguments
if [ $# -eq 0 ]; then
    echo "Error: No command provided"
    echo ""
    usage
    exit 1
fi


COMMAND="$1"

case "$COMMAND" in
    configure)
        check_required_vars
        configure
        ;;
    list)
        list
        ;;
    run-tests)
        check_required_vars
        run_tests
        ;;
    run-vm)
        check_required_vars
        run_vm
        ;;
    *)
        echo "Error: Unknown command '$COMMAND'"
        echo ""
        usage
        exit 1
        ;;
esac
