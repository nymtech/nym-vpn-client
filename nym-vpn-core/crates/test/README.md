# Nym VPN end to end test framework

This was forked from (https://github.com/mullvad/mullvadvpn-app) under
SPDX-License-Identifier: GPL-3.0-only

## Project structure

### test-manager

The client part of the testing environment. This program runs on the host and
connects over a virtual serial port to the `test-runner`.

The tests themselves are defined in this package, using the interface provided
by `test-runner`.

### test-runner

The server part of the testing environment. This program runs in guest VMs and
provides the `test-manager` with the building blocks (RPCs) needed to create
tests.

### test-rpc

A support library for the other two packages. Defines an RPC interface,
transports, shared types, etc.
