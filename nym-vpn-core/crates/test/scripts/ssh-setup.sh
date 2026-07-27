#!/usr/bin/env bash
# Copyright 2016-2025 Mullvad VPN AB. All Rights Reserved.
# Copyright 2025 Nym Technologies SA <contact@nymtech.net>
# SPDX-License-Identifier: GPL-3.0-only

set -eu

SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
cd "$SCRIPT_DIR"

# usually /opt/testing
RUNNER_DIR="$1"
UNPRIVILEGED_USER="$2"

# Copy over test runner to correct place

echo "Copying test-runner to $RUNNER_DIR"

mkdir -p "$RUNNER_DIR"

# Copy required files
for file in test-runner; do
    echo "Moving $SCRIPT_DIR/$file to $RUNNER_DIR"
    cp -f "$SCRIPT_DIR/$file" "$RUNNER_DIR"
done

echo "Copying Nym VPN files to $RUNNER_DIR"

for file in nym-vpnc nym-vpnd nym-socks5-proxy; do
    echo "Moving $SCRIPT_DIR/$file to $RUNNER_DIR"
    cp -f "$SCRIPT_DIR/$file" "$RUNNER_DIR"
done


# Unprivileged users need execute rights for some executables
chmod 775 "${RUNNER_DIR}/test-runner"
chmod 775 "${RUNNER_DIR}/nym-vpnd"
chmod 775 "${RUNNER_DIR}/nym-socks5-proxy"
chmod 775 "${RUNNER_DIR}/nym-vpnc"


chown -R root "$RUNNER_DIR/"

# Create service


function setup_systemd_nym {
    VPND_SERVICE_PATH="/etc/systemd/system/nymvpnd.service"

    echo "Creating Nym VPNd service as $VPND_SERVICE_PATH"

    cat > $VPND_SERVICE_PATH << EOF
[Unit]
Description=Nym VPN daemon

[Service]
Type=simple
ExecStart=$RUNNER_DIR/nym-vpnd run-as-service --disable-client-verification
Restart=on-failure
Environment="RUST_LOG=debug"

[Install]
WantedBy=multi-user.target
EOF

    # The base VM image may ship a pre-baked, logged-in account under
    # /var/lib/nym-vpnd.
    echo "Clearing any pre-baked nym-vpnd account state"
    rm -rf /var/lib/nym-vpnd/mainnet

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
Description=Nym VPN Test Runner

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
    echo "MacOS currently not supported"
    exit 1
fi

# Load netfilter helpers before testrunner binds /dev/ttyS0. First use of
# xt_connbytes (delayed_ip_block.sh) otherwise prints to the console and desyncs
# the framed serial mux (CI: implausible length from ASCII "[  2...").
function preload_netfilter_modules {
    echo "Preloading netfilter modules used by censorship block scripts"
    modprobe -q ip_tables || true
    modprobe -q ip6_tables || true
    modprobe -q iptable_filter || true
    modprobe -q ip6table_filter || true
    modprobe -q xt_conntrack || true
    modprobe -q xt_connbytes || true
    iptables -L OUTPUT -n >/dev/null 2>&1 || true
    ip6tables -L OUTPUT -n >/dev/null 2>&1 || true
    # Warm the connbytes match so module init printk is not mid-suite.
    iptables -A OUTPUT -p tcp -d 127.0.0.1 --dport 9 -m connbytes --connbytes 0:1 --connbytes-mode bytes --connbytes-dir reply -j ACCEPT 2>/dev/null \
        && iptables -D OUTPUT -p tcp -d 127.0.0.1 --dport 9 -m connbytes --connbytes 0:1 --connbytes-mode bytes --connbytes-dir reply -j ACCEPT 2>/dev/null \
        || true
}

move_getty_to_another_port
preload_netfilter_modules
setup_systemd
setup_systemd_nym

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
