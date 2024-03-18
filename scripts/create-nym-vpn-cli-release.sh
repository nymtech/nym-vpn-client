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

NAME=nym-vpn-cli
PACKAGE=nym-vpn-cli

cargo_version_bump() {
    local command="cargo set-version -p $PACKAGE --bump patch"
    echo "Running in dry-run mode: $command --dry-run"
    $command --dry-run
    ask_for_confirmation "$command"
}

tag_release() {
    local version=$(cargo get package.version --entry="$PACKAGE")
    local tag_name="$NAME-v$version"
    echo "New version: $version, prepared tag: $tag_name"
    ask_and_tag_release "$tag_name" "$version"
}

main() {
    check_unstaged_changes
    confirm_root_directory
    cargo_version_bump
    tag_release
}

main
