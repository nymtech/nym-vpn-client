// Copyright 2016-2026 Mullvad VPN AB. All Rights Reserved.
// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#pragma once

#include "wfp.h"
#include <wdf.h>
#include "../procbroker/procbroker.h"

//
// This module is currently used for pending redirection classifications,
// but could plausibly be extended to handle other types of classifications,
// as and when the need arises.
//

namespace firewall { namespace pending {

struct CONTEXT;

NTSTATUS
Initialize(CONTEXT** Context, procbroker::CONTEXT* ProcessEventBroker);

void TearDown(CONTEXT** Context);

NTSTATUS
PendRequest(
    CONTEXT* Context,
    HANDLE ProcessId,
    void* ClassifyContext,
    UINT64 FilterId,
    UINT16 LayerId,
    FWPS_CLASSIFY_OUT0* ClassifyOut);

NTSTATUS
FailRequest(HANDLE ProcessId, void* ClassifyContext, UINT64 FilterId, UINT16 LayerId, FWPS_CLASSIFY_OUT0* ClassifyOut);

}} // namespace firewall::pending
