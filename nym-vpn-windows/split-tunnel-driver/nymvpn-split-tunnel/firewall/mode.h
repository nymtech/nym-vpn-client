// Copyright 2016-2026 Mullvad VPN AB. All Rights Reserved.
// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#pragma once

#include <wdm.h>
#include <wdf.h>
#include "../ipaddr.h"

namespace firewall {

enum class SPLITTING_MODE {
    // Placeholder
    MODE_0 = 0,

    // Exclude IPv4/IPv6
    MODE_1,

    // Exclude IPv4
    MODE_2,

    // Exclude IPv4, Permit non-tunnel IPv6
    MODE_3,

    // Exclude IPv4, Block tunnel-IPv6
    MODE_4,

    // Exclude IPv6
    MODE_5,

    // Exclude IPv6, Permit non-tunnel IPv4
    MODE_6,

    // Exclude IPv6, Block tunnel-IPv4
    MODE_7,

    // Block tunnel IPv4, Permit non-tunnel IPv6
    MODE_8,

    // Block tunnel IPv6, Permit non-tunnel IPv4
    MODE_9
};

NTSTATUS
DetermineSplittingMode(const ST_IP_ADDRESSES* IpAddresses, SPLITTING_MODE* Mode);

} // namespace firewall
