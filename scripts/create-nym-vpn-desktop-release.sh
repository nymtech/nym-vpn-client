#! /usr/bin/env bash
#
# Bump the version of nym-vpn-desktop and tag it.
# Pushing this upstream will then trigger a new release.
#
# Currently it's always the patch version that is bumped, if you need to bump
# another version you have to do it manually, which is a good thing, so that we
# don't bump on accident.

# set -x
set -euo pipefail

source "$(dirname "$0")/common.sh"

NAME=nym-vpn-desktop
DIRNAME=nym-vpn-desktop

cargo_version_bump() {
    cd $DIRNAME/src-tauri
    local command="cargo set-version --bump patch"
    echo "Running in dry-run mode: $command --dry-run"
    $command --dry-run
    ask_for_confirmation "$command"
    cd ../..
}

npm_version_bump() {
    # We can't run this with --dry-run, so let's assume it will be fine
    cd $DIRNAME
    local command_npm="npm version patch"
    echo "Running: $command_npm"
    $command_npm
    cd ..
}

tag_release() {
    cd $DIRNAME
    local version=$(cargo get package.version --entry src-tauri)
    local tag_name="$NAME-v$version"
    echo "New version: $version, Tag: $tag_name"
    ask_and_tag_release "$tag_name" "$version" "$NAME"
}

main() {
    check_unstaged_changes
    confirm_root_directory
    cargo_version_bump
    npm_version_bump
    tag_release
}

main
