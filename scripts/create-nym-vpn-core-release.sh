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
DIRNAME=nym-vpn-core
YES=false

# We want to set the workspace version, but I didn't manage to get cargo
# set-version to do this explicitly, only implicitly by specifying the lib
# crate. This seems to trigger a version bump at the workspace level, affecting
# all relevant crates.
PACKAGE=nym-vpn-lib

# Parse arguments
for arg in "$@"; do
    case $arg in
        --yes)
        YES=true
        shift
        ;;
    esac
done

# Check if cargo set-version is installed
if cargo set-version --help >/dev/null 2>&1; then
    echo "cargo set-version is installed."
else
    echo "cargo set-version is not installed. Please install it by running 'cargo install --locked cargo-get'"
fi

# Check if cargo-get is installed
if cargo get --help >/dev/null 2>&1; then
    echo "cargo-get is installed."
else
    echo "cargo-get is not installed. Please install it by running 'cargo install --locked cargo-get'."
fi

cargo_version_bump() {
    cd $DIRNAME
    local package_flags="-p $PACKAGE"
    local command="cargo set-version $package_flags --bump patch"
    echo "Running in dry-run mode: $command --dry-run"
    $command --dry-run
    ask_for_confirmation "$command" "$YES"
    cd ..
}

tag_release() {
    cd $DIRNAME
    local version=$(cargo get workspace.package.version)
    local tag_name="$TAG_BASE_NAME-v$version"
    echo "New version: $version, prepared tag: $tag_name"
    ask_and_tag_release "$tag_name" "$version" "$TAG_BASE_NAME" "$YES"
}

main() {
    check_unstaged_changes
    confirm_root_directory
    check_cargo_utils_installed
    cargo_version_bump
    tag_release
}

main
