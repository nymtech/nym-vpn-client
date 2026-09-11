#!/usr/bin/env bash
set -euo pipefail

RPM_PATH=${1:-}
EXPECTED_VERSION=2026.11.2

if [[ -z "${RPM_PATH}" || ! -f "${RPM_PATH}" ]]; then
    echo "Usage: $0 /path/to/nym-vpn.rpm" >&2
    exit 2
fi

RPM_PATH=$(readlink -f "${RPM_PATH}")
RPM_ARCH=$(rpm -qp --queryformat '%{ARCH}' "${RPM_PATH}")

case "${RPM_ARCH}" in
    aarch64|x86_64) ;;
    *) echo "Unexpected RPM architecture: ${RPM_ARCH}" >&2; exit 1 ;;
esac

echo "Checking RPM digest, metadata, dependencies, and payload"
rpmkeys --checksig "${RPM_PATH}"
rpm -qip "${RPM_PATH}"
rpm -qpR "${RPM_PATH}"
rpm -qplv "${RPM_PATH}"

required_paths=(
    /usr/bin/nym-vpn-app
    /usr/bin/nym-vpnd
    /usr/bin/nym-vpnc
    /usr/bin/nym-exclude
    /usr/bin/nym-socks5-proxy
    /usr/bin/nym-diagnostic
    /usr/lib/systemd/system/nym-vpnd.service
    /usr/share/polkit-1/actions/com.nymvpn.vpnd.unix-access.policy
    /usr/share/applications/net.nymtech.NymVPN.desktop
    /usr/share/icons/hicolor/scalable/apps/nym-vpn.svg
    /usr/share/metainfo/net.nymtech.NymVPN.metainfo.xml
)

PAYLOAD=$(rpm -qpl "${RPM_PATH}")
for path in "${required_paths[@]}"; do
    grep -Fqx "${path}" <<<"${PAYLOAD}" || {
        echo "Required payload path is missing: ${path}" >&2
        exit 1
    }
done

exclude_line=$(rpm -qplv "${RPM_PATH}" | awk '$NF == "/usr/bin/nym-exclude" { print; exit }')
grep -Eq '^-rwsr-xr-x[.]?[[:space:]]+1[[:space:]]+root[[:space:]]+root' <<<"${exclude_line}" || {
    echo "nym-exclude is not root-owned mode 4755 in the RPM payload" >&2
    echo "${exclude_line}" >&2
    exit 1
}

echo "Running rpmlint (the unsigned and intentional setuid findings are reviewed separately)"
set +e
rpmlint "${RPM_PATH}"
rpmlint_status=$?
set -e
if [[ ${rpmlint_status} -ne 0 ]]; then
    echo "rpmlint returned ${rpmlint_status}; continuing with hard validation checks"
fi

EXTRACT_DIR=$(mktemp -d)
trap 'rm -rf "${EXTRACT_DIR}"' EXIT
(
    cd "${EXTRACT_DIR}"
    rpm2cpio "${RPM_PATH}" | cpio --quiet -idm
)

desktop-file-validate \
    "${EXTRACT_DIR}/usr/share/applications/net.nymtech.NymVPN.desktop"
grep -Fqx 'MimeType=x-scheme-handler/nymvpn;' \
    "${EXTRACT_DIR}/usr/share/applications/net.nymtech.NymVPN.desktop"
appstreamcli validate --no-net \
    "${EXTRACT_DIR}/usr/share/metainfo/net.nymtech.NymVPN.metainfo.xml"
xmllint --noout \
    "${EXTRACT_DIR}/usr/share/polkit-1/actions/com.nymvpn.vpnd.unix-access.policy"

stat -c '%U:%G %a' "${EXTRACT_DIR}/usr/bin/nym-exclude" | grep -Fqx 'root:root 4755'
file "${EXTRACT_DIR}/usr/bin/"nym-* | tee /tmp/nym-vpn-file-report.txt
if grep -Fvq "${RPM_ARCH/\//}" /tmp/nym-vpn-file-report.txt; then
    echo "Note: file(1) architecture descriptions use ELF names rather than RPM names"
fi

echo "Installing the RPM with DNF in the clean Fedora 44 builder"
dnf -y --setopt=install_weak_deps=False install "${RPM_PATH}"
rpm -V nym-vpn
systemd-analyze verify /usr/lib/systemd/system/nym-vpnd.service

for binary in \
    nym-vpn-app \
    nym-vpnd \
    nym-vpnc \
    nym-exclude \
    nym-socks5-proxy \
    nym-diagnostic; do
    ldd "/usr/bin/${binary}" | tee "/tmp/${binary}.ldd"
    if grep -Fq 'not found' "/tmp/${binary}.ldd"; then
        echo "Unresolved shared library in ${binary}" >&2
        exit 1
    fi
    test "$(rpm -qf --queryformat '%{VERSION}' "/usr/bin/${binary}")" = "${EXPECTED_VERSION}"
done

for binary in nym-vpn-app nym-vpnd nym-vpnc nym-diagnostic; do
    version_output=$("/usr/bin/${binary}" --version 2>&1)
    grep -F "${EXPECTED_VERSION}" <<<"${version_output}"
done

# nym-exclude and nym-socks5-proxy are daemon protocol helpers, not public
# command-line interfaces. Their owning RPM is the authoritative version.
test "$(stat -c '%U:%G %a' /usr/bin/nym-exclude)" = 'root:root 4755'

echo "Smoke-launching the graphical client under Xvfb"
set +e
timeout 15s xvfb-run -a dbus-run-session -- \
    nym-vpn-app --nosplash >/tmp/nym-vpn-gui-smoke.log 2>&1
gui_status=$?
set -e
case "${gui_status}" in
    0|124) ;;
    *)
        echo "GUI smoke launch failed with status ${gui_status}" >&2
        cat /tmp/nym-vpn-gui-smoke.log >&2
        exit 1
        ;;
esac

echo "RPM validation passed for ${RPM_ARCH}"
