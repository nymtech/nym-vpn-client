// Copyright 2016-2026 Mullvad VPN AB. All Rights Reserved.
// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#pragma once

#include <wdm.h>
#include <wdf.h>
#include "procmon.h"

namespace procmon {

struct CONTEXT {
    // The thread that services queued process events.
    PETHREAD DispatchWorker;

    // Lock to coordinate work on the queue.
    WDFWAITLOCK QueueLock;

    // Queue of incoming process events.
    LIST_ENTRY EventQueue;

    // Event that signals worker should exit.
    KEVENT ExitWorker;

    // Event that signals a new process event has been queued.
    KEVENT WakeUpWorker;

    //
    // Initially events are not dispatched.
    //
    // This variable controls whether an event should only be queued or if the queue should be
    // signalled as well.
    //
    bool DispatchingEnabled;

    //
    // Client callback function that receives process events.
    // Single client only in this layer.
    //
    PROCESS_EVENT_SINK ProcessEventSink;

    //
    // Context to pass along when making the callback.
    //
    void* SinkContext;
};

} // namespace procmon
