#! /usr/bin/env bash
#
# Update nym-vpn-apple to use the latest release of nym-vpnd

# set -x
set -euo pipefail

source "$(dirname "$0")/common.sh"

CORE_DIR=nym-vpn-core
APPLE_SCRIPTS_DIR=nym-vpn-apple/Scripts

get_latest_core_version() {
    local core_version=$(cargo get --entry nym-vpn-core workspace.package.version)

    # Extract the version tag from the JSON response
    latest_version=$(echo "$latest_release" | jq -r .tag_name)

    # Check if jq command was successful
    if [ $? -eq 0 ]; then
        echo "$latest_version"
    else
        echo "Failed to fetch the latest release version." >&2
        return 1
    fi
}

update_daemon_version_on_mac() {
    local core_version=$1
    pushd "$APPLE_SCRIPTS_DIR"
    local command="./UpdateDaemonInfoPlist.sh $core_version"
    echo "Running: $command"
    $command
    popd
}

main() {
    check_unstaged_changes
    confirm_root_directory

    local core_version=$(cargo get --entry nym-vpn-core workspace.package.version)
    update_daemon_version_on_mac $core_version
}

main
