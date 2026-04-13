// Copyright 2016-2026 Mullvad VPN AB. All Rights Reserved.
// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#pragma once

#include "../procmon/procmon.h"
#include "../procbroker/procbroker.h"
#include "../containers.h"
#include "../eventing/eventing.h"
#include "../firewall/firewall.h"
#include "callbacks.h"

namespace procmgmt {

struct CONTEXT {
    procmon::CONTEXT* ProcessMonitor;

    procbroker::CONTEXT* ProcessEventBroker;

    PROCESS_REGISTRY_MGMT* ProcessRegistry;

    REGISTERED_IMAGE_MGMT* RegisteredImage;

    eventing::CONTEXT* Eventing;

    firewall::CONTEXT* Firewall;

    ACQUIRE_STATE_LOCK_FN AcquireStateLock;
    RELEASE_STATE_LOCK_FN ReleaseStateLock;
    ENGAGED_STATE_ACTIVE_FN EngagedStateActive;

    void* CallbackContext;
};

} // namespace procmgmt
