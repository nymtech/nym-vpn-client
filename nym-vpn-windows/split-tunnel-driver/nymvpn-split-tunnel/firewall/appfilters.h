// Copyright 2016-2026 Mullvad VPN AB. All Rights Reserved.
// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#pragma once

#include <wdm.h>
#include <inaddr.h>
#include <in6addr.h>
#include "../defs/types.h"

//
// This module is used to manage app-specific filters.
//
// App-specific filters apply only to apps being split and use the full app path
// to qualify candidates.
//

namespace firewall::appfilters {

NTSTATUS
Initialize(HANDLE WfpSession, void** Context);

void TearDown(void** Context);

NTSTATUS
TransactionBegin(void* Context);

void TransactionCommit(void* Context);

void TransactionAbort(void* Context);

//
// RegisterFilterBlockAppTunnelTrafficTx2()
//
// Register WFP filters, with linked callout, that will block connections in the tunnel
// from applications being split.
//
// This is used to block existing connections inside the tunnel for applications that are
// just now being split.
//
// All available tunnel addresses must be provided.
//
// IMPORTANT: These functions need to be running inside a WFP transaction as well as a
// local transaction managed by this module.
//
NTSTATUS
RegisterFilterBlockAppTunnelTrafficTx2(
    void* Context, const LOWER_UNICODE_STRING* ImageName, const IN_ADDR* TunnelIpv4, const IN6_ADDR* TunnelIpv6);

NTSTATUS
RemoveFilterBlockAppTunnelTrafficTx2(void* Context, const LOWER_UNICODE_STRING* ImageName);

//
// ResetTx2()
//
// Remove all app-specific blocking filters.
//
// IMPORTANT: This function needs to be running inside a WFP transaction as well as a
// local transaction managed by this module.
//
NTSTATUS
ResetTx2(void* Context);

//
// UpdateFiltersTx2()
//
// Rewrite filters with updated IP addresses.
//
// IMPORTANT: This function needs to be running inside a WFP transaction as well as a
// local transaction managed by this module.
//
NTSTATUS
UpdateFiltersTx2(void* Context, const IN_ADDR* TunnelIpv4, const IN6_ADDR* TunnelIpv6);

} // namespace firewall::appfilters
