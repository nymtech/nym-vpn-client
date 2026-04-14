// Copyright 2016-2026 Mullvad VPN AB. All Rights Reserved.
// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#include <ntifs.h>
#include "registeredimage.h"
#include "../util.h"

namespace registeredimage {

struct CONTEXT {
    LIST_ENTRY ListEntry;
    ST_PAGEABLE Pageable;
};

namespace {

//
// FindEntry()
//
// Use at PASSIVE only (APC is OK unless in paging file IO path).
// Presumably because character tables are stored in pageable memory.
//
// Implements case-insensitive comparison.
//
REGISTERED_IMAGE_ENTRY* FindEntry(CONTEXT* Context, UNICODE_STRING* ImageName) {
    for (auto entry = Context->ListEntry.Flink; entry != &Context->ListEntry; entry = entry->Flink) {
        auto candidate = (REGISTERED_IMAGE_ENTRY*)entry;

        if (RtlCompareUnicodeString((UNICODE_STRING*)&candidate->ImageName, ImageName, TRUE) == 0) {
            return candidate;
        }
    }

    return NULL;
}

//
// FindEntryExact()
//
// Use at DISPATCH.
// Implements case-sensitive comparison.
//
REGISTERED_IMAGE_ENTRY* FindEntryExact(CONTEXT* Context, LOWER_UNICODE_STRING* ImageName) {
    for (auto entry = Context->ListEntry.Flink; entry != &Context->ListEntry; entry = entry->Flink) {
        auto candidate = (REGISTERED_IMAGE_ENTRY*)entry;

        if (util::Equal(ImageName, &candidate->ImageName)) {
            return candidate;
        }
    }

    return NULL;
}

NTSTATUS
AddEntryInner(CONTEXT* Context, LOWER_UNICODE_STRING* ImageName, USHORT Flags) {
    //
    // Make a single allocation for the struct and string buffer.
    //

    auto offsetStringBuffer = util::RoundToMultiple(sizeof(REGISTERED_IMAGE_ENTRY), 8);

    auto allocationSize = offsetStringBuffer + ImageName->Length;

    auto const poolType = (Context->Pageable == ST_PAGEABLE::YES) ? PagedPool : NonPagedPool;

    auto record = (REGISTERED_IMAGE_ENTRY*)ExAllocatePoolUninitialized(poolType, allocationSize, ST_POOL_TAG);

    if (record == NULL) {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    auto stringBuffer = (WCHAR*)(((CHAR*)record) + offsetStringBuffer);

    InitializeListHead(&record->ListEntry);

    record->ImageName.Length = ImageName->Length;
    record->ImageName.MaximumLength = ImageName->Length;
    record->ImageName.Buffer = stringBuffer;
    record->Flags = Flags;

    RtlCopyMemory(stringBuffer, ImageName->Buffer, ImageName->Length);

    InsertTailList(&Context->ListEntry, &record->ListEntry);

    return STATUS_SUCCESS;
}

bool RemoveEntryInner(REGISTERED_IMAGE_ENTRY* Entry) {
    if (Entry == NULL) {
        return false;
    }

    RemoveEntryList(&Entry->ListEntry);

    ExFreePoolWithTag(Entry, ST_POOL_TAG);

    return true;
}

} // anonymous namespace

NTSTATUS
Initialize(CONTEXT** Context, ST_PAGEABLE Pageable) {
    auto const poolType = (Pageable == ST_PAGEABLE::YES) ? PagedPool : NonPagedPool;

    *Context = (CONTEXT*)ExAllocatePoolUninitialized(poolType, sizeof(CONTEXT), ST_POOL_TAG);

    if (*Context == NULL) {
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    InitializeListHead(&(*Context)->ListEntry);
    (*Context)->Pageable = Pageable;

    return STATUS_SUCCESS;
}

_IRQL_requires_(PASSIVE_LEVEL) NTSTATUS AddEntry(CONTEXT* Context, UNICODE_STRING* ImageName, USHORT Flags) {
    NT_ASSERT(KeGetCurrentIrql() == PASSIVE_LEVEL);

    //
    // Avoid storing duplicates.
    // FindEntry doesn't care about character casing.
    //

    if (NULL != FindEntry(Context, ImageName)) {
        return STATUS_SUCCESS;
    }

    //
    // Make a lower case string copy.
    //

    UNICODE_STRING lowerImageName;

    auto status = RtlDowncaseUnicodeString(&lowerImageName, ImageName, TRUE);

    if (!NT_SUCCESS(status)) {
        return status;
    }

    status = AddEntryInner(Context, (LOWER_UNICODE_STRING*)&lowerImageName, Flags);

    RtlFreeUnicodeString(&lowerImageName);

    return status;
}

NTSTATUS
AddEntryExact(CONTEXT* Context, LOWER_UNICODE_STRING* ImageName, USHORT Flags) {
    if (NULL != FindEntryExact(Context, ImageName)) {
        return STATUS_SUCCESS;
    }

    return AddEntryInner(Context, ImageName, Flags);
}

bool HasEntry(CONTEXT* Context, UNICODE_STRING* ImageName) {
    auto record = FindEntry(Context, ImageName);

    return record != NULL;
}

bool HasEntryExact(CONTEXT* Context, LOWER_UNICODE_STRING* ImageName) {
    auto record = FindEntryExact(Context, ImageName);

    return record != NULL;
}

USHORT GetEntryFlagsExact(CONTEXT* Context, LOWER_UNICODE_STRING* ImageName) {
    auto record = FindEntryExact(Context, ImageName);

    return (record != NULL) ? record->Flags : 0;
}

bool RemoveEntry(CONTEXT* Context, UNICODE_STRING* ImageName) {
    return RemoveEntryInner(FindEntry(Context, ImageName));
}

bool RemoveEntryExact(CONTEXT* Context, LOWER_UNICODE_STRING* ImageName) {
    return RemoveEntryInner(FindEntryExact(Context, ImageName));
}

bool ForEach(CONTEXT* Context, ST_RI_FOREACH Callback, void* ClientContext) {
    for (auto entry = Context->ListEntry.Flink; entry != &Context->ListEntry; entry = entry->Flink) {
        auto typedEntry = (REGISTERED_IMAGE_ENTRY*)entry;

        if (!Callback(&typedEntry->ImageName, typedEntry->Flags, ClientContext)) {
            return false;
        }
    }

    return true;
}

void Reset(CONTEXT* Context) {
    while (!IsListEmpty(&Context->ListEntry)) {
        auto entry = RemoveHeadList(&Context->ListEntry);

        ExFreePoolWithTag(entry, ST_POOL_TAG);
    }
}

void TearDown(CONTEXT** Context) {
    Reset(*Context);

    ExFreePoolWithTag(*Context, ST_POOL_TAG);

    *Context = NULL;
}

bool IsEmpty(CONTEXT* Context) {
    return bool_cast(IsListEmpty(&Context->ListEntry));
}

} // namespace registeredimage
