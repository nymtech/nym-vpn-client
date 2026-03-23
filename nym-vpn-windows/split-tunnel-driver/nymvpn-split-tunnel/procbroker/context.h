// Copyright 2016-2026 Mullvad VPN AB. All Rights Reserved.
// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#pragma once

#include <wdf.h>
#include "procbroker.h"

namespace procbroker {

struct SUBSCRIPTION {
    LIST_ENTRY ListEntry;
    ST_PB_CALLBACK Callback;
    void* ClientContext;
};

struct CONTEXT {
    WDFWAITLOCK SubscriptionsLock;
    LIST_ENTRY Subscriptions;
};

} // namespace procbroker
