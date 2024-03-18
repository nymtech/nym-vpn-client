#! /usr/bin/env bash
#
# Bump the version of nym-vpn-cli to the next dev version.

# set -x
set -euo pipefail

source "$(dirname "$0")/common.sh"

# Function to increment version and append -dev suffix
increment_version() {
    local version=$1
    local IFS='.'  # Internal Field Separator for splitting version parts
    read -r -a parts <<< "$version"  # Read version into an array

    # Validate version format (basic check)
    if [[ ! $version =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
        echo "Error: version=$version format must be X.Y.Z (e.g., 0.0.7)" >&2
        exit 1
    fi

    # Increment the patch version
    ((parts[2]++))

    # Reassemble the version and append -dev suffix
    local new_version="${parts[0]}.${parts[1]}.${parts[2]}-dev"

    echo "$new_version"
}

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
