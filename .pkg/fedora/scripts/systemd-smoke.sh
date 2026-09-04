#!/usr/bin/env bash
set -euo pipefail

CONTAINER_ENGINE=${CONTAINER_ENGINE:-podman}
RPM_PATH=${1:-}
ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)

if [[ -z "${RPM_PATH}" || ! -f "${RPM_PATH}" ]]; then
    echo "Usage: $0 /path/to/nym-vpn.rpm" >&2
    exit 2
fi

RPM_PATH=$(cd "$(dirname "${RPM_PATH}")" && pwd)/$(basename "${RPM_PATH}")
case "$(basename "${RPM_PATH}")" in
    *.aarch64.rpm) platform=linux/arm64 ;;
    *.x86_64.rpm) platform=linux/amd64 ;;
    *) echo "Cannot determine architecture from ${RPM_PATH}" >&2; exit 2 ;;
esac

name="nym-vpn-systemd-smoke-$RANDOM-$$"
image="localhost/nym-vpn-systemd-smoke:fedora44-${platform#linux/}"
cleanup() {
    "${CONTAINER_ENGINE}" rm --force "${name}" >/dev/null 2>&1 || true
}
trap cleanup EXIT

"${CONTAINER_ENGINE}" build \
    --platform "${platform}" \
    --tag "${image}" \
    --file "${ROOT_DIR}/packaging/systemd-smoke.Containerfile" \
    "${ROOT_DIR}"

"${CONTAINER_ENGINE}" run --detach \
    --name "${name}" \
    --platform "${platform}" \
    --privileged \
    --systemd=always \
    --volume "${RPM_PATH}:/tmp/nym-vpn.rpm:ro" \
    "${image}" >/dev/null

for _ in $(seq 1 30); do
    if "${CONTAINER_ENGINE}" exec "${name}" systemctl is-system-running --wait >/dev/null 2>&1; then
        break
    fi
    sleep 1
done

"${CONTAINER_ENGINE}" exec "${name}" systemctl start NetworkManager
"${CONTAINER_ENGINE}" exec "${name}" \
    dnf -y --setopt=install_weak_deps=False install /tmp/nym-vpn.rpm

"${CONTAINER_ENGINE}" exec "${name}" systemctl is-enabled nym-vpnd.service
"${CONTAINER_ENGINE}" exec "${name}" systemctl is-active nym-vpnd.service

cli_ok=false
for _ in $(seq 1 20); do
    if "${CONTAINER_ENGINE}" exec "${name}" timeout 10s nym-vpnc status; then
        cli_ok=true
        break
    fi
    sleep 2
done
if [[ "${cli_ok}" != true ]]; then
    "${CONTAINER_ENGINE}" exec "${name}" journalctl -u nym-vpnd.service --no-pager -n 100
    echo "nym-vpnc could not communicate with nym-vpnd" >&2
    exit 1
fi

"${CONTAINER_ENGINE}" exec "${name}" mkdir -p \
    /etc/nym /var/lib/nym-vpnd /var/log/nym-vpnd
"${CONTAINER_ENGINE}" exec "${name}" touch \
    /etc/nym/rpm-preservation-test \
    /var/lib/nym-vpnd/rpm-preservation-test \
    /var/log/nym-vpnd/rpm-preservation-test

# Disabling an active daemon models a user opting out of future boot-time
# activation. Reinstall must restart the active process without re-enabling it.
"${CONTAINER_ENGINE}" exec "${name}" systemctl disable nym-vpnd.service
"${CONTAINER_ENGINE}" exec "${name}" systemctl is-active nym-vpnd.service
"${CONTAINER_ENGINE}" exec "${name}" dnf -y reinstall /tmp/nym-vpn.rpm
if "${CONTAINER_ENGINE}" exec "${name}" systemctl is-enabled nym-vpnd.service; then
    echo "RPM reinstall unexpectedly re-enabled nym-vpnd" >&2
    exit 1
fi
"${CONTAINER_ENGINE}" exec "${name}" systemctl is-active nym-vpnd.service

"${CONTAINER_ENGINE}" exec "${name}" dnf -y remove nym-vpn
if "${CONTAINER_ENGINE}" exec "${name}" systemctl is-active nym-vpnd.service; then
    echo "nym-vpnd is still active after final removal" >&2
    exit 1
fi
for marker in \
    /etc/nym/rpm-preservation-test \
    /var/lib/nym-vpnd/rpm-preservation-test \
    /var/log/nym-vpnd/rpm-preservation-test; do
    "${CONTAINER_ENGINE}" exec "${name}" test -f "${marker}"
done

echo "systemd lifecycle smoke test passed"
