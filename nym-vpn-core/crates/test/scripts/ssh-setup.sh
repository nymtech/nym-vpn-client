#!/usr/bin/env bash
# Copyright 2016-2025 Mullvad VPN AB. All Rights Reserved.
# Copyright 2025 Nym Technologies SA <contact@nymtech.net>
# SPDX-License-Identifier: GPL-3.0-only

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

# usually /opt/testing
RUNNER_DIR="$1"
APP_PACKAGE="$2"
PREVIOUS_APP="$3"
UI_RUNNER="$4"
UNPRIVILEGED_USER="$5"

# Copy over test runner to correct place

echo "Copying test-runner to $RUNNER_DIR"

mkdir -p "$RUNNER_DIR"

# Copy required files
for file in test-runner connection-checker "$APP_PACKAGE"; do
    echo "Moving $SCRIPT_DIR/$file to $RUNNER_DIR"
    cp -f "$SCRIPT_DIR/$file" "$RUNNER_DIR"
done

echo "Copying Nym VPN files to $RUNNER_DIR"

for file in nym-vpnc nym-vpnd; do
    echo "Moving $SCRIPT_DIR/$file to $RUNNER_DIR"
    cp -f "$SCRIPT_DIR/$file" "$RUNNER_DIR"
done

# Copy optional files if they exist and are not empty
if [[ -n "$PREVIOUS_APP" && -f "$SCRIPT_DIR/$PREVIOUS_APP" ]]; then
    echo "Moving $SCRIPT_DIR/$PREVIOUS_APP to $RUNNER_DIR"
    cp -f "$SCRIPT_DIR/$PREVIOUS_APP" "$RUNNER_DIR"
fi

if [[ -n "$UI_RUNNER" && -f "$SCRIPT_DIR/$UI_RUNNER" ]]; then
    echo "Moving $SCRIPT_DIR/$UI_RUNNER to $RUNNER_DIR"
    cp -f "$SCRIPT_DIR/$UI_RUNNER" "$RUNNER_DIR"
fi


# Unprivileged users need execute rights for some executables
chmod 775 "${RUNNER_DIR}/${APP_PACKAGE}"
chmod 775 "${RUNNER_DIR}/test-runner"
chmod 775 "${RUNNER_DIR}/connection-checker"
chmod 775 "${RUNNER_DIR}/nym-vpnd"
chmod 775 "${RUNNER_DIR}/nym-vpnc"

if [[ -n "$UI_RUNNER" && -f "${RUNNER_DIR}/$UI_RUNNER" ]]; then
    chmod 775 "${RUNNER_DIR}/$UI_RUNNER"
fi

chown -R root "$RUNNER_DIR/"

# Create service

function setup_macos {
    RUNNER_PLIST_PATH="/Library/LaunchDaemons/net.mullvad.testunner.plist"

    echo "Creating test runner service as $RUNNER_PLIST_PATH"

    cat > $RUNNER_PLIST_PATH << EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple Computer//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>net.mullvad.testrunner</string>

    <key>ProgramArguments</key>
    <array>
        <string>$RUNNER_DIR/test-runner</string>
        <string>/dev/tty.virtio</string>
        <string>serve</string>
    </array>

    <key>UserName</key>
    <string>root</string>

    <key>RunAtLoad</key>
    <true/>

    <key>KeepAlive</key>
    <true/>

    <key>StandardOutPath</key>
    <string>/tmp/runner.out</string>

    <key>StandardErrorPath</key>
    <string>/tmp/runner.err</string>

    <key>EnvironmentVariables</key>
    <dict>
        <key>PATH</key>
        <string>/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin:/usr/local/sbin</string>
    </dict>
</dict>
</plist>
EOF

    create_test_user_macos

    echo "Starting test runner service"

    launchctl load -w $RUNNER_PLIST_PATH
}

function create_test_user_macos {
    echo "Adding test user account"
    sysadminctl -addUser "$UNPRIVILEGED_USER" -fullName "$UNPRIVILEGED_USER" -password "$UNPRIVILEGED_USER"
}

function setup_systemd_nym {
    VPND_SERVICE_PATH="/etc/systemd/system/nymvpnd.service"

    echo "Creating Nym VPNd service as $VPND_SERVICE_PATH"

    cat > $VPND_SERVICE_PATH << EOF
[Unit]
Description=Nym VPN daemon

[Service]
Type=simple
ExecStart=$RUNNER_DIR/nym-vpnd run-as-service
Restart=on-failure
Environment="RUST_LOG=debug"

[Install]
WantedBy=multi-user.target
EOF

    echo "Starting Nym VPNd service"

    semanage fcontext -a -t bin_t "$RUNNER_DIR/.*" &> /dev/null || true

    # create_test_user_linux

    systemctl daemon-reload
    systemctl enable nymvpnd.service
    systemctl start nymvpnd.service
}

function setup_systemd {
    RUNNER_SERVICE_PATH="/etc/systemd/system/testrunner.service"

    echo "Creating test runner service as $RUNNER_SERVICE_PATH"

    # adding restart on failure because sometimes the runner panicks trying to bind to a socket
    cat > $RUNNER_SERVICE_PATH << EOF
[Unit]
Description=Mullvad Test Runner

[Service]
ExecStart=$RUNNER_DIR/test-runner /dev/ttyS0 serve
Restart=on-failure

[Install]
WantedBy=multi-user.target
EOF

    echo "Starting test runner service"

    semanage fcontext -a -t bin_t "$RUNNER_DIR/.*" &> /dev/null || true

    create_test_user_linux

    systemctl enable testrunner.service
    systemctl start testrunner.service
}

# getty by default uses the first serial port for login, but that conflicts with
# the serial test-manager uses for communiction with test-runner
function move_getty_to_another_port {
    sudo systemctl stop serial-getty@ttyS0
    sudo systemctl disable serial-getty@ttyS0

    sudo systemctl start serial-getty@ttyS1
    sudo systemctl enable serial-getty@ttyS1
}

function create_test_user_linux {
    echo "Adding test user account"
    useradd -m "$UNPRIVILEGED_USER"
    echo "$UNPRIVILEGED_USER:$UNPRIVILEGED_USER" | chpasswd
}

if [[ "$(uname -s)" == "Darwin" ]]; then
    setup_macos
    exit 0
fi

setup_systemd
setup_systemd_nym
move_getty_to_another_port

# Run apt with some arguments
robust_apt () {
    # We don't want to fail due to the global apt lock being
    # held, which happens sporadically. It is fine to wait for
    # some time if it means that the test run can continue.
    DEBIAN_FRONTEND=noninteractive apt-get -qy -o DPkg::Lock::Timeout=60 "$@"
}

function install_packages_apt {
    echo "Installing required apt packages"
    robust_apt update
    robust_apt install xvfb wireguard-tools curl
    if ! which ping &>/dev/null; then
        robust_apt install iputils-ping
    fi
    curl -fsSL https://get.docker.com | sh
}

# Install required packages
if which apt &>/dev/null; then
    install_packages_apt
elif which dnf &>/dev/null; then
    dnf install -y xorg-x11-server-Xvfb wireguard-tools podman
fi
