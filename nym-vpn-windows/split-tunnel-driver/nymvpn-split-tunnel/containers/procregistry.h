// Copyright 2016-2026 Mullvad VPN AB. All Rights Reserved.
// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#pragma once

#include <ntddk.h>
#include "../defs/types.h"

namespace procregistry {

struct PROCESS_REGISTRY_ENTRY_SETTINGS {
    // Whether traffic should be split.
    ST_PROCESS_SPLIT_STATUS Split;

    // Whether the process is associated with any firewall filters.
    bool HasFirewallState;
};

struct PROCESS_REGISTRY_ENTRY {
    HANDLE ParentProcessId;
    HANDLE ProcessId;

    PROCESS_REGISTRY_ENTRY_SETTINGS Settings;

    PROCESS_REGISTRY_ENTRY_SETTINGS TargetSettings;

    PROCESS_REGISTRY_ENTRY_SETTINGS PreviousSettings;

    // Device path using all lower-case characters.
    LOWER_UNICODE_STRING ImageName;

    //
    // This is management data initialized and updated
    // by the implementation.
    //
    // It would be inconvenient to store it anywhere else.
    //
    PROCESS_REGISTRY_ENTRY* ParentEntry;
};

struct CONTEXT;

NTSTATUS
Initialize(CONTEXT** Context, ST_PAGEABLE Pageable);

void TearDown(CONTEXT** Context);

void Reset(CONTEXT* Context);

//
// InitializeEntry()
//
// IRQL <= APC.
//
// Initializes `Entry` with provided values and initializes a buffer of
// the correct backing and format for `Entry->ImageName.Buffer`.
//
// The provided `Entry` argument is typically allocated on the stack.
//
NTSTATUS
InitializeEntry(
    CONTEXT* Context,
    HANDLE ParentProcessId,
    HANDLE ProcessId,
    ST_PROCESS_SPLIT_STATUS Split,
    UNICODE_STRING* ImageName,
    PROCESS_REGISTRY_ENTRY* Entry);

//
// AddEntry()
//
// IRQL <= DISPATCH.
//
// On Success:
//
// The `Entry` argument will be copied and `Entry->ImageName.Buffer`
// is taken ownership of.
//
// On failure:
//
// `Entry->ImageName.Buffer` is not taken ownership of.
//
NTSTATUS
AddEntry(CONTEXT* Context, PROCESS_REGISTRY_ENTRY* Entry);

//
// ReleaseEntry()
//
// Memory backing the imagename string buffer is allocated by InitializeEntry().
//
// Use this function to release an entry that could not be added, in order to
// keep details abstracted.
//
void ReleaseEntry(PROCESS_REGISTRY_ENTRY* Entry);

PROCESS_REGISTRY_ENTRY* FindEntry(CONTEXT* Context, HANDLE ProcessId);

bool DeleteEntry(CONTEXT* Context, PROCESS_REGISTRY_ENTRY* Entry);

bool DeleteEntryById(CONTEXT* Context, HANDLE ProcessId);

typedef bool(NTAPI* ST_PR_FOREACH)(PROCESS_REGISTRY_ENTRY* Entry, void* Context);

bool ForEach(CONTEXT* Context, ST_PR_FOREACH Callback, void* ClientContext);

PROCESS_REGISTRY_ENTRY* GetParentEntry(CONTEXT* Context, PROCESS_REGISTRY_ENTRY* Entry);

bool IsEmpty(CONTEXT* Context);

} // namespace procregistry
