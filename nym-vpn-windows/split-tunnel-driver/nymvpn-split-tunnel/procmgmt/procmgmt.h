// Copyright 2016-2026 Mullvad VPN AB. All Rights Reserved.
// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#pragma once

#include <ntddk.h>
#include <wdf.h>
#include "../procbroker/procbroker.h"
#include "../containers.h"
#include "../eventing/eventing.h"
#include "../firewall/firewall.h"
#include "callbacks.h"

namespace procmgmt {

struct CONTEXT;

NTSTATUS
Initialize(
    CONTEXT** Context,
    procbroker::CONTEXT* ProcessEventBroker,
    PROCESS_REGISTRY_MGMT* ProcessRegistry,
    REGISTERED_IMAGE_MGMT* RegisteredImage,
    eventing::CONTEXT* Eventing,
    firewall::CONTEXT* Firewall,
    ACQUIRE_STATE_LOCK_FN AcquireStateLock,
    RELEASE_STATE_LOCK_FN ReleaseStateLock,
    ENGAGED_STATE_ACTIVE_FN EngagedStateActive,
    void* CallbackContext);

void TearDown(CONTEXT** Context);

//
// Activate()
//
// Until after you call Activate(), all process events are queued.
// Call Activate() after the process registry is populated.
//
void Activate(CONTEXT* Context);

} // namespace procmgmt
