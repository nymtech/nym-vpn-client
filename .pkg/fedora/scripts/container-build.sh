#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR=/workspace
RPM_ARCH=${1:-}
TOPDIR=/build/rpmbuild
OUT_DIR="${ROOT_DIR}/dist/${RPM_ARCH}"

case "${RPM_ARCH}" in
    aarch64|x86_64) ;;
    *) echo "Usage: $0 aarch64|x86_64" >&2; exit 2 ;;
esac

case "$(uname -m):${RPM_ARCH}" in
    aarch64:aarch64|x86_64:x86_64) ;;
    *)
        echo "Builder architecture $(uname -m) does not match RPM target ${RPM_ARCH}" >&2
        exit 1
        ;;
esac

# shellcheck disable=SC1091
source "${ROOT_DIR}/sources.lock"

mkdir -p \
    "${TOPDIR}/BUILD" \
    "${TOPDIR}/BUILDROOT" \
    "${TOPDIR}/RPMS" \
    "${TOPDIR}/SOURCES" \
    "${TOPDIR}/SPECS" \
    "${TOPDIR}/SRPMS" \
    "${OUT_DIR}" \
    "${ROOT_DIR}/dist/SRPMS"

SOURCE_PATH="${TOPDIR}/SOURCES/${NYM_SOURCE_ARCHIVE}"
curl --fail --location --retry 5 --retry-all-errors \
    --output "${SOURCE_PATH}" "${NYM_SOURCE_URL}"
echo "${NYM_SOURCE_SHA256}  ${SOURCE_PATH}" | sha256sum --check --strict

install -m 0644 "${ROOT_DIR}/nym-vpn.spec" "${TOPDIR}/SPECS/nym-vpn.spec"
install -m 0644 "${ROOT_DIR}/sources.lock" "${TOPDIR}/SOURCES/sources.lock"
install -m 0644 "${ROOT_DIR}/packaging/net.nymtech.NymVPN.desktop" \
    "${TOPDIR}/SOURCES/net.nymtech.NymVPN.desktop"
install -m 0644 "${ROOT_DIR}/packaging/net.nymtech.NymVPN.metainfo.xml" \
    "${TOPDIR}/SOURCES/net.nymtech.NymVPN.metainfo.xml"
install -m 0644 "${ROOT_DIR}/packaging/nym-vpnd.service" \
    "${TOPDIR}/SOURCES/nym-vpnd.service"
install -m 0644 "${ROOT_DIR}/packaging/com.nymvpn.vpnd.unix-access.policy" \
    "${TOPDIR}/SOURCES/com.nymvpn.vpnd.unix-access.policy"

echo "Toolchain versions"
rustc --version
cargo --version
go version
node --version
npm --version
protoc --version

set -o pipefail
rpmbuild -ba --nodeps \
    --define "_topdir ${TOPDIR}" \
    --define "_smp_build_ncpus $(nproc)" \
    --target "${RPM_ARCH}" \
    "${TOPDIR}/SPECS/nym-vpn.spec" 2>&1 | tee "${OUT_DIR}/build.log"

RPM_PATH=$(find "${TOPDIR}/RPMS/${RPM_ARCH}" -maxdepth 1 -type f \
    -name "nym-vpn-${NYM_VERSION}-${NYM_RELEASE}.*.${RPM_ARCH}.rpm" -print -quit)
SRPM_PATH=$(find "${TOPDIR}/SRPMS" -maxdepth 1 -type f \
    -name "nym-vpn-${NYM_VERSION}-${NYM_RELEASE}.*.src.rpm" -print -quit)

if [[ -z "${RPM_PATH}" || -z "${SRPM_PATH}" ]]; then
    echo "rpmbuild completed without the expected binary and source RPMs" >&2
    exit 1
fi

install -m 0644 "${RPM_PATH}" "${OUT_DIR}/"
install -m 0644 "${SRPM_PATH}" "${ROOT_DIR}/dist/SRPMS/"
FINAL_RPM="${OUT_DIR}/$(basename "${RPM_PATH}")"

"${ROOT_DIR}/scripts/validate-rpm.sh" "${FINAL_RPM}" 2>&1 | \
    tee "${OUT_DIR}/validation.log"

(
    cd "${OUT_DIR}"
    : > SHA256SUMS
    for artifact in *.rpm build.log validation.log; do
        sha256sum "${artifact}" >> SHA256SUMS
    done
)

echo "Completed ${FINAL_RPM}"
