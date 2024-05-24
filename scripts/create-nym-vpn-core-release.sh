#! /usr/bin/env bash
#
# Bump the version of nym-vpn-cli and tag it.
# Pushing this upstream will then trigger a new release.
#
# Currently it's always the patch version that is bumped, if you need to bump
# another version you have to do it manually, which is a good thing, so that we
# don't bump on accident.

# set -x
set -euo pipefail

source "$(dirname "$0")/common.sh"

TAG_BASE_NAME=nym-vpn-core
PACKAGES=(nym-vpn-lib nym-vpn-cli nym-vpnd nym-vpnc)

cargo_version_bump() {
    local package_flags=""
    for PACKAGE in "${PACKAGES[@]}"; do
        package_flags+=" -p $PACKAGE"
    done
    local command="cargo set-version $package_flags --bump patch"
    echo "Running in dry-run mode: $command --dry-run"
    $command --dry-run
    ask_for_confirmation "$command"
}

assert_same_versions() {
    local first_version=$(cargo get package.version --entry="${PACKAGES[0]}")
    for PACKAGE in "${PACKAGES[@]}"; do
        local version=$(cargo get package.version --entry="$PACKAGE")
        if [[ "$version" != "$first_version" ]]; then
            echo "Error: Version mismatch detected. $PACKAGE has version $version, but expected $first_version."
            exit 1
        fi
    done
    echo "All packages have the same version: $first_version"
}

tag_release() {
    local version=$(cargo get package.version --entry="${PACKAGES[0]}")
    local tag_name="$TAG_BASE_NAME-v$version"
    echo "New version: $version, prepared tag: $tag_name"
    ask_and_tag_release "$tag_name" "$version" "$TAG_BASE_NAME"
}

main() {
    check_unstaged_changes
    confirm_nym_vpn_core_directory
    cargo_version_bump
    assert_same_versions
    tag_release
}

main
