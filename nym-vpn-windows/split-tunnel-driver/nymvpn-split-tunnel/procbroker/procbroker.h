// Copyright 2016-2026 Mullvad VPN AB. All Rights Reserved.
// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#pragma once

#include <wdm.h>

//
// Process event broker.
//
// Distributes events in the system to notify subsystems when
// processes arrive and depart.
//
// Introduced to break the dependency between "procmgmt" and "firewall".
//

namespace procbroker {

struct CONTEXT;

NTSTATUS
Initialize(CONTEXT** Context);

void TearDown(CONTEXT** Context);

typedef void(NTAPI* ST_PB_CALLBACK)(HANDLE ProcessId, bool Arriving, void* Context);

NTSTATUS
Subscribe(CONTEXT* Context, ST_PB_CALLBACK Callback, void* ClientContext);

void CancelSubscription(CONTEXT* Context, ST_PB_CALLBACK Callback);

void Publish(CONTEXT* Context, HANDLE ProcessId, bool Arriving);

} // namespace procbroker
