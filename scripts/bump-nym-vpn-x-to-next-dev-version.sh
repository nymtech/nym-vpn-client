#! /usr/bin/env bash
#
# Bump the version of nym-vpn-x to the next dev version.

# set -x
set -euo pipefail

source "$(dirname "$0")/common.sh"

NAME=nym-vpn-x
DIRNAME=nym-vpn-x
YES=false

# Parse arguments
for arg in "$@"; do
    case $arg in
        --yes)
        YES=true
        shift
        ;;
    esac
done

get_current_cargo_version() {
    cd $DIRNAME
    echo "$(cargo get package.version --entry src-tauri)"
    cd ..
}

run_cargo_set_version() {
    cd $DIRNAME/src-tauri
    local next_version=$1
    local yes=$2
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
    $command
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

    run_cargo_set_version "$next_version" "$YES"
    run_npm_set_version "$next_version"
    git_commit_new_dev_version "$next_version" "$NAME"
}

main
