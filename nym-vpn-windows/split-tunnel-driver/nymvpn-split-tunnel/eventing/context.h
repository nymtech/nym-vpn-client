// Copyright 2016-2026 Mullvad VPN AB. All Rights Reserved.
// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#pragma once

#include <wdm.h>
#include <wdf.h>

namespace eventing {

struct CONTEXT {
    // Pended IOCTL requests for inverted call.
    WDFQUEUE RequestQueue;

    WDFSPINLOCK EventQueueLock;

    LIST_ENTRY EventQueue;

    SIZE_T NumEvents;
};

} // namespace eventing
