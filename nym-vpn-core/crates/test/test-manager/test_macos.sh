#!/bin/bash

# Local equivalent of the e2e-test GitHub Actions workflow.
# Builds all Linux binaries in a container, spins up an Exoscale VM via
# Terraform, SCPs binaries + test script, and runs the E2E suite.

set -euo pipefail

GREEN='\e[32m'
YELLOW='\e[33m'
RED='\e[31m'
NC='\e[0m'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TEST_FRAMEWORK_ROOT="$(realpath "$SCRIPT_DIR/..")"
REPO_DIR="$(realpath "${TEST_FRAMEWORK_ROOT}/../../..")"
TF_DIR="${TEST_FRAMEWORK_ROOT}/ci"

CONTAINER_RUNNER="${CONTAINER_RUNNER:-docker}"
CARGO_REGISTRY_VOLUME_NAME="${CARGO_REGISTRY_VOLUME_NAME:-cargo-registry}"
CARGO_TARGET_VOLUME_NAME="${CARGO_TARGET_VOLUME_NAME:-nym-test-cargo-target}"
IMAGE_TAG="nym-test-builder"

DIST_DIR="${DIST_DIR:-${HOME}/.cache/nym-test/dist}"

export TF_VAR_instance_type="${TF_VAR_instance_type:-standard.medium}"
export TF_VAR_run_id="${TF_VAR_run_id:-local-$(date +%s)}"

VM_USER="ubuntu"
VM_DIST_DIR="/home/${VM_USER}/dist"
SSH_OPTS="-o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -o ConnectTimeout=10 -o ServerAliveInterval=30 -o ServerAliveCountMax=3"

VM_IP=""

REQUIRED_BINARIES=(nym-vpnc nym-vpnd nym-socks5-proxy test-manager test-runner)


function validate_env() {
    local required_vars=(
        TF_VAR_exoscale_api_key
        TF_VAR_exoscale_api_secret
        TF_VAR_data_volume_id
        TF_VAR_template_id
        EXOSCALE_SSH_KEY_PATH
        TEST_HARNESS_MNEMONIC
    )

    local missing=0
    for var in "${required_vars[@]}"; do
        eval "val=\${${var}:-}"
        if [ -z "$val" ]; then
            echo -e "${RED}Error: ${var} is not set${NC}"
            missing=1
        fi
    done
    if [ $missing -eq 1 ]; then
        echo ""
        echo "Required environment variables:"
        printf '  %s\n' "${required_vars[@]}"
        exit 1
    fi

    if [ ! -f "${EXOSCALE_SSH_KEY_PATH}" ]; then
        echo -e "${RED}Error: SSH key not found at ${EXOSCALE_SSH_KEY_PATH}${NC}"
        exit 1
    fi
}


function build_all_in_container() {
    if [ -n "${SKIP_BUILD:-}" ]; then
        echo -e "${YELLOW}SKIP_BUILD set: skipping container build${NC}"
        return
    fi

    mkdir -p "${DIST_DIR}"

    echo -e "${YELLOW}Building container image (${IMAGE_TAG})...${NC}"

    local rust_arg=""
    if [ -n "${RUST_VERSION:-}" ]; then
        rust_arg="--build-arg RUST_VERSION=${RUST_VERSION}"
    fi

    # Force linux/amd64 so the build produces x86_64 binaries even when the host
    # is ARM (e.g. Apple Silicon) because Go compiler defaults to matching host architecture
    "$CONTAINER_RUNNER" build --platform linux/amd64 ${rust_arg} \
        -t "${IMAGE_TAG}" -f "${TEST_FRAMEWORK_ROOT}/Dockerfile" "${REPO_DIR}"

    echo -e "${YELLOW}Building all binaries in container...${NC}"

    # All test crates are members of the nym-vpn-core workspace, so output
    # for every package lands in nym-vpn-core/target/release/ by default.
    # This would invalidate host build cache every time container build runs and vice versa.
    # In order to have incremental compilation (to speed up subsequent builds)
    # even with container builds, we need named volume that would hold those.
    # But we mount it to a different path than the host's release artefacts (target/release)
    # so it keeps containerized build artifacts isolatedso from the host's so
    # both proper incremental compilation.
    "$CONTAINER_RUNNER" run --rm --platform linux/amd64 \
        -v "${CARGO_REGISTRY_VOLUME_NAME}":/root/.cargo/registry:Z \
        -v "${CARGO_TARGET_VOLUME_NAME}":/cargo-target:Z \
        -v "${REPO_DIR}":/build:z \
        -w /build \
        -v "${DIST_DIR}":/dist:Z \
        -e CARGO_TARGET_DIR=/cargo-target \
        "${IMAGE_TAG}" \
        /bin/bash -c "
            set -ex

            # wireguard-go shared library
            cd /build && ./wireguard/build-wireguard-go.sh

            # VPN client + daemon
            cd /build/nym-vpn-core
            cargo build --release -p nym-vpnc -p nym-socks5-proxy -p nym-vpnd

            # test-manager (runs on exoscale ubuntu host),
            # test-runner (runs inside debian guest VM)
            cd /build/nym-vpn-core/crates/test
            cargo build --release -p test-manager -p test-runner

            # collect all binaries
            cp /cargo-target/release/nym-vpnc          /dist/
            cp /cargo-target/release/nym-socks5-proxy /dist/
            cp /cargo-target/release/nym-vpnd          /dist/
            cp /cargo-target/release/test-manager       /dist/
            cp /cargo-target/release/test-runner        /dist/
        "

    echo -e "${GREEN}All binaries built in ${DIST_DIR}${NC}"
}

function verify_binaries() {
    for bin in "${REQUIRED_BINARIES[@]}"; do
        if [ ! -f "${DIST_DIR}/${bin}" ]; then
            echo -e "${RED}Error: ${bin} not found in ${DIST_DIR}${NC}"
            exit 1
        fi
    done
    echo -e "${GREEN}All required binaries present in ${DIST_DIR}${NC}"
}


function terraform_apply() {
    echo -e "${YELLOW}Provisioning Exoscale VM (run_id=${TF_VAR_run_id}, type=${TF_VAR_instance_type})...${NC}"

    pushd "${TF_DIR}" > /dev/null
    tofu init -input=false
    tofu plan
    tofu apply -auto-approve
    VM_IP=$(tofu output -raw instance_ip)
    popd > /dev/null

    echo -e "${GREEN}VM provisioned at ${VM_IP}${NC}"
}

function wait_for_ssh() {
    echo "Waiting for SSH to become available..."
    for i in $(seq 1 30); do
        if ssh ${SSH_OPTS} -i "${EXOSCALE_SSH_KEY_PATH}" \
            "${VM_USER}@${VM_IP}" "echo ok" 2>/dev/null; then
            echo -e "${GREEN}SSH is ready${NC}"
            return
        fi
        echo "  Attempt $i/30: waiting 10s..."
        sleep 10
    done
    echo -e "${RED}ERROR: SSH never became available after 30 attempts${NC}"
    exit 1
}


function upload_binaries() {
    echo -e "${YELLOW}Uploading binaries and test script to VM...${NC}"

    ssh ${SSH_OPTS} -i "${EXOSCALE_SSH_KEY_PATH}" \
        "${VM_USER}@${VM_IP}" "mkdir -p ${VM_DIST_DIR}"

    scp ${SSH_OPTS} -i "${EXOSCALE_SSH_KEY_PATH}" \
        "${DIST_DIR}"/* "${VM_USER}@${VM_IP}:${VM_DIST_DIR}/"

    scp ${SSH_OPTS} -i "${EXOSCALE_SSH_KEY_PATH}" \
        "${SCRIPT_DIR}/test.sh" "${VM_USER}@${VM_IP}:${VM_DIST_DIR}/test.sh"

    ssh ${SSH_OPTS} -i "${EXOSCALE_SSH_KEY_PATH}" \
        "${VM_USER}@${VM_IP}" "chmod +x ${VM_DIST_DIR}/*"

    echo -e "${GREEN}Upload complete${NC}"
}

function configure_test_vm() {
    echo -e "${YELLOW}Configuring test VM...${NC}"

    ssh ${SSH_OPTS} -i "${EXOSCALE_SSH_KEY_PATH}" \
        "${VM_USER}@${VM_IP}" \
        "NYM_TEST_QCOW_IMAGE=/mnt/data/fresh_debian12_cli.qcow2 \
         NYM_TEST_VM_CONFIG=debian12_cli \
         TEST_HARNESS_MNEMONIC='${TEST_HARNESS_MNEMONIC}' \
         TEST_DIST_DIR=${VM_DIST_DIR} \
         bash ${VM_DIST_DIR}/test.sh configure"

    echo -e "${GREEN}Configuration complete${NC}"
}

function run_tests() {
    echo -e "${YELLOW}Running E2E tests...${NC}"

    ssh ${SSH_OPTS} -i "${EXOSCALE_SSH_KEY_PATH}" \
        "${VM_USER}@${VM_IP}" \
        "NYM_TEST_QCOW_IMAGE=/mnt/data/fresh_debian12_cli.qcow2 \
         NYM_TEST_VM_CONFIG=debian12_cli \
         TEST_HARNESS_MNEMONIC='${TEST_HARNESS_MNEMONIC}' \
         TEST_DIST_DIR=${VM_DIST_DIR} \
         TEST_FILTERS='${TEST_FILTERS:-}' \
         SKIP_TESTS='${SKIP_TESTS:-}' \
         bash ${VM_DIST_DIR}/test.sh run-tests"

    echo -e "${GREEN}Tests complete${NC}"
}

function print_vm_info() {
    echo ""
    echo -e "${GREEN}=====================================${NC}"
    echo -e "${GREEN} Done. VM is still running.${NC}"
    echo -e "${GREEN}=====================================${NC}"
    echo ""
    echo -e "  IP:      ${VM_IP}"
    echo -e "  SSH:     ssh ${SSH_OPTS} -i ${EXOSCALE_SSH_KEY_PATH} ${VM_USER}@${VM_IP}"
    echo -e "  Destroy: cd ${TF_DIR} && terraform destroy -auto-approve"
    echo ""
}

validate_env
build_all_in_container
verify_binaries
terraform_apply
wait_for_ssh
upload_binaries
configure_test_vm
run_tests
print_vm_info
