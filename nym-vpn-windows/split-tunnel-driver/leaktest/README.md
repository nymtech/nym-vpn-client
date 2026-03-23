# Traffic leak tests

These are a set of semi-automated tests that can be used to test for conformance and deficiencies in VPN client software.

All tests are built into a command line launcher. The first argument on the command line specifies the test to run, e.g. `leaktest gen1`. Subsequent arguments are passed on to the test implementation.

As a test completes, an over all status of passed/failed is presented.

**Note: Testing results should be interpreted by a qualified developer. Tests can be easily tricked into failing or succeeding if there's a preferred outcome. This is not the point of these tests. They should be used to aid in testing and as a part of a larger toolbox.**

## Requirements

1. VPN client software of your choice.

1. An instance of Kong tcpbin, accessible across the Internet.

1. Updated configuration file.

## Configuration

Most tests depend on the same basic configuration elements. The configuration should be stored in a file named `leaktest.settings`, which should be placed alongside `leaktest.exe`.

An example configuration file might contain:

```
TunnelAdapter=Mullvad
LanAdapter=Ethernet
TcpBinServerIp=1.2.3.4
TcpBinEchoPort=30000
TcpBinEchoPortUdp=40000
```

# Overview of tests

## General tests

### `gen1`

Evaluate whether VPN client state changes have momentary leaks.

The test aggressively tries to send and receive data to see if any one of the requests being made are successful. The delay between iterations is configurable.

`leaktest gen1 [tcp/udp] [delay-ms]`

## Split tunnel tests

### `st1`

Evaluate whether different kinds of binds are correctly handled.

1. Bind to tunnel interface and validate that bind is redirected to lan interface.
1. Bind to lan interface and validate that bind is successful.
1. Do not bind before connecting. Validate that bind is directed to lan interface.

`leaktest st1 [tcp/udp]`

### `st2`

Evaluate whether existing connections are blocked when an app becomes excluded.

It's desirable that an app exists exclusively on one side of the tunnel, and never on both sides at the same time. For an app that has active connections in the tunnel, and then becomes excluded, we verify that the existing connections are blocked.

`leaktest st2 [tcp/udp]`

### `st3`

Evaluate whether excluded connections are blocked when an app stops being excluded.

This is similar to `st2`, in that it's testing for the same properties, but the inverse scenario.

`leaktest st3 [tcp/udp]`

### `st4`

Evaluate whether DNS requests can be moved outside tunnel.

Due to how DNS requests and caching of responses is architectured in Windows, it's rare that a split tunneling solution will be able to move requests outside the tunnel. In this test, we're issuing DNS requests from an app being excluded, using the Windows infrastructure, and monitoring for DNS traffic outside the tunnel.

`leaktest st4`

### `st5`

Evaluate whether child processes are automatically and atomically handled.

For an app being excluded, it's desirable that any child processes are treated as belonging to one and the same context as the parent. It's plausible to believe that an app would share data with its child processes, and therefore, they should all operate on the same side of the tunnel.

The test is interesting because some VPN client software has failed to account for this. Yet other VPN software do account for it, but have massive races.

`leaktest st5`

### `st6`

Evaluate whether binds to localhost are correctly **NOT** being redirected.

When redirecting binds, one has to be mindful about attempted binds towards localhost. These binds must not be redirected because it would expose local services on the LAN.

`leaktest st6`

### `st7`

Evaluate whether existing child processes become excluded with their parent.

Again, because processes are likely to share data with their children, it makes sense that any existing child processes become excluded with their parent.

`leaktest st7`
