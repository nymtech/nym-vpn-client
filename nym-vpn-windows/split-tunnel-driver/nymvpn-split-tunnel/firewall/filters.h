// Copyright 2016-2026 Mullvad VPN AB. All Rights Reserved.
// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#pragma once

#include "wfp.h"
#include "context.h"

namespace firewall {

//
// RegisterFilterBindRedirectIpv4Tx()
//
// Register filter, with linked callout, that will pass all bind requests through the bind callout
// for validation/redirection.
//
// Applicable binds are rewritten for apps being split.
//
// "Tx" (in transaction) suffix means there's no clean-up in failure paths.
//
NTSTATUS
RegisterFilterBindRedirectIpv4Tx(HANDLE WfpSession);

NTSTATUS
RemoveFilterBindRedirectIpv4Tx(HANDLE WfpSession);

//
// RegisterFilterBindRedirectIpv6Tx()
//
// Refer comment on corresponding function for IPv4.
//
NTSTATUS
RegisterFilterBindRedirectIpv6Tx(HANDLE WfpSession);

NTSTATUS
RemoveFilterBindRedirectIpv6Tx(HANDLE WfpSession);

//
// RegisterFilterConnectRedirectIpv4Tx()
//
// Register filter, with linked callout, that will pass all connection requests through
// the connection callout for validation/redirection.
//
// The callout will look for and amend broken localhost client connections.
//
// "Tx" (in transaction) suffix means there's no clean-up in failure paths.
//
NTSTATUS
RegisterFilterConnectRedirectIpv4Tx(HANDLE WfpSession);

NTSTATUS
RemoveFilterConnectRedirectIpv4Tx(HANDLE WfpSession);

//
// RegisterFilterConnectRedirectIpv6Tx()
//
// Refer comment on corresponding function for IPv4.
//
NTSTATUS
RegisterFilterConnectRedirectIpv6Tx(HANDLE WfpSession);

NTSTATUS
RemoveFilterConnectRedirectIpv6Tx(HANDLE WfpSession);

//
// RegisterFilterPermitNonTunnelIpv4Tx()
//
// Register filters, with linked callout, that permit non-tunnel IPv4 traffic
// associated with applications being split.
//
// This ensures winfw filters are not applied to these apps.
//
// The TunnelIpv4 argument is optional but must be specified if the tunnel adapter
// uses an IPv4 interface.
//
// "Tx" (in transaction) suffix means there's no clean-up in failure paths.
//
NTSTATUS
RegisterFilterPermitNonTunnelIpv4Tx(HANDLE WfpSession, const IN_ADDR* TunnelIpv4);

NTSTATUS
RemoveFilterPermitNonTunnelIpv4Tx(HANDLE WfpSession);

//
// RegisterFilterPermitNonTunnelIpv6Tx()
//
// Refer comment on corresponding function for IPv4.
//
NTSTATUS
RegisterFilterPermitNonTunnelIpv6Tx(HANDLE WfpSession, const IN6_ADDR* TunnelIpv6);

NTSTATUS
RemoveFilterPermitNonTunnelIpv6Tx(HANDLE WfpSession);

//
// RegisterFilterBlockTunnelIpv4Tx()
//
// Block all tunnel IPv4 traffic for applications being split.
// To be used when the primary physical adapter doesn't have an IPv4 interface.
//
NTSTATUS
RegisterFilterBlockTunnelIpv4Tx(HANDLE WfpSession, const IN_ADDR* TunnelIp);

NTSTATUS
RemoveFilterBlockTunnelIpv4Tx(HANDLE WfpSession);

//
// RegisterFilterBlockTunnelIpv6Tx()
//
// Refer comment on corresponding function for IPv4.
//
NTSTATUS
RegisterFilterBlockTunnelIpv6Tx(HANDLE WfpSession, const IN6_ADDR* TunnelIp);

NTSTATUS
RemoveFilterBlockTunnelIpv6Tx(HANDLE WfpSession);

} // namespace firewall
