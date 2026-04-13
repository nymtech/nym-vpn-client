// Copyright 2016-2026 Mullvad VPN AB. All Rights Reserved.
// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#pragma once

#include <wdm.h>
#include "../defs/types.h"

namespace registeredimage {

struct REGISTERED_IMAGE_ENTRY {
    LIST_ENTRY ListEntry;

    // Device path using all lower-case characters.
    LOWER_UNICODE_STRING ImageName;

    // Combination of ST_CONFIGURATION_ENTRY_FLAG_* values.
    USHORT Flags;
};

struct CONTEXT;

NTSTATUS
Initialize(CONTEXT** Context, ST_PAGEABLE Pageable);

//
// AddEntry()
//
// IRQL == PASSIVE_LEVEL
//
// Converts imagename to lower case before creating an entry.
//
_IRQL_requires_(PASSIVE_LEVEL) NTSTATUS AddEntry(CONTEXT* Context, UNICODE_STRING* ImageName, USHORT Flags = 0);

//
// AddEntryExact()
//
// IRQL <= DISPATCH
//
// Creates a new entry with the `ImageName` argument exactly as passed.
//
NTSTATUS
AddEntryExact(CONTEXT* Context, LOWER_UNICODE_STRING* ImageName, USHORT Flags = 0);

//
// HasEntry()
//
// IRQL <= APC
//
// Compares existing entries against `ImageName` without regard to character casing.
//
bool HasEntry(CONTEXT* Context, UNICODE_STRING* ImageName);

//
// HasEntryExact()
//
// IRQL <= DISPATCH
//
// Compares existing entries against case-sensitive `ImageName` argument.
//
bool HasEntryExact(CONTEXT* Context, LOWER_UNICODE_STRING* ImageName);

//
// RemoveEntry()
//
// IRQL <= APC
//
// Searches for and removes entry matching `ImageName` without regard to character casing.
//
bool RemoveEntry(CONTEXT* Context, UNICODE_STRING* ImageName);

//
// RemoveEntryExact()
//
// IRQL <= DISPATCH
//
// Searches for and removes entry using case-sensitive matching of `ImageName`.
//
bool RemoveEntryExact(CONTEXT* Context, LOWER_UNICODE_STRING* ImageName);

//
// GetEntryFlagsExact()
//
// IRQL <= DISPATCH
//
// Returns the flags stored with the entry matching `ImageName`, or 0 if not found.
//
USHORT GetEntryFlagsExact(CONTEXT* Context, LOWER_UNICODE_STRING* ImageName);

typedef bool(NTAPI* ST_RI_FOREACH)(const LOWER_UNICODE_STRING* ImageName, USHORT Flags, void* Context);

bool ForEach(CONTEXT* Context, ST_RI_FOREACH Callback, void* ClientContext);

void Reset(CONTEXT* Context);

void TearDown(CONTEXT** Context);

bool IsEmpty(CONTEXT* Context);

} // namespace registeredimage
