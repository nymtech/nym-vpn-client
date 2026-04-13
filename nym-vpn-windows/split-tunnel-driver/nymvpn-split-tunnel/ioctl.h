// Copyright 2016-2026 Mullvad VPN AB. All Rights Reserved.
// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#pragma once

#include <ntddk.h>
#include <wdf.h>
#include "containers/registeredimage.h"

namespace ioctl {

//
// Initialize()
//
// Initialize subsystems and device context.
//
NTSTATUS
Initialize(WDFDEVICE Device);

//
// SetConfigurationPrepare()
//
// Parse client buffer into a registeredimage instance with per-entry flags.
//
// This should be called at PASSIVE, and the actual updating and
// state transition may be performed at DISPATCH.
//
NTSTATUS
SetConfigurationPrepare(WDFREQUEST Request, registeredimage::CONTEXT** Imageset);

NTSTATUS
SetConfiguration(WDFDEVICE Device, registeredimage::CONTEXT* Imageset);

void GetConfigurationComplete(WDFDEVICE Device, WDFREQUEST Request);

NTSTATUS
ClearConfiguration(WDFDEVICE Device);

NTSTATUS
RegisterProcesses(WDFDEVICE Device, WDFREQUEST Request);

NTSTATUS
RegisterIpAddresses(WDFDEVICE Device, WDFREQUEST Request);

void GetIpAddressesComplete(WDFDEVICE Device, WDFREQUEST Request);

void GetStateComplete(WDFDEVICE Device, WDFREQUEST Request);

void QueryProcessComplete(WDFDEVICE Device, WDFREQUEST Request);

void ResetComplete(WDFDEVICE Device, WDFREQUEST Request);

NTSTATUS
Reset(WDFDEVICE Device);

} // namespace ioctl
