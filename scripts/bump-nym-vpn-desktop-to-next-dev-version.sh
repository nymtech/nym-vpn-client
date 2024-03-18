#! /usr/bin/env bash
#
# Bump the version of nym-vpn-desktop to the next dev version.

# set -x
set -euo pipefail

source "$(dirname "$0")/common.sh"

NAME=nym-vpn-desktop
DIRNAME=nym-vpn-desktop

get_current_cargo_version() {
    cd $DIRNAME
    echo "$(cargo get package.version --entry src-tauri)"
    cd ..
}

run_cargo_set_version() {
    cd $DIRNAME/src-tauri
    local next_version=$1
    local command="cargo set-version $next_version"

    # Run the command with --dry-run option first
    echo "Running in dry-run mode: $command --dry-run"
    $command --dry-run

    ask_for_confirmation "$command"
    cd ../..
}

run_npm_set_version() {
    cd $DIRNAME
    local next_version=$1
    local command="npm version $next_version"
    echo "Running: $command"
    cd ..
}

main() {
    check_unstaged_changes
    confirm_root_directory
    local version=$(get_current_cargo_version)
    local next_version=$(increment_version "$version")

    if [[ -z "$next_version" ]]; then
        echo "Error: next_version is empty. Exiting."
        exit 1
    fi

    run_cargo_set_version "$next_version"
    run_npm_set_version "$next_version"
}

main
