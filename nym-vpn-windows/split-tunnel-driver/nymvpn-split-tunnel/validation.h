// Copyright 2016-2026 Mullvad VPN AB. All Rights Reserved.
// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#pragma once

#include <wdm.h>

//
// ValidateUserBufferConfiguration()
//
// Validates configuration data sent by user mode.
//
bool ValidateUserBufferConfiguration(void* Buffer, size_t BufferLength);

//
// ValidateUserBufferProcesses()
//
// Validates process data sent by user mode.
//
bool ValidateUserBufferProcesses(void* Buffer, size_t BufferLength);
