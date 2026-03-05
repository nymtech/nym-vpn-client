// Copyright 2016-2026 Mullvad VPN AB. All Rights Reserved.
// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#include "logging.h"
#include "../util.h"

#include "../trace.h"
#include "logging.tmh"

namespace firewall {

void LogBindRedirect(HANDLE ProcessId, const SOCKADDR_IN* Target, const IN_ADDR* Override) {
    char targetString[32];
    char overrideString[32];

    RtlIpv4AddressToStringA(&Target->sin_addr, targetString);
    RtlIpv4AddressToStringA(Override, overrideString);

    auto const port = ntohs(Target->sin_port);

    DbgPrint(
        "[BIND][%p] Rewriting Non-TCP bind request %s:%d into %s:%d\n",
        ProcessId,
        targetString,
        port,
        overrideString,
        port);
}

void LogBindRedirect(HANDLE ProcessId, const SOCKADDR_IN6* Target, const IN6_ADDR* Override) {
    char targetString[64];
    char overrideString[64];

    RtlIpv6AddressToStringA(&Target->sin6_addr, targetString);
    RtlIpv6AddressToStringA(Override, overrideString);

    auto const port = ntohs(Target->sin6_port);

    DbgPrint(
        "[BIND][%p] Rewriting Non-TCP bind request [%s]:%d into [%s]:%d\n",
        ProcessId,
        targetString,
        port,
        overrideString,
        port);
}

void LogConnectRedirectPass(
    HANDLE ProcessId, const IN_ADDR* LocalAddress, USHORT LocalPort, const IN_ADDR* RemoteAddress, USHORT RemotePort) {
    char localAddrString[32];
    char remoteAddrString[32];

    RtlIpv4AddressToStringA(LocalAddress, localAddrString);
    RtlIpv4AddressToStringA(RemoteAddress, remoteAddrString);

    DbgPrint(
        "[CONN][%p] Passing on opportunity to redirect %s:%d -> %s:%d\n",
        ProcessId,
        localAddrString,
        LocalPort,
        remoteAddrString,
        RemotePort);
}

void LogConnectRedirectPass(
    HANDLE ProcessId, const IN6_ADDR* LocalAddress, USHORT LocalPort, const IN6_ADDR* RemoteAddress, USHORT RemotePort) {
    char localAddrString[64];
    char remoteAddrString[64];

    RtlIpv6AddressToStringA(LocalAddress, localAddrString);
    RtlIpv6AddressToStringA(RemoteAddress, remoteAddrString);

    DbgPrint(
        "[CONN][%p] Passing on opportunity to redirect [%s]:%d -> [%s]:%d\n",
        ProcessId,
        localAddrString,
        LocalPort,
        remoteAddrString,
        RemotePort);
}

void LogConnectRedirect(
    HANDLE ProcessId,
    const IN_ADDR* LocalAddress,
    USHORT LocalPort,
    const IN_ADDR* LocalAddressOverride,
    const IN_ADDR* RemoteAddress,
    USHORT RemotePort) {
    char localAddrString[32];
    char localAddrOverrideString[32];
    char remoteAddrString[32];

    RtlIpv4AddressToStringA(LocalAddress, localAddrString);
    RtlIpv4AddressToStringA(LocalAddressOverride, localAddrOverrideString);
    RtlIpv4AddressToStringA(RemoteAddress, remoteAddrString);

    DbgPrint(
        "[CONN][%p] Rewriting connection on %s:%d as %s:%d -> %s:%d\n",
        ProcessId,
        localAddrString,
        LocalPort,
        localAddrOverrideString,
        LocalPort,
        remoteAddrString,
        RemotePort);
}

void LogConnectRedirect(
    HANDLE ProcessId,
    const IN6_ADDR* LocalAddress,
    USHORT LocalPort,
    const IN6_ADDR* LocalAddressOverride,
    const IN6_ADDR* RemoteAddress,
    USHORT RemotePort) {
    char localAddrString[64];
    char localAddrOverrideString[64];
    char remoteAddrString[64];

    RtlIpv6AddressToStringA(LocalAddress, localAddrString);
    RtlIpv6AddressToStringA(LocalAddressOverride, localAddrOverrideString);
    RtlIpv6AddressToStringA(RemoteAddress, remoteAddrString);

    DbgPrint(
        "[CONN][%p] Rewriting connection on [%s]:%d as [%s]:%d -> [%s]:%d\n",
        ProcessId,
        localAddrString,
        LocalPort,
        localAddrOverrideString,
        LocalPort,
        remoteAddrString,
        RemotePort);
}

void LogPermitConnection(
    HANDLE ProcessId,
    const IN_ADDR* LocalAddress,
    USHORT LocalPort,
    const IN_ADDR* RemoteAddress,
    USHORT RemotePort,
    bool outgoing) {
    char localAddrString[32];
    char remoteAddrString[32];

    RtlIpv4AddressToStringA(LocalAddress, localAddrString);
    RtlIpv4AddressToStringA(RemoteAddress, remoteAddrString);

    auto const direction = outgoing ? "->" : "<-";

    DbgPrint(
        "[PRMT][%p] %s:%d %s %s:%d\n", ProcessId, localAddrString, LocalPort, direction, remoteAddrString, RemotePort);
}

void LogPermitConnection(
    HANDLE ProcessId,
    const IN6_ADDR* LocalAddress,
    USHORT LocalPort,
    const IN6_ADDR* RemoteAddress,
    USHORT RemotePort,
    bool outgoing) {
    char localAddrString[64];
    char remoteAddrString[64];

    RtlIpv6AddressToStringA(LocalAddress, localAddrString);
    RtlIpv6AddressToStringA(RemoteAddress, remoteAddrString);

    auto const direction = outgoing ? "->" : "<-";

    DbgPrint(
        "[PRMT][%p] [%s]:%d %s [%s]:%d\n", ProcessId, localAddrString, LocalPort, direction, remoteAddrString, RemotePort);
}

void LogBlockConnection(
    HANDLE ProcessId,
    const IN_ADDR* LocalAddress,
    USHORT LocalPort,
    const IN_ADDR* RemoteAddress,
    USHORT RemotePort,
    bool outgoing) {
    char localAddrString[32];
    char remoteAddrString[32];

    RtlIpv4AddressToStringA(LocalAddress, localAddrString);
    RtlIpv4AddressToStringA(RemoteAddress, remoteAddrString);

    auto const direction = outgoing ? "->" : "<-";

    DbgPrint(
        "[BLCK][%p] %s:%d %s %s:%d\n", ProcessId, localAddrString, LocalPort, direction, remoteAddrString, RemotePort);
}

void LogBlockConnection(
    HANDLE ProcessId,
    const IN6_ADDR* LocalAddress,
    USHORT LocalPort,
    const IN6_ADDR* RemoteAddress,
    USHORT RemotePort,
    bool outgoing) {
    char localAddrString[64];
    char remoteAddrString[64];

    RtlIpv6AddressToStringA(LocalAddress, localAddrString);
    RtlIpv6AddressToStringA(RemoteAddress, remoteAddrString);

    auto const direction = outgoing ? "->" : "<-";

    DbgPrint(
        "[BLCK][%p] [%s]:%d %s [%s]:%d\n", ProcessId, localAddrString, LocalPort, direction, remoteAddrString, RemotePort);
}

void LogActivatedSplittingMode(SPLITTING_MODE Mode) {
    //
    // This only works because SPLITTING_MODE::MODE_1 is defined as 1, etc.
    //

    NT_ASSERT(static_cast<size_t>(SPLITTING_MODE::MODE_1) == 1 && static_cast<size_t>(SPLITTING_MODE::MODE_9) == 9);

    DbgPrint("Activated splitting mode: %u\n", static_cast<size_t>(Mode));
}

} // namespace firewall
