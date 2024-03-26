#! /usr/bin/env bash
#
# Bump the version of nym-vpn-cli to the next dev version.

# set -x
set -euo pipefail

source "$(dirname "$0")/common.sh"

get_current_version() {
    echo "$(cargo get package.version --entry="nym-vpn-cli")"
}

run_cargo_set_version() {
    local next_version=$1
    local command="cargo set-version -p nym-vpn-cli $next_version"

    # Run the command with --dry-run option first
    echo "Running in dry-run mode: $command --dry-run"
    $command --dry-run

    ask_for_confirmation "$command"
}

main() {
    check_unstaged_changes
    confirm_root_directory
    local version=$(get_current_version)
    local next_version=$(increment_version "$version")

    if [[ -z "$next_version" ]]; then
        echo "Error: next_version is empty. Exiting."
        exit 1
    fi

    run_cargo_set_version "$next_version"
}

main
