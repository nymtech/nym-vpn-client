// Copyright 2016-2026 Mullvad VPN AB. All Rights Reserved.
// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#pragma once

#include "wfp.h"
#include "mode.h"

namespace firewall {

void LogBindRedirect(HANDLE ProcessId, const SOCKADDR_IN* Target, const IN_ADDR* Override);

void LogBindRedirect(HANDLE ProcessId, const SOCKADDR_IN6* Target, const IN6_ADDR* Override);

void LogConnectRedirectPass(
    HANDLE ProcessId, const IN_ADDR* LocalAddress, USHORT LocalPort, const IN_ADDR* RemoteAddress, USHORT RemotePort);

void LogConnectRedirectPass(
    HANDLE ProcessId, const IN6_ADDR* LocalAddress, USHORT LocalPort, const IN6_ADDR* RemoteAddress, USHORT RemotePort);

void LogConnectRedirect(
    HANDLE ProcessId,
    const IN_ADDR* LocalAddress,
    USHORT LocalPort,
    const IN_ADDR* LocalAddressOverride,
    const IN_ADDR* RemoteAddress,
    USHORT RemotePort);

void LogConnectRedirect(
    HANDLE ProcessId,
    const IN6_ADDR* LocalAddress,
    USHORT LocalPort,
    const IN6_ADDR* LocalAddressOverride,
    const IN6_ADDR* RemoteAddress,
    USHORT RemotePort);

void LogPermitConnection(
    HANDLE ProcessId,
    const IN_ADDR* LocalAddress,
    USHORT LocalPort,
    const IN_ADDR* RemoteAddress,
    USHORT RemotePort,
    bool outgoing);

void LogPermitConnection(
    HANDLE ProcessId,
    const IN6_ADDR* LocalAddress,
    USHORT LocalPort,
    const IN6_ADDR* RemoteAddress,
    USHORT RemotePort,
    bool outgoing);

void LogBlockConnection(
    HANDLE ProcessId,
    const IN_ADDR* LocalAddress,
    USHORT LocalPort,
    const IN_ADDR* RemoteAddress,
    USHORT RemotePort,
    bool outgoing);

void LogBlockConnection(
    HANDLE ProcessId,
    const IN6_ADDR* LocalAddress,
    USHORT LocalPort,
    const IN6_ADDR* RemoteAddress,
    USHORT RemotePort,
    bool outgoing);

void LogActivatedSplittingMode(SPLITTING_MODE Mode);

}; // namespace firewall
