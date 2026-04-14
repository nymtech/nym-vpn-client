// Copyright 2016-2026 Mullvad VPN AB. All Rights Reserved.
// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#pragma once

#include <wdm.h>
#include "../ipaddr.h"
#include "../defs/types.h"
#include "../procbroker/procbroker.h"
#include "../eventing/eventing.h"

namespace firewall {

struct CONTEXT;

///////////////////////////////////////////////////////////////////////////////
//
// Callback definitions.
// Client(s) of the firewall subsystem provide the implementations.
//
///////////////////////////////////////////////////////////////////////////////

enum class PROCESS_SPLIT_VERDICT {
    DO_SPLIT,
    DONT_SPLIT,

    // Route traffic according to the source IP the process binds to.
    // No bind/connect rewriting is performed.
    BYPASS,

    // PID is unknown
    UNKNOWN
};

typedef PROCESS_SPLIT_VERDICT(NTAPI* QUERY_PROCESS_FUNC)(HANDLE ProcessId, void* Context);

typedef struct tag_CALLBACKS {
    QUERY_PROCESS_FUNC QueryProcess;
    void* Context;
} CALLBACKS;

///////////////////////////////////////////////////////////////////////////////
//
// Public functions.
//
///////////////////////////////////////////////////////////////////////////////

NTSTATUS
Initialize(
    CONTEXT** Context,
    PDEVICE_OBJECT DeviceObject,
    const CALLBACKS* Callbacks,
    procbroker::CONTEXT* ProcessEventBroker,
    eventing::CONTEXT* Eventing);

NTSTATUS
TearDown(CONTEXT** Context);

NTSTATUS
EnableSplitting(CONTEXT* Context, const ST_IP_ADDRESSES* IpAddresses);

NTSTATUS
DisableSplitting(CONTEXT* Context);

NTSTATUS
RegisterUpdatedIpAddresses(CONTEXT* Context, const ST_IP_ADDRESSES* IpAddresses);

NTSTATUS
TransactionBegin(CONTEXT* Context);

NTSTATUS
TransactionCommit(CONTEXT* Context, bool ForceAleReauthorization = false);

NTSTATUS
TransactionAbort(CONTEXT* Context);

NTSTATUS
RegisterAppBecomingSplitTx(CONTEXT* Context, const LOWER_UNICODE_STRING* ImageName);

NTSTATUS
RegisterAppBecomingUnsplitTx(CONTEXT* Context, const LOWER_UNICODE_STRING* ImageName);

} // namespace firewall
