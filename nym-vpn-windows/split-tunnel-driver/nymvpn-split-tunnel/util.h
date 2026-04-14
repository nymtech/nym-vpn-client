// Copyright 2016-2026 Mullvad VPN AB. All Rights Reserved.
// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#pragma once

#include <wdm.h>
#include "defs/types.h"

#define bswapw(s) (((s & 0xFF) << 8) | ((s >> 8) & 0xFF))

#define ntohs(s) bswapw(s)
#define htons(s) bswapw(s)

template <typename T>
bool bool_cast(T t) {
    return t != 0;
}

namespace util {

//
// N.B. m has to be a power of two.
//
inline constexpr SIZE_T RoundToMultiple(SIZE_T v, SIZE_T m) {
    return ((v + m - 1) & ~(m - 1));
}

void ReparentList(LIST_ENTRY* Dest, LIST_ENTRY* Src);

//
// GetDevicePathImageName()
//
// Returns the device path of the process binary.
// I.e. the returned path begins with `\Device\HarddiskVolumeX\`
// rather than a symbolic link of the form `\??\C:\`.
//
// A UNICODE_STRING structure and an associated filename buffer
// is allocated and returned.
//
// TODO: The type PEPROCESS seems to require C-linkage on any function
// that uses it as an argument. Fix, maybe.
//
extern "C" NTSTATUS GetDevicePathImageName(PEPROCESS Process, UNICODE_STRING** ImageName);

bool ValidateBufferRange(void const* Buffer, void const* BufferEnd, SIZE_T RangeOffset, SIZE_T RangeLength);

bool IsEmptyRange(void const* Buffer, SIZE_T Length);

//
// AllocateCopyDowncaseString()
//
// Make a lower case copy of the string.
// `Dest->Buffer` is allocated and assigned.
//
NTSTATUS
AllocateCopyDowncaseString(LOWER_UNICODE_STRING* Dest, const UNICODE_STRING* const Src, ST_PAGEABLE Pageable);

void FreeStringBuffer(UNICODE_STRING* String);

void FreeStringBuffer(LOWER_UNICODE_STRING* String);

NTSTATUS
DuplicateString(UNICODE_STRING* Dest, const UNICODE_STRING* Src, ST_PAGEABLE Pageable);

NTSTATUS
DuplicateString(LOWER_UNICODE_STRING* Dest, const LOWER_UNICODE_STRING* Src, ST_PAGEABLE Pageable);

void StopIfDebugBuild();

bool SplittingEnabled(ST_PROCESS_SPLIT_STATUS Status);

bool HybridEnabled(ST_PROCESS_SPLIT_STATUS Status);

bool Equal(const LOWER_UNICODE_STRING* lhs, const LOWER_UNICODE_STRING* rhs);

void Swap(LOWER_UNICODE_STRING* lhs, LOWER_UNICODE_STRING* rhs);

} // namespace util
