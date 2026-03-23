// Copyright 2016-2026 Mullvad VPN AB. All Rights Reserved.
// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#include "mode.h"

namespace firewall {

NTSTATUS
DetermineSplittingMode(const ST_IP_ADDRESSES* IpAddresses, SPLITTING_MODE* Mode) {
    auto const internetV4 = ip::ValidInternetIpv4Address(IpAddresses);
    auto const internetV6 = ip::ValidInternetIpv6Address(IpAddresses);
    auto const tunnelV4 = ip::ValidTunnelIpv4Address(IpAddresses);
    auto const tunnelV6 = ip::ValidTunnelIpv6Address(IpAddresses);

    if (internetV4 && tunnelV4 && internetV6 && tunnelV6) {
        *Mode = SPLITTING_MODE::MODE_1;
        return STATUS_SUCCESS;
    }

    if (internetV4 && tunnelV4 && !internetV6 && !tunnelV6) {
        *Mode = SPLITTING_MODE::MODE_2;
        return STATUS_SUCCESS;
    }

    if (internetV4 && tunnelV4 && internetV6 && !tunnelV6) {
        *Mode = SPLITTING_MODE::MODE_3;
        return STATUS_SUCCESS;
    }

    if (internetV4 && tunnelV4 && !internetV6 && tunnelV6) {
        *Mode = SPLITTING_MODE::MODE_4;
        return STATUS_SUCCESS;
    }

    if (!internetV4 && !tunnelV4 && internetV6 && tunnelV6) {
        *Mode = SPLITTING_MODE::MODE_5;
        return STATUS_SUCCESS;
    }

    if (internetV4 && !tunnelV4 && internetV6 && tunnelV6) {
        *Mode = SPLITTING_MODE::MODE_6;
        return STATUS_SUCCESS;
    }

    if (!internetV4 && tunnelV4 && internetV6 && tunnelV6) {
        *Mode = SPLITTING_MODE::MODE_7;
        return STATUS_SUCCESS;
    }

    if (!internetV4 && tunnelV4 && internetV6 && !tunnelV6) {
        *Mode = SPLITTING_MODE::MODE_8;
        return STATUS_SUCCESS;
    }

    if (internetV4 && !tunnelV4 && !internetV6 && tunnelV6) {
        *Mode = SPLITTING_MODE::MODE_9;
        return STATUS_SUCCESS;
    }

    return STATUS_INVALID_DISPOSITION;
}

} // namespace firewall
