#!/usr/bin/env bash
# Copyright 2016-2025 Mullvad VPN AB. All Rights Reserved.
# Copyright 2025 Nym Technologies SA <contact@nymtech.net>
# SPDX-License-Identifier: GPL-3.0-only

# Builds the Linux app in the current build container.
# See the `container-run.sh` script for possible configuration.

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
REPO_DIR="$( cd "$SCRIPT_DIR/.." && pwd )"
cd "$SCRIPT_DIR"

source "$REPO_DIR/scripts/utils/log"

platform=${1-:""}
case $platform in
    linux)
        build_command=("./build.sh")
        shift 1
    ;;
    *)
        log_error "Invalid platform. Specify 'linux' as first argument"
        exit 1
esac

set -x
exec "$SCRIPT_DIR/container-run.sh" "$platform" "${build_command[@]}" "$@"
