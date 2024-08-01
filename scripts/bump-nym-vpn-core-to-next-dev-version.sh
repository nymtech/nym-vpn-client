#! /usr/bin/env bash
#
# Bump the version of nym-vpn-core to the next dev version.

# set -x
set -euo pipefail

source "$(dirname "$0")/common.sh"

TAG_BASE_NAME="nym-vpn-core"
DIRNAME="nym-vpn-core"
PACKAGE=nym-vpn-lib

get_current_version() {
    echo "$(cargo get workspace.package.version)"
}

run_cargo_set_version() {
    local next_version=$1

    local package_flags="-p $PACKAGE"
    local command="cargo set-version $package_flags $next_version"

    # Run the command with --dry-run option first
    echo "Running in dry-run mode: $command --dry-run"
    $command --dry-run

    ask_for_confirmation "$command"
}

main() {
    check_unstaged_changes
    confirm_root_directory
    cd $DIRNAME
    local version=$(get_current_version)
    local next_version=$(increment_version "$version")

    if [[ -z "$next_version" ]]; then
        echo "Error: next_version is empty. Exiting."
        exit 1
    fi

    run_cargo_set_version "$next_version"
    git_commit_new_dev_version "$next_version" "$TAG_BASE_NAME"
}

main
