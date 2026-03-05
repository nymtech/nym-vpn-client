// Copyright 2016-2026 Mullvad VPN AB. All Rights Reserved.
// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#include "validation.h"
#include "defs/config.h"
#include "defs/process.h"
#include "util.h"
#include <ntintsafe.h>

bool ValidateUserBufferConfiguration(void* Buffer, size_t BufferLength) {
    auto bufferEnd = (UCHAR*)Buffer + BufferLength;

    if (BufferLength < sizeof(ST_CONFIGURATION_HEADER) || bufferEnd < (UCHAR*)Buffer) {
        return false;
    }

    auto header = (ST_CONFIGURATION_HEADER*)Buffer;

    if (header->TotalLength != BufferLength) {
        return false;
    }

    //
    // Verify that the entries reside within the buffer
    //

    SIZE_T entriesSize = 0;

    if (STATUS_SUCCESS != RtlSIZETMult(sizeof(ST_CONFIGURATION_ENTRY), header->NumEntries, &entriesSize)) {
        return false;
    }

    void* stringBuffer = nullptr;

    auto const status = RtlULongPtrAdd(
        (ULONG_PTR)((UCHAR*)Buffer + sizeof(ST_CONFIGURATION_HEADER)), entriesSize, (ULONG_PTR*)&stringBuffer);

    if (STATUS_SUCCESS != status || stringBuffer >= bufferEnd) {
        return false;
    }

    //
    // Verify that all strings reside within the string buffer.
    //

    auto entry = (ST_CONFIGURATION_ENTRY*)(header + 1);

    for (auto i = 0; i < header->NumEntries; ++i, ++entry) {
        auto const valid =
            util::ValidateBufferRange(stringBuffer, bufferEnd, entry->ImageNameOffset, entry->ImageNameLength);

        if (!valid) {
            return false;
        }
    }

    return true;
}

bool ValidateUserBufferProcesses(void* Buffer, size_t BufferLength) {
    auto bufferEnd = (UCHAR*)Buffer + BufferLength;

    if (BufferLength < sizeof(ST_PROCESS_DISCOVERY_HEADER) || bufferEnd < (UCHAR*)Buffer) {
        return false;
    }

    auto header = (ST_PROCESS_DISCOVERY_HEADER*)Buffer;

    if (header->TotalLength != BufferLength) {
        return false;
    }

    //
    // Verify that the entries reside within the buffer
    //

    SIZE_T entriesSize = 0;

    if (STATUS_SUCCESS != RtlSIZETMult(sizeof(ST_PROCESS_DISCOVERY_ENTRY), header->NumEntries, &entriesSize)) {
        return false;
    }

    void* stringBuffer = nullptr;

    auto const status = RtlULongPtrAdd(
        (ULONG_PTR)((UCHAR*)Buffer + sizeof(ST_PROCESS_DISCOVERY_HEADER)), entriesSize, (ULONG_PTR*)&stringBuffer);

    if (STATUS_SUCCESS != status || stringBuffer >= bufferEnd) {
        return false;
    }

    //
    // Verify that all strings reside within the string buffer.
    //

    auto entry = (ST_PROCESS_DISCOVERY_ENTRY*)(header + 1);

    for (auto i = 0; i < header->NumEntries; ++i, ++entry) {
        auto const valid =
            util::ValidateBufferRange(stringBuffer, bufferEnd, entry->ImageNameOffset, entry->ImageNameLength);

        if (!valid) {
            return false;
        }
    }

    return true;
}
