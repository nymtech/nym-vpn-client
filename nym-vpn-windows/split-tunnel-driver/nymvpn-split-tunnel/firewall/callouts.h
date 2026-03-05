// Copyright 2016-2026 Mullvad VPN AB. All Rights Reserved.
// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#pragma once

#include <wdm.h>

namespace firewall {

NTSTATUS
RegisterCalloutClassifyBindTx(PDEVICE_OBJECT DeviceObject, HANDLE WfpSession);

NTSTATUS
UnregisterCalloutClassifyBind();

NTSTATUS
RegisterCalloutClassifyConnectTx(PDEVICE_OBJECT DeviceObject, HANDLE WfpSession);

NTSTATUS
UnregisterCalloutClassifyConnect();

NTSTATUS
RegisterCalloutPermitSplitAppsTx(PDEVICE_OBJECT DeviceObject, HANDLE WfpSession);

NTSTATUS
UnregisterCalloutPermitSplitApps();

NTSTATUS
RegisterCalloutBlockSplitAppsTx(PDEVICE_OBJECT DeviceObject, HANDLE WfpSession);

NTSTATUS
UnregisterCalloutBlockSplitApps();

} // namespace firewall
