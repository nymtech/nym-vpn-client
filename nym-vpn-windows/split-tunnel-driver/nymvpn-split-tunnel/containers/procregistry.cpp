// Copyright 2016-2026 Mullvad VPN AB. All Rights Reserved.
// Copyright 2026 Nym Technologies SA <contact@nymtech.net>
// SPDX-License-Identifier: GPL-3.0-only

#include "procregistry.h"
#include "../util.h"

namespace procregistry {

struct CONTEXT {
    RTL_AVL_TABLE Tree;
    ST_PAGEABLE Pageable;
};

namespace {

RTL_GENERIC_COMPARE_RESULTS
TreeCompareRoutine(__in struct _RTL_AVL_TABLE* Table, __in PVOID FirstStruct, __in PVOID SecondStruct) {
    UNREFERENCED_PARAMETER(Table);

    auto first = ((PROCESS_REGISTRY_ENTRY*)FirstStruct)->ProcessId;
    auto second = ((PROCESS_REGISTRY_ENTRY*)SecondStruct)->ProcessId;

    if (first < second) {
        return GenericLessThan;
    }

    if (first > second) {
        return GenericGreaterThan;
    }

    return GenericEqual;
}

PVOID
TreeAllocateRoutineNonPaged(__in struct _RTL_AVL_TABLE* Table, __in CLONG ByteSize) {
    UNREFERENCED_PARAMETER(Table);

    return ExAllocatePoolUninitialized(NonPagedPool, ByteSize, ST_POOL_TAG);
}

PVOID
TreeAllocateRoutinePaged(__in struct _RTL_AVL_TABLE* Table, __in CLONG ByteSize) {
    UNREFERENCED_PARAMETER(Table);

    return ExAllocatePoolUninitialized(PagedPool, ByteSize, ST_POOL_TAG);
}

VOID TreeFreeRoutine(__in struct _RTL_AVL_TABLE* Table, __in PVOID Buffer) {
    UNREFERENCED_PARAMETER(Table);

    ExFreePoolWithTag(Buffer, ST_POOL_TAG);
}

//
// ClearDepartingParentLink()
//
// `Entry` is an enumerated entry in the tree.
// `Context` is the PID corresponding to an entry that's being removed from the
// tree.
//
// If `Entry` is a child of `Context` it needs to be updated to indicate that
// the parent process is no longer available.
//
bool NTAPI ClearDepartingParentLink(PROCESS_REGISTRY_ENTRY* Entry, void* Context) {
    auto departingProcessId = (HANDLE)Context;

    if (Entry->ParentProcessId == departingProcessId) {
        Entry->ParentProcessId = 0;
        Entry->ParentEntry = NULL;
    }

    return true;
}

bool InnerDeleteEntry(CONTEXT* Context, PROCESS_REGISTRY_ENTRY* Entry) {
    LOWER_UNICODE_STRING imageName = { 0 };

    util::Swap(&Entry->ImageName, &imageName);

    auto const status = RtlDeleteElementGenericTableAvl(&Context->Tree, Entry);

    if (!status) {
        util::Swap(&Entry->ImageName, &imageName);

        return false;
    }

    if (imageName.Buffer != NULL) {
        util::FreeStringBuffer(&imageName);
    }

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

    (*Context)->Pageable = Pageable;

    auto const allocRoutine = (Pageable == ST_PAGEABLE::YES) ? TreeAllocateRoutinePaged : TreeAllocateRoutineNonPaged;

    RtlInitializeGenericTableAvl(&(*Context)->Tree, TreeCompareRoutine, allocRoutine, TreeFreeRoutine, NULL);

    return STATUS_SUCCESS;
}

void TearDown(CONTEXT** Context) {
    Reset(*Context);

    ExFreePoolWithTag(*Context, ST_POOL_TAG);

    *Context = NULL;
}

void Reset(CONTEXT* Context) {
    for (;;) {
        auto entry = (PROCESS_REGISTRY_ENTRY*)RtlGetElementGenericTableAvl(&Context->Tree, 0);

        if (entry == NULL) {
            break;
        }

        //
        // It's believed that `InnerDeleteEntry` will never fail as long as
        // the following conditions are met:
        //
        // - The tree's CompareRoutine and FreeRoutine are correctly implemented.
        // - Nodes in the tree are regarded as internally consistent by tree
        // CompareRoutine.
        // - `entry` argument is valid.
        //

        InnerDeleteEntry(Context, entry);
    }
}

NTSTATUS
InitializeEntry(
    CONTEXT* Context,
    HANDLE ParentProcessId,
    HANDLE ProcessId,
    ST_PROCESS_SPLIT_STATUS Split,
    UNICODE_STRING* ImageName,
    PROCESS_REGISTRY_ENTRY* Entry) {
    RtlZeroMemory(Entry, sizeof(*Entry));

    if (ImageName != NULL && ImageName->Length != 0) {
        LOWER_UNICODE_STRING lowerImageName;

        auto status = util::AllocateCopyDowncaseString(&lowerImageName, ImageName, Context->Pageable);

        if (!NT_SUCCESS(status)) {
            return status;
        }

        Entry->ImageName = lowerImageName;
    }

    Entry->ParentProcessId = ParentProcessId;
    Entry->ProcessId = ProcessId;

    static const PROCESS_REGISTRY_ENTRY_SETTINGS settings = { .Split = ST_PROCESS_SPLIT_STATUS_OFF,
                                                              .HasFirewallState = false };

    Entry->Settings = { Split, false };
    Entry->TargetSettings = settings;
    Entry->PreviousSettings = settings;

    Entry->ParentEntry = NULL;

    return STATUS_SUCCESS;
}

NTSTATUS
AddEntry(CONTEXT* Context, PROCESS_REGISTRY_ENTRY* Entry) {
    //
    // Insert entry into tree.
    // This makes a copy of the entry.
    //

    BOOLEAN newElement;

    auto record = RtlInsertElementGenericTableAvl(&Context->Tree, Entry, (CLONG)sizeof(*Entry), &newElement);

    if (record != NULL && newElement != FALSE) {
        return STATUS_SUCCESS;
    }

    //
    // Handle failure cases.
    //

    if (record == NULL) {
        // Allocation of record failed.
        return STATUS_INSUFFICIENT_RESOURCES;
    }

    // There's already a record for this PID.
    return STATUS_DUPLICATE_OBJECTID;
}

void ReleaseEntry(PROCESS_REGISTRY_ENTRY* Entry) {
    if (Entry->ImageName.Buffer != NULL) {
        util::FreeStringBuffer(&Entry->ImageName);
    }
}

PROCESS_REGISTRY_ENTRY* FindEntry(CONTEXT* Context, HANDLE ProcessId) {
    PROCESS_REGISTRY_ENTRY record = { 0 };

    record.ProcessId = ProcessId;

    return (PROCESS_REGISTRY_ENTRY*)RtlLookupElementGenericTableAvl(&Context->Tree, &record);
}

bool DeleteEntry(CONTEXT* Context, PROCESS_REGISTRY_ENTRY* Entry) {
    auto const processId = Entry->ProcessId;

    auto const status = InnerDeleteEntry(Context, Entry);

    if (status) {
        ForEach(Context, ClearDepartingParentLink, processId);
    }

    return status;
}

bool DeleteEntryById(CONTEXT* Context, HANDLE ProcessId) {
    auto entry = FindEntry(Context, ProcessId);

    if (entry == NULL) {
        return false;
    }

    return DeleteEntry(Context, entry);
}

bool ForEach(CONTEXT* Context, ST_PR_FOREACH Callback, void* ClientContext) {
    for (auto entry = RtlEnumerateGenericTableAvl(&Context->Tree, TRUE); entry != NULL;
         entry = RtlEnumerateGenericTableAvl(&Context->Tree, FALSE)) {
        if (!Callback((PROCESS_REGISTRY_ENTRY*)entry, ClientContext)) {
            return false;
        }
    }

    return true;
}

PROCESS_REGISTRY_ENTRY* GetParentEntry(CONTEXT* Context, PROCESS_REGISTRY_ENTRY* Entry) {
    if (NULL != Entry->ParentEntry) {
        return Entry->ParentEntry;
    }

    if (Entry->ParentProcessId == 0) {
        return NULL;
    }

    return (Entry->ParentEntry = FindEntry(Context, Entry->ParentProcessId));
}

bool IsEmpty(CONTEXT* Context) {
    return FALSE != RtlIsGenericTableEmptyAvl(&Context->Tree);
}

} // namespace procregistry
