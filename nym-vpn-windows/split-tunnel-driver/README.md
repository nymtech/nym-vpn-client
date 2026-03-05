# Nym VPN Split Tunnel driver for Windows

This is a non-PnP KMDF driver suitable for implementing split tunneling in VPN client software. The driver works on Windows 10 and later versions of Windows.

Main features:

- Exclude network traffic from VPN tunnel based on process paths.
- Tracking of arriving and departing processes.
- Atomic process classifications remove any races that could enable traffic leaks.
- Propagation of exclusion flag to child processes.
- Dynamic reconfiguration.
- Blocking of pre-existing unwanted connections.
- Blocking of traffic in cases where it would otherwise leak inside the tunnel.
- Full IPv6 support.

# Development environment

Any recent version of Visual Studio, the WDK, and Windows SDK should work.

The code and project file is known to work with the following combination of software:

- Visual Studio 2022
- WDK v10.0.22621.382
- Windows SDK v10.0.22621.0

It will only build on Windows 10, version 2004 or later.

# Architecture

## Overview

The features mentioned above are wholly implemented in the driver. However, the driver needs a user mode agent to initially and continuously provide it with configuration data.

Specifically, the agent provides a set of application paths that should be excluded from the tunnel. It also communicates the tunnel IPs (IPv4/IPv6) as well as IPs of the primary network interface.

The agent is required to monitor network interfaces and update the driver with new IPs, as they change.

The code in `./testing` gives an example of building blocks needed in the agent. This code is mostly useful for manual testing. For an implementation that's more suited for production use, refer to relevant sections of the [Mullvad VPN app](https://github.com/mullvad/mullvadvpn-app)

## Major subsystems

### Firewall

The firewall subsystem integrates with WFP and manages things such as:

- Socket bind redirection. Excluded apps aren't allowed to bind to `inaddr_any`/`in6addr_any` or bind to the tunnel interface.
- Traffic blocking. Excluded apps aren't allowed to communicate inside the tunnel.
- Traffic approval to override Mullvad VPN app WFP filters and enable excluded apps to communicate outside the tunnel. For more information on how the Mullvad VPN app configures WFP, refer to [Mullvad VPN app security](https://github.com/mullvad/mullvadvpn-app/blob/master/docs/security.md)

This subsystem is fairly large, and part of the reason is because we have to carefully track our modifications in WFP. This is addressed using a transactional system which is employed in parallell with the standard WFP transactions.

### Process management

The driver maintains a complete and updated process tree, and is registered with the system to be notified of arriving/departing processes.

- Examine arriving processes to determine if they should be excluded.
- Discover child processes of excluded processes and propagate exclusion flag.
- Examine departing processes and update WFP and internal state, as applicable.

### Eventing

The eventing subsystem is used to keep the user mode agent up-to-date with important events in the driver. Events are delivered using the inverted call model. An event is sent when e.g. a process starts being excluded, or can't be excluded event though it should be, etc.

## Major data structures

### Process registry

The process registry is used to track all processes in the system. Each entry corresponds to a single process and has information on PID, imagename, and whether the process should be excluded or not.

A single process registry instance is shared between most parts of the driver.

### Registered image

The registered image data structure is used to maintain a set of device paths.

A single instance identifies all executable images that should be excluded from the tunnel. Said instance is shared between IOCTL handlers and the process management subsystem.

## Driver states

The driver uses a fixed sequence of initial state transitions:

1. `ST_DRIVER_STATE_STARTED`. The driver has loaded successfully and completed the most basic initialization. To advance the state, ask the driver to initialize subsystems (`IOCTL_ST_INITIALIZE`).

1. `ST_DRIVER_STATE_INITIALIZED`. All subsystems have been initialized. To advance the state, provide the driver with a complete process tree (`IOCTL_ST_REGISTER_PROCESSES`).

1. `ST_DRIVER_STATE_READY`. The driver is initialized but idle. To advance the state, configure the set of images that should be excluded (`IOCTL_ST_SET_CONFIGURATION`) and register relevant IP addresses (`IOCTL_ST_REGISTER_IP_ADDRESSES`).

1. `ST_DRIVER_STATE_ENGAGED`. The driver is actively excluding traffic, as applicable.

Having reached the _engaged_ state, it's possible to return the driver to the _ready_ state. You can do this by either clearing the configuration (`IOCTL_ST_CLEAR_CONFIGURATION`) or by registering all-zeros IPs on interfaces that are or should be seen as unavailable.

# Operation

The following matrix describes the actions taken by the driver on excluded apps. The actions taken depend on which network interfaces are available.

| \#  | Internet IPv4 | Tunnel IPv4 | Internet IPv6 | Tunnel IPv6 | Actions                                   |
| :-: | :-----------: | :---------: | :-----------: | :---------: | :---------------------------------------- |
|  1  |       x       |      x      |       x       |      x      | Exclude IPv4/IPv6                         |
|  2  |       x       |      x      |               |             | Exclude IPv4                              |
|  3  |       x       |      x      |       x       |             | Exclude IPv4, Permit non-tunnel IPv6      |
|  4  |       x       |      x      |               |      x      | Exclude IPv4, Block tunnel IPv6           |
|  5  |               |             |       x       |      x      | Exclude IPv6                              |
|  6  |       x       |             |       x       |      x      | Exclude IPv6, Permit non-tunnel IPv4      |
|  7  |               |      x      |       x       |      x      | Exclude IPv6, Block tunnel IPv4           |
|  8  |               |      x      |       x       |             | Block tunnel IPv4, Permit non-tunnel IPv6 |
|  9  |       x       |             |               |      x      | Block tunnel IPv6, Permit non-tunnel IPv4 |

**Exclude** means:

- Redirect socket binds away from the tunnel interface.
- Permit non-tunnel traffic.
- Block existing connections in the tunnel.

The explicit block-action is used to prevent traffic from leaking inside the tunnel, in cases where exclusions cannot be applied.

The explicit permit-action is used to override restrictive firewall filters installed by the Mullvad VPN app.

# Limitations

## DNS

Most of the time, DNS requests are not sent from the process where the request originates. The request is first transferred via IPC to the `dnscache` service. If the response is not cached, the `dnscache` service proceeds to make an actual DNS request.

From the point of view of the driver, all DNS requests are made by a particular instance of `svchost`. Because `svchost` is not excluded, and because we can't easily know which process initiated the request, default processing takes precedence and sends the traffic inside the tunnel.

This can be mitigated for individual apps if they can be configured to use DoT/DoH.

## Localhost UDP communications

Excluded apps aren't allowed to bind to `inaddr_any`. In certain cases, if a client socket isn't explicitly bound, the socket will momentarily be seen in the system as binding/bound towards `inaddr_any`, before the correct binding is realized. The driver sees the initial bind and redirects it to the primary network interface.

This means that, for excluded apps, if a UDP socket isn't explicitly bound to `127.0.0.1` before sending, it won't be able to talk to localhost.

Because explicitly bound UDP client sockets are rare, it can be expected that most excluded apps are affected. No generally applicable mitigations are available.

## Multicast reception

When configuring a network socket for multicast packet reception, it's common to both bind `inaddr_any` and join a multicast group on that same address. This is problematic on multi-homed machines such as a machine with an active VPN connection, because the "wrong" interface may be selected behind the scenes.

If we add split tunneling into the mix things become even more complicated. With split tunneling engaged, the socket bind will be redirected to the LAN interface but the group join will still be on `inaddr_any`. This will typically not result in any API errors but the net effect is that incoming traffic can't be properly matched, and will instead be discarded.

No generally applicable mitigations are available.

# License

Copyright (C) 2022 Mullvad VPN AB

Licensed under either of

- GNU General Public License, version 3 or later ([LICENSE-GPL](LICENSE-GPL.md))
- Mozilla Public License, version 2.0 ([LICENSE-MPL](LICENSE-MPL.txt))

at your option.
