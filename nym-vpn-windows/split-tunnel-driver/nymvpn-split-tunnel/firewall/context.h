// Copyright 2016-2026 Mullvad VPN AB. All Rights Reserved.
// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#pragma once

#include <wdm.h>
#include <wdf.h>
#include "firewall.h"
#include "mode.h"
#include "pending.h"
#include "../ipaddr.h"
#include "../procbroker/procbroker.h"
#include "../eventing/eventing.h"

namespace firewall {

struct IP_ADDRESSES_MGMT {
    WDFSPINLOCK Lock;
    ST_IP_ADDRESSES Addresses;
    SPLITTING_MODE SplittingMode;
};

struct TRANSACTION_MGMT {
    // Lock that is held for the duration of a transaction.
    WDFWAITLOCK Lock;

    // Indicator of active transaction.
    bool Active;

    // Thread ID of transaction owner.
    HANDLE OwnerId;
};

struct ACTIVE_FILTERS {
    bool BindRedirectIpv4;
    bool BindRedirectIpv6;
    bool ConnectRedirectIpv4;
    bool ConnectRedirectIpv6;
    bool PermitNonTunnelIpv4;
    bool PermitNonTunnelIpv6;
    bool BlockTunnelIpv4;
    bool BlockTunnelIpv6;
};

struct CONTEXT {
    bool SplittingEnabled;

    ACTIVE_FILTERS ActiveFilters;

    CALLBACKS Callbacks;

    HANDLE WfpSession;

    IP_ADDRESSES_MGMT IpAddresses;

    pending::CONTEXT* PendedClassifications;

    eventing::CONTEXT* Eventing;

    TRANSACTION_MGMT Transaction;

    //
    // Context used with the appfilters module.
    //
    void* AppFiltersContext;
};

} // namespace firewall
